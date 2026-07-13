use esp_hal::delay::Delay;
use esp_println::println;

use crate::{
    device::DEVICE_IDENTITY,
    drivers::{gnss::GpsInfo, Modem},
    network::{heartbeat, http},
    telemetry::payload,
};

pub fn publish_live_fix(modem: &mut Modem, delay: &Delay, gps_info: &GpsInfo) {
    println!("========================");
    println!("LIVE GPS PARSED");
    println!("========================");
    println!("Latitude: {}", gps_info.latitude);
    println!("Longitude: {}", gps_info.longitude);
    println!("Speed: {}", gps_info.speed);
    println!("Timestamp: {}", gps_info.timestamp);

    let heartbeat_payload = heartbeat::build_heartbeat_payload(gps_info.timestamp.as_str());

    http::send_heartbeat(modem, delay, &heartbeat_payload);

    let live_reading = payload::GpsReading {
        device_id: DEVICE_IDENTITY.device_code,
        timestamp: gps_info.timestamp.as_str(),
        latitude: gps_info.latitude,
        longitude: gps_info.longitude,
        speed: gps_info.speed,
        heading: 0.0,
    };

    let live_payload = payload::build_gps_only_payload(&live_reading);

    println!("========================");
    println!("LIVE GPS PAYLOAD");
    println!("========================");
    println!("{}", live_payload);

    http::send_payload(modem, delay, &live_payload);
}
