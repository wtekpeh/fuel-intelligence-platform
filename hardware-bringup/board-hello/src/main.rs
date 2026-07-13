#![no_std]
#![no_main]

mod board;
mod device_identity;
mod gnss;
mod heartbeat;
mod http;
mod i2c_scan;
mod modem;
mod network;
mod telemetry;
mod telemetry_payload;

use board::BoardPins;
use esp_backtrace as _;
use esp_hal::delay::Delay;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::main;
use modem::Modem;

use device_identity::DEVICE_IDENTITY;
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    println!("A7670E modem AT test starting from modules...");
    println!("========================");
    println!("ORBI DEVICE IDENTITY");
    println!("========================");
    println!("Device Code: {}", DEVICE_IDENTITY.device_code);
    println!("Firmware: {}", DEVICE_IDENTITY.firmware_version);
    println!("Product: {}", DEVICE_IDENTITY.product_code);
    println!(
        "Hardware Profile: {}",
        DEVICE_IDENTITY.hardware_profile_code
    );
    println!(
        "Capabilities: GPS={} FUEL={} VIBRATION={} KILL_SWITCH={}",
        DEVICE_IDENTITY.capabilities.gps,
        DEVICE_IDENTITY.capabilities.fuel,
        DEVICE_IDENTITY.capabilities.vibration,
        DEVICE_IDENTITY.capabilities.kill_switch,
    );

    let mut board_pins = BoardPins::new(peripherals.GPIO12, peripherals.GPIO5, peripherals.GPIO4);

    Modem::power_on(
        &mut board_pins.modem_power_on,
        &mut board_pins.modem_reset,
        &mut board_pins.modem_pwrkey,
        &delay,
    );

    let mut modem = Modem::new(peripherals.UART1, peripherals.GPIO26, peripherals.GPIO27);

    i2c_scan::print_scan_banner();

    let mut i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
        .unwrap()
        .with_sda(peripherals.GPIO32)
        .with_scl(peripherals.GPIO33);

    i2c_scan::scan_i2c_bus(&mut i2c);

    let test_reading = telemetry_payload::GpsReading {
        device_id: DEVICE_IDENTITY.device_code,
        timestamp: "2026-06-16T14:37:02Z",
        latitude: 51.8776168,
        longitude: -0.4291513,
        speed: 0.0,
        heading: 335.36,
    };

    let payload = telemetry_payload::build_gps_only_payload(&test_reading);

    println!("========================");
    println!("GPS PAYLOAD TEST");
    println!("========================");
    println!("{}", payload);

    let latitude = gnss::convert_nmea_latitude("5152.65701", "N").unwrap();

    let longitude = gnss::convert_nmea_longitude("00025.74908", "W").unwrap();

    println!("========================");
    println!("GNSS CONVERSION TEST");
    println!("========================");
    println!("Latitude: {}", latitude);
    println!("Longitude: {}", longitude);

    loop {
        network::run_network_diagnostics(&mut modem, &delay);

        if let Some(gps_info) = gnss::get_live_fix(&mut modem, &delay) {
            telemetry::publish_live_fix(&mut modem, &delay, &gps_info);
        } else {
            println!("Could not obtain or parse GPS response.");
        }
        delay.delay_millis(5000);
    }
}
