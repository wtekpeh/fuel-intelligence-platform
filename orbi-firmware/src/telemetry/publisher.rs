use esp_hal::delay::Delay;
use esp_println::println;

use crate::{
    drivers::{gnss::GpsInfo, vibration::ImuData, Modem},
    network::{heartbeat, http},
    storage::RecordStorage,
    telemetry::{payload, record::TelemetryRecord},
};

/*
 * Maximum number of queued records included in one HTTP request.
 *
 * Four records keep memory usage bounded and leave room inside the
 * 4096-byte payload buffer used by build_queue_batch_payload().
 */
const QUEUE_BATCH_SIZE: usize = 4;

/*
 * Prevent one publishing operation from occupying the modem indefinitely.
 *
 * With a batch size of four, this permits up to 100 queued records to be
 * processed during one flush operation.
 */
const MAX_RECORDS_PER_FLUSH: usize = 100;

/*
 * Remove acknowledged records from the front of the persistent queue.
 *
 * This handles cases where:
 *
 * - an earlier upload succeeded;
 * - one or more ACK entries were persisted;
 * - queue removal was interrupted by power loss or another storage failure.
 *
 * Because ORBIQ.LOG is FIFO, cleanup stops immediately when the first
 * unacknowledged record is encountered.
 */
fn cleanup_acknowledged_queue_front<S>(storage: &mut S) -> usize
where
    S: RecordStorage,
{
    let mut removed = 0;

    loop {
        let queued_record = match storage.read_first_record() {
            Some(record) => record,

            None => {
                break;
            }
        };

        let (device_id, timestamp) = match payload::extract_replay_identity(queued_record.as_str())
        {
            Some(identity) => identity,

            None => {
                println!("Invalid record found at the front of ORBIQ.LOG.");
                println!("Acknowledged queue cleanup stopped.");

                break;
            }
        };

        if !storage.is_acknowledged(device_id, timestamp) {
            break;
        }

        println!("Removing previously acknowledged queued record.");

        if !storage.remove_first_record() {
            println!("Failed to remove previously acknowledged queued record.");

            break;
        }

        removed += 1;
    }

    removed
}

/*
 * Flush queued telemetry through the normal telemetry HTTP endpoint.
 *
 * ORBIQ.LOG remains the source of truth:
 *
 * 1. Read records from the persistent queue.
 * 2. Build one batch payload directly from those queued JSON records.
 * 3. Upload the batch.
 * 4. Persist an ACK for every uploaded record.
 * 5. Remove the acknowledged records from the front of the queue.
 *
 * Records are never removed before the upload and ACK stages complete.
 */
pub fn flush_queue<S>(modem: &mut Modem, delay: &Delay, mut storage: Option<&mut S>) -> bool
where
    S: RecordStorage,
{
    println!("========================");
    println!("ORBI QUEUE PUBLISHER");
    println!("========================");

    let storage = match storage.as_deref_mut() {
        Some(storage) => storage,

        None => {
            println!("SD storage unavailable. Queue publishing skipped.");

            return false;
        }
    };

    let recovered_records = cleanup_acknowledged_queue_front(storage);

    if recovered_records > 0 {
        println!(
            "Removed {} previously acknowledged queued record(s).",
            recovered_records
        );
    }

    let mut published_records = 0usize;
    let mut uploaded_any_batch = false;

    while published_records < MAX_RECORDS_PER_FLUSH {
        let remaining_capacity = MAX_RECORDS_PER_FLUSH - published_records;
        let requested_records = core::cmp::min(QUEUE_BATCH_SIZE, remaining_capacity);

        let queued_records = storage.read_first_records::<QUEUE_BATCH_SIZE>(requested_records);

        if queued_records.is_empty() {
            if !uploaded_any_batch {
                println!("No pending telemetry records found.");
            }

            break;
        }

        let batch_record_count = queued_records.len();

        let batch_payload = match payload::build_queue_batch_payload(&queued_records) {
            Some(payload) => payload,

            None => {
                println!("Failed to build telemetry queue batch payload.");
                println!("Queue remains unchanged.");

                break;
            }
        };

        println!("========================");
        println!("QUEUED TELEMETRY BATCH");
        println!("========================");
        println!("Records: {}", batch_record_count);
        println!("{}", batch_payload);

        if !http::send_payload(modem, delay, &batch_payload) {
            println!("Queued telemetry upload failed.");
            println!("Queue remains unchanged.");

            break;
        }

        /*
         * Persist every ACK before removing any queue entry.
         *
         * If ACK persistence stops partway through, no records are removed
         * here. On the next flush, cleanup_acknowledged_queue_front() will
         * safely remove only the records whose ACKs were persisted.
         */
        let mut all_acks_persisted = true;

        for queued_record in queued_records.iter() {
            let (device_id, timestamp) =
                match payload::extract_replay_identity(queued_record.as_str()) {
                    Some(identity) => identity,

                    None => {
                        println!("Unable to extract identity from uploaded record.");
                        all_acks_persisted = false;

                        break;
                    }
                };

            if !storage.append_ack(device_id, timestamp) {
                println!("Failed to persist telemetry batch ACK.");
                all_acks_persisted = false;

                break;
            }
        }

        if !all_acks_persisted {
            println!("Batch upload succeeded, but ACK persistence was incomplete.");
            println!("Queue records have not been removed.");

            break;
        }

        let mut removed_records = 0usize;

        for _ in 0..batch_record_count {
            if !storage.remove_first_record() {
                println!("Failed while removing acknowledged queue records.");

                break;
            }

            removed_records += 1;
        }

        published_records += removed_records;
        uploaded_any_batch = true;

        println!(
            "Published and removed {} queued telemetry record(s).",
            removed_records
        );

        if removed_records != batch_record_count {
            println!(
                "Only {} of {} acknowledged record(s) were removed.",
                removed_records, batch_record_count
            );

            /*
             * Remaining records already have ACK entries. They will be
             * recovered safely by cleanup_acknowledged_queue_front() during
             * the next queue flush.
             */
            break;
        }
    }

    if published_records >= MAX_RECORDS_PER_FLUSH {
        println!(
            "Queue flush limit reached after {} record(s).",
            published_records
        );
    }

    println!(
        "Queue publishing completed. {} record(s) processed.",
        published_records
    );

    uploaded_any_batch
}

pub fn publish_live_fix<S>(
    modem: &mut Modem,
    delay: &Delay,
    device_code: &str,
    gps_info: &GpsInfo,
    imu_data: &ImuData,
    mut storage: Option<&mut S>,
    send_heartbeat: bool,
) -> bool
where
    S: RecordStorage,
{
    println!("========================");
    println!("LIVE SENSOR MEASUREMENTS");
    println!("========================");

    println!("Latitude: {}", gps_info.latitude);
    println!("Longitude: {}", gps_info.longitude);
    println!("Speed: {}", gps_info.speed);
    println!("Heading: {}", gps_info.heading);
    println!("Timestamp: {}", gps_info.timestamp);

    println!("Accelerometer X: {} g", imu_data.accel_x_g);
    println!("Accelerometer Y: {} g", imu_data.accel_y_g);
    println!("Accelerometer Z: {} g", imu_data.accel_z_g);

    println!("Gyroscope X: {} dps", imu_data.gyro_x_dps);
    println!("Gyroscope Y: {} dps", imu_data.gyro_y_dps);
    println!("Gyroscope Z: {} dps", imu_data.gyro_z_dps);

    println!("IMU Temperature: {} C", imu_data.temperature_c);

    /*
     * The firmware records physical measurements only.
     *
     * Movement classification, vibration severity, impact detection and
     * alert generation remain backend intelligence responsibilities.
     */
    let live_reading = TelemetryRecord {
        device_id: device_code,
        timestamp: gps_info.timestamp.as_str(),

        latitude: gps_info.latitude,
        longitude: gps_info.longitude,

        speed: gps_info.speed,
        heading: gps_info.heading,

        /*
         * Fuel sensor integration will be completed during the later
         * fuel-measurement phase.
         */
        fuel_level_litres: 0.0,
        fuel_level_percentage: 0.0,

        accel_x_g: imu_data.accel_x_g,
        accel_y_g: imu_data.accel_y_g,
        accel_z_g: imu_data.accel_z_g,

        gyro_x_dps: imu_data.gyro_x_dps,
        gyro_y_dps: imu_data.gyro_y_dps,
        gyro_z_dps: imu_data.gyro_z_dps,

        imu_temperature_c: imu_data.temperature_c,

        simulation_mode: "physical_gps_imu",
    };

    println!("========================");
    println!("PERSISTING LIVE TELEMETRY");
    println!("========================");

    /*
     * Offline-first invariant:
     *
     * The live measurement must enter ORBIQ.LOG before any telemetry upload
     * is attempted. HTTP publishing reads from the queue rather than directly
     * from the in-memory TelemetryRecord.
     */
    let telemetry_persisted = match storage.as_deref_mut() {
        Some(storage) => {
            if storage.append_record(&live_reading) {
                true
            } else {
                println!("Telemetry SD append failed.");
                println!("Live telemetry will not be uploaded from volatile memory.");

                false
            }
        }

        None => {
            println!("SD storage unavailable.");
            println!("Live telemetry cannot enter the persistent queue.");

            false
        }
    };

    let heartbeat_success = if send_heartbeat {
        let heartbeat_payload =
            heartbeat::build_heartbeat_payload(device_code, gps_info.timestamp.as_str());

        http::send_heartbeat(modem, delay, &heartbeat_payload)
    } else {
        println!("Heartbeat not due. Telemetry publishing may confirm device activity.");

        false
    };

    let telemetry_upload_success = if telemetry_persisted {
        flush_queue(modem, delay, storage.as_deref_mut())
    } else {
        println!("Queue publishing skipped because the live record was not persisted.");

        false
    };

    telemetry_upload_success || heartbeat_success
}
