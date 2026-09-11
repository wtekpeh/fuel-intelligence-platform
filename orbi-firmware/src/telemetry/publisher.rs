use esp_println::println;

use crate::{
    storage::{
        owned_record::{OwnedAckRecord, OwnedTelemetryRecord},
        owner as storage_owner,
    },
    telemetry::{payload, record::TelemetryRecord, snapshot::SensorSnapshot},
};

pub async fn publish_live_fix(device_code: &str, snapshot: &SensorSnapshot<'_>) -> bool {
    let gps_info = snapshot.gps;
    let imu_data = snapshot.imu;

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

    match snapshot.fuel {
        Some(measurement) => {
            println!(
                "Fuel smooth distance: {} cm",
                measurement.smooth_distance_cm
            );

            println!(
                "Fuel real-time distance: {} cm",
                measurement.realtime_distance_cm
            );

            println!("Fuel raw distance: {} cm", measurement.raw_distance_cm);

            println!("Fuel sensor temperature: {} C", measurement.temperature_c);

            println!("Fuel status byte 1: 0x{:02X}", measurement.status_byte_1);

            println!("Fuel status byte 2: 0x{:02X}", measurement.status_byte_2);

            println!(
                "Fuel raw data validity: 0x{:02X}",
                measurement.raw_data_validity
            );
        }

        None => {
            println!("Fuel measurement unavailable for this telemetry cycle.");
        }
    }

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
         * The KUM driver now provides raw physical measurements.
         *
         * Tank calibration, fuel volume, and percentage remain backend
         * responsibilities, so the firmware publishes the raw sensor
         * measurements only.
         */
        fuel_distance_smooth_cm: snapshot
            .fuel
            .map_or(0.0, |measurement| measurement.smooth_distance_cm),

        fuel_distance_realtime_cm: snapshot
            .fuel
            .map_or(0.0, |measurement| measurement.realtime_distance_cm),

        fuel_distance_raw_cm: snapshot
            .fuel
            .map_or(0.0, |measurement| measurement.raw_distance_cm),

        fuel_sensor_temperature_c: snapshot
            .fuel
            .map_or(0.0, |measurement| measurement.temperature_c),

        fuel_sensor_status_1: snapshot
            .fuel
            .map_or(0, |measurement| measurement.status_byte_1),

        fuel_sensor_status_2: snapshot
            .fuel
            .map_or(0, |measurement| measurement.status_byte_2),

        fuel_raw_data_validity: snapshot
            .fuel
            .map_or(0, |measurement| measurement.raw_data_validity),

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
    let telemetry_persisted = match OwnedTelemetryRecord::from_borrowed(&live_reading) {
        Some(owned_record) => {
            if storage_owner::request_live_telemetry_persistence(owned_record).await {
                true
            } else {
                println!("Telemetry SD append failed.");

                println!("Live telemetry was not persisted.");

                false
            }
        }

        None => {
            println!("Unable to create owned telemetry record.");

            println!("Live telemetry was not persisted.");

            false
        }
    };

    let live_payload = payload::build_telemetry_payload(&live_reading);

    let telemetry_upload_success = if telemetry_persisted {
        println!("========================");
        println!("DIRECT LIVE TELEMETRY");
        println!("========================");
        println!("Uploading newest live telemetry ahead of queued backlog.");

        let upload_succeeded =
            crate::network::modem_owner::request_live_telemetry(live_payload).await;

        if upload_succeeded {
            println!("Live telemetry upload succeeded.");

            /*
             * Persist the ACK for this exact live record.
             *
             * The physical record remains in ORBIQ.LOG if older records are
             * still ahead of it. Later replay will eventually reach this record,
             * see that it is already acknowledged, and remove it without
             * uploading it again.
             */
            match OwnedAckRecord::new(live_reading.device_id, live_reading.timestamp) {
                Some(ack) => {
                    if storage_owner::request_live_ack(ack).await {
                        println!("Live telemetry ACK persisted.");
                    } else {
                        println!("Live telemetry upload succeeded, but ACK persistence failed.");

                        println!("The record remains queued and may be uploaded again later.");
                    }
                }

                None => {
                    println!(
                        "Live telemetry upload succeeded, but ACK ownership conversion failed."
                    );

                    println!("The record remains queued and may be uploaded again later.");
                }
            }
        }

        upload_succeeded
    } else {
        println!("========================");
        println!("DIRECT LIVE TELEMETRY FALLBACK");
        println!("========================");
        println!("SD persistence unavailable.");
        println!("Attempting direct telemetry upload.");

        crate::network::modem_owner::request_live_telemetry(live_payload).await
    };

    telemetry_upload_success
}
