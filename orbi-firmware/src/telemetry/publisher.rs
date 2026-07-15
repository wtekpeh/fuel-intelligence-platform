use esp_hal::delay::Delay;
use esp_println::println;

use crate::{
    device::DEVICE_IDENTITY,
    drivers::{gnss::GpsInfo, Modem},
    network::{heartbeat, http},
    storage::RecordStorage,
    telemetry::{payload, record::TelemetryRecord},
};

pub fn publish_live_fix<S>(
    modem: &mut Modem,
    delay: &Delay,
    gps_info: &GpsInfo,
    mut storage: Option<&mut S>,
) where
    S: RecordStorage,
{
    println!("========================");
    println!("LIVE GPS PARSED");
    println!("========================");
    println!("Latitude: {}", gps_info.latitude);
    println!("Longitude: {}", gps_info.longitude);
    println!("Speed: {}", gps_info.speed);
    println!("Timestamp: {}", gps_info.timestamp);

    let live_reading = TelemetryRecord {
        device_id: DEVICE_IDENTITY.device_code,
        timestamp: gps_info.timestamp.as_str(),

        latitude: gps_info.latitude,
        longitude: gps_info.longitude,

        fuel_level_litres: 0.0,
        fuel_level_percentage: 0.0,

        vibration_level: 0.0,
        motion_detected: false,

        speed: gps_info.speed,
        heading: 0.0,

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

    let heartbeat_payload = heartbeat::build_heartbeat_payload(gps_info.timestamp.as_str());

    let _heartbeat_success = http::send_heartbeat(modem, delay, &heartbeat_payload);

    let live_payload = payload::build_gps_only_payload(&live_reading);

    println!("========================");
    println!("LIVE GPS PAYLOAD");
    println!("========================");
    println!("{}", live_payload);

    let upload_success = http::send_payload(modem, delay, &live_payload);

    if upload_success {
        if let Some(storage) = storage.as_mut() {
            if !storage.append_ack(live_reading.device_id, live_reading.timestamp) {
                println!("Telemetry ACK append failed.");
            }
        } else {
            println!("Upload succeeded, but SD storage is unavailable for ACK.");
        }
    } else {
        println!("Telemetry upload failed. Record remains pending in ORBIQ.LOG.");
    }
}
