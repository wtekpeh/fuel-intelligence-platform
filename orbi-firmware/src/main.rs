#![no_std]
#![no_main]

mod board;
mod device;
mod drivers;
mod network;
mod scheduler;
mod storage;
mod telemetry;

use board::BoardPins;
use drivers::Modem;
use esp_backtrace as _;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::main;
use esp_hal::{delay::Delay, time::Instant};

use device::{load_runtime_identity, FIRMWARE_IDENTITY};
use esp_println::println;
use telemetry::record::TelemetryRecord;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    let runtime_identity = device::storage::load_runtime_identity_from_flash(peripherals.FLASH)
        .unwrap_or_else(load_runtime_identity);

    println!("A7670E modem AT test starting from modules...");
    println!("========================");
    println!("ORBI DEVICE IDENTITY");
    println!("========================");
    println!("Device Code: {}", runtime_identity.device_code());
    println!("Provisioned: {}", runtime_identity.is_provisioned());
    println!("Firmware: {}", FIRMWARE_IDENTITY.firmware_version);
    println!("Product: {}", FIRMWARE_IDENTITY.product_code);
    println!(
        "Hardware Profile: {}",
        FIRMWARE_IDENTITY.hardware_profile_code
    );
    println!(
        "Capabilities: GPS={} FUEL={} VIBRATION={} KILL_SWITCH={}",
        FIRMWARE_IDENTITY.capabilities.gps,
        FIRMWARE_IDENTITY.capabilities.fuel,
        FIRMWARE_IDENTITY.capabilities.vibration,
        FIRMWARE_IDENTITY.capabilities.kill_switch,
    );

    let mut persistent_storage = storage::sdcard::initialize(
        peripherals.SPI2,
        peripherals.GPIO2,
        peripherals.GPIO15,
        peripherals.GPIO14,
        peripherals.GPIO13,
    );

    let mut board_pins = BoardPins::new(peripherals.GPIO12, peripherals.GPIO5, peripherals.GPIO4);

    Modem::power_on(
        &mut board_pins.modem_power_on,
        &mut board_pins.modem_reset,
        &mut board_pins.modem_pwrkey,
        &delay,
    );

    let mut modem = Modem::new(peripherals.UART1, peripherals.GPIO26, peripherals.GPIO27);

    drivers::gnss::initialize(&mut modem, &delay);

    drivers::i2c::print_scan_banner();

    let mut i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
        .unwrap()
        .with_sda(peripherals.GPIO32)
        .with_scl(peripherals.GPIO33);

    drivers::i2c::scan_i2c_bus(&mut i2c);

    let test_reading = TelemetryRecord {
        device_id: runtime_identity.device_code(),
        timestamp: "2026-06-16T14:37:02Z",

        latitude: 51.8776168,
        longitude: -0.4291513,

        fuel_level_litres: 0.0,
        fuel_level_percentage: 0.0,

        vibration_level: 0.0,
        motion_detected: false,

        speed: 0.0,
        heading: 335.36,

        simulation_mode: "real_gps_only",
    };

    let payload = telemetry::payload::build_gps_only_payload(&test_reading);

    println!("========================");
    println!("GPS PAYLOAD TEST");
    println!("========================");
    println!("{}", payload);

    let latitude = drivers::gnss::convert_nmea_latitude("5152.65701", "N").unwrap();

    let longitude = drivers::gnss::convert_nmea_longitude("00025.74908", "W").unwrap();

    println!("========================");
    println!("GNSS CONVERSION TEST");
    println!("========================");
    println!("Latitude: {}", latitude);
    println!("Longitude: {}", longitude);

    println!("========================");
    println!("WAITING FOR NETWORK BEFORE REPLAY");
    println!("========================");

    let mut network_ready = false;

    for attempt in 1..=6 {
        println!("Network readiness attempt {}/6", attempt);

        let state = network::state::read_network_state(&mut modem, &delay);

        if state.is_ready() {
            network_ready = true;
            println!("Network ready for queued telemetry replay.");
            break;
        }

        println!("Network not ready yet. Waiting 5 seconds...");
        delay.delay_millis(5000);
    }

    if network_ready {
        telemetry::replay::replay_pending_records(&mut modem, &delay, persistent_storage.as_mut());
    } else {
        println!("Network did not become ready. Replay skipped for this boot.");
    }

    let reporting_policy = scheduler::reporting::ReportingPolicy::default();

    let mut next_reporting_interval_ms = reporting_policy.parked_interval_ms;

    const HEARTBEAT_INTERVAL_SECONDS: u64 = 300;
    const DIAGNOSTICS_INTERVAL_SECONDS: u64 = 600;

    let mut last_successful_cloud_contact = Instant::now();

    let mut last_network_diagnostics = Instant::now();

    loop {
        let diagnostics_due =
            last_network_diagnostics.elapsed().as_secs() >= DIAGNOSTICS_INTERVAL_SECONDS;

        if diagnostics_due {
            println!("========================");
            println!("PERIODIC NETWORK DIAGNOSTICS");
            println!("========================");

            network::diagnostics::run_network_diagnostics(&mut modem, &delay);

            last_network_diagnostics = Instant::now();
        } else {
            println!("Periodic network diagnostics not due.");
        }

        if let Some(gps_info) = drivers::gnss::get_live_fix(&mut modem, &delay) {
            let heartbeat_due =
                last_successful_cloud_contact.elapsed().as_secs() >= HEARTBEAT_INTERVAL_SECONDS;

            println!("Heartbeat due: {}", heartbeat_due);

            let cloud_contact_succeeded = telemetry::publisher::publish_live_fix(
                &mut modem,
                &delay,
                runtime_identity.device_code(),
                &gps_info,
                persistent_storage.as_mut(),
                heartbeat_due,
            );

            if cloud_contact_succeeded {
                last_successful_cloud_contact = Instant::now();
            } else {
                println!("========================");
                println!("CLOUD CONTACT FAILED");
                println!("Running immediate network diagnostics...");
                println!("========================");

                network::diagnostics::run_network_diagnostics(&mut modem, &delay);

                last_network_diagnostics = Instant::now();
            }

            next_reporting_interval_ms = reporting_policy.next_interval_from_speed(gps_info.speed);
        } else {
            println!("Could not obtain or parse GPS response.");

            // Retry sooner when a GNSS fix temporarily fails.
            next_reporting_interval_ms = 30_000;
        }

        println!(
            "Waiting {} seconds before the next telemetry cycle.",
            next_reporting_interval_ms / 1000
        );

        delay.delay_millis(next_reporting_interval_ms);
    }
}
