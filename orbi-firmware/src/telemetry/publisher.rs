use esp_hal::delay::Delay;
use esp_println::println;

use crate::{
    drivers::{gnss::GpsInfo, Modem},
    network::{heartbeat, http},
    storage::RecordStorage,
    telemetry::{payload, record::TelemetryRecord, replay},
};

pub fn publish_live_fix<S>(
    modem: &mut Modem,
    delay: &Delay,
    device_code: &str,
    gps_info: &GpsInfo,
    mut storage: Option<&mut S>,
    send_heartbeat: bool,
) -> bool
where
    S: RecordStorage,
{
    println!("========================");
    println!("LIVE GPS PARSED");
    println!("========================");
    println!("Latitude: {}", gps_info.latitude);
    println!("Longitude: {}", gps_info.longitude);
    println!("Speed: {}", gps_info.speed);
    println!("Heading: {}", gps_info.heading);
    println!("Timestamp: {}", gps_info.timestamp);

    let live_reading = TelemetryRecord {
        device_id: device_code,
        timestamp: gps_info.timestamp.as_str(),

        latitude: gps_info.latitude,
        longitude: gps_info.longitude,

        fuel_level_litres: 0.0,
        fuel_level_percentage: 0.0,

        vibration_level: 0.0,
        motion_detected: false,

        speed: gps_info.speed,
        heading: gps_info.heading,

        simulation_mode: "real_gps_only",
    };

    println!("========================");
    println!("SAVING TELEMETRY TO SD");
    println!("========================");

    if let Some(storage) = storage.as_mut() {
        if !storage.append_record(&live_reading) {
            println!("Telemetry SD append failed.");
        }
    } else {
        println!("SD storage unavailable. Continuing with upload.");
    }

    let heartbeat_success = if send_heartbeat {
        let heartbeat_payload =
            heartbeat::build_heartbeat_payload(device_code, gps_info.timestamp.as_str());

        http::send_heartbeat(modem, delay, &heartbeat_payload)
    } else {
        println!("Heartbeat not due. Telemetry upload will confirm device activity.");
        false
    };

    let live_payload = payload::build_gps_only_payload(&live_reading);

    println!("========================");
    println!("LIVE GPS PAYLOAD");
    println!("========================");
    println!("{}", live_payload);

    let upload_success = http::send_payload(modem, delay, &live_payload);

    if upload_success {
        if let Some(storage) = storage.as_mut() {
            if storage.append_ack(live_reading.device_id, live_reading.timestamp) {
                println!("Telemetry upload acknowledged. Cleaning completed queue records.");

                replay::cleanup_acknowledged_records(Some(&mut **storage));
            } else {
                println!("Telemetry ACK append failed. Record remains in ORBIQ.LOG.");
            }
        } else {
            println!("Upload succeeded, but SD storage is unavailable for ACK.");
        }
    } else {
        println!("Telemetry upload failed. Record remains pending in ORBIQ.LOG.");
    }

    upload_success || heartbeat_success
}
