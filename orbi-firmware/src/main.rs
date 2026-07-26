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
use scheduler::reporting::{knots_to_kmh, MotionState};
use storage::record::GnssDiagnosticRecord;
use storage::service::RecordStorage;
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

    let vibration_sensor_ready = drivers::vibration::initialize(&mut i2c);

    println!("========================");
    println!("ORBI VIBRATION SENSOR STATUS");
    println!("========================");
    println!("MPU6050 ready: {}", vibration_sensor_ready);

    if vibration_sensor_ready {
        drivers::vibration::print_imu_data(&mut i2c);
    } else {
        println!("WARNING: MPU6050 unavailable. Continuing without IMU measurements.");
    }

    let imu_test = drivers::vibration::read_imu_data(&mut i2c);

    let test_reading = TelemetryRecord {
        device_id: runtime_identity.device_code(),
        timestamp: "2026-06-16T14:37:02Z",

        latitude: 51.8776168,
        longitude: -0.4291513,

        speed: 0.0,
        heading: 335.36,

        fuel_level_litres: 0.0,
        fuel_level_percentage: 0.0,

        accel_x_g: imu_test.as_ref().map_or(0.0, |v| v.accel_x_g),
        accel_y_g: imu_test.as_ref().map_or(0.0, |v| v.accel_y_g),
        accel_z_g: imu_test.as_ref().map_or(0.0, |v| v.accel_z_g),

        gyro_x_dps: imu_test.as_ref().map_or(0.0, |v| v.gyro_x_dps),
        gyro_y_dps: imu_test.as_ref().map_or(0.0, |v| v.gyro_y_dps),
        gyro_z_dps: imu_test.as_ref().map_or(0.0, |v| v.gyro_z_dps),

        imu_temperature_c: imu_test.as_ref().map_or(0.0, |v| v.temperature_c),

        simulation_mode: "physical_gps_imu",
    };

    let payload = telemetry::payload::build_telemetry_payload(&test_reading);

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

        delay.delay_millis(5_000);
    }

    if network_ready {
        telemetry::replay::replay_pending_records(&mut modem, &delay, persistent_storage.as_mut());
    } else {
        println!("Network did not become ready. Replay skipped for this boot.");
    }

    /*
     * GNSS is sampled every 1 seconds.
     *
     * Cloud reporting remains adaptive:
     *
     * Moving -> 10 seconds
     * Idle   -> 20 seconds
     * Parked -> 30 seconds
     *
     * A motion-state change causes an immediate report.
     */
    const GNSS_SAMPLE_INTERVAL_MS: u32 = 1_000;

    const HEARTBEAT_INTERVAL_SECONDS: u64 = 300;
    const DIAGNOSTICS_INTERVAL_SECONDS: u64 = 600;

    let reporting_policy = scheduler::reporting::ReportingPolicy::default();

    /*
     * Assume parked when the firmware starts.
     *
     * If the vehicle is already moving, the first valid GNSS reading
     * will change Parked -> Moving and trigger an immediate upload.
     */
    let mut current_motion_state = MotionState::Parked;

    /*
     * This tracks the most recent cloud-report attempt.
     *
     * It is intentionally separate from the GNSS sampling timer.
     */
    let mut last_report_time = Instant::now();

    /*
     * Heartbeat scheduling is independent of GNSS, IMU measurements,
     * telemetry reporting and persistent storage.
     *
     * The boolean causes one heartbeat attempt immediately after startup.
     */
    let mut heartbeat_attempted_once = false;
    let mut last_heartbeat_attempt = Instant::now();

    /*
     * This tracks when network diagnostics were last performed.
     */
    let mut last_network_diagnostics = Instant::now();

    loop {
        /*
         * Heartbeat is evaluated before GNSS acquisition.
         *
         * It therefore remains available when:
         *
         * - no GNSS fix is available;
         * - the MPU6050 cannot be read;
         * - SD storage is unavailable;
         * - no telemetry report is due.
         */
        let heartbeat_due = !heartbeat_attempted_once
            || last_heartbeat_attempt.elapsed().as_secs() >= HEARTBEAT_INTERVAL_SECONDS;

        if heartbeat_due {
            println!("========================");
            println!("ORBI INDEPENDENT HEARTBEAT");
            println!("========================");

            let heartbeat_payload =
                network::heartbeat::build_heartbeat_payload(runtime_identity.device_code());

            let heartbeat_succeeded =
                network::http::send_heartbeat(&mut modem, &delay, &heartbeat_payload);

            /*
             * Record the attempt regardless of success.
             *
             * This prevents a network failure from causing another blocking
             * heartbeat request every one-second loop cycle.
             */
            heartbeat_attempted_once = true;
            last_heartbeat_attempt = Instant::now();

            if heartbeat_succeeded {
                println!("Independent heartbeat succeeded.");
            } else {
                println!("Independent heartbeat failed.");
                println!(
                    "The next heartbeat attempt will occur in {} seconds.",
                    HEARTBEAT_INTERVAL_SECONDS
                );
            }
        }

        println!("========================");
        println!("GNSS SAMPLE CYCLE START");
        println!("========================");

        /*
         * Run the broader modem/network diagnostics every 10 minutes.
         */
        let diagnostics_due =
            last_network_diagnostics.elapsed().as_secs() >= DIAGNOSTICS_INTERVAL_SECONDS;

        if diagnostics_due {
            println!("========================");
            println!("PERIODIC NETWORK DIAGNOSTICS");
            println!("========================");

            network::diagnostics::run_network_diagnostics(&mut modem, &delay);

            last_network_diagnostics = Instant::now();
        }

        /*
         * GNSS is queried every 5 seconds regardless of whether the
         * vehicle is moving, idle or parked.
         */
        if let Some(gps_info) = drivers::gnss::get_live_fix(&mut modem, &delay) {
            let speed_kmh = knots_to_kmh(gps_info.speed);

            let new_motion_state = reporting_policy.classify_speed_knots(gps_info.speed);

            let reporting_interval_ms = reporting_policy.interval_for(new_motion_state);

            let reporting_interval_seconds = reporting_interval_ms / 1_000;

            /*
             * Detect Parked -> Moving, Moving -> Idle,
             * Idle -> Parked and any other state transition.
             */
            let motion_state_changed = new_motion_state != current_motion_state;

            /*
             * Check whether the normal adaptive reporting interval
             * has elapsed.
             */
            let reporting_due =
                last_report_time.elapsed().as_secs() >= reporting_interval_seconds as u64;

            println!("========================");
            println!("GNSS MOTION DECISION");
            println!("========================");
            println!("Speed: {} knots", gps_info.speed);
            println!("Speed: {} km/h", speed_kmh);
            println!("Previous State: {:?}", current_motion_state);
            println!("New State: {:?}", new_motion_state);
            println!("State Changed: {}", motion_state_changed);
            println!("Reporting Due: {}", reporting_due);
            println!(
                "Selected Reporting Interval: {} seconds",
                reporting_interval_seconds
            );

            /*
             * Store every GNSS sample locally.
             *
             * This means ORBIGNSS.LOG should now contain samples
             * approximately every 5 seconds, not only cloud reports.
             */
            let diagnostic_record = GnssDiagnosticRecord {
                timestamp: &gps_info.timestamp,

                latitude: gps_info.latitude,
                longitude: gps_info.longitude,

                speed_knots: gps_info.speed,
                speed_kmh,

                heading_degrees: gps_info.heading,

                motion_state: new_motion_state,

                reporting_interval_seconds,
            };

            if persistent_storage.is_none() {
                println!("ERROR: Persistent storage is NONE.");
            } else {
                println!("Persistent storage is AVAILABLE.");

                let storage = persistent_storage.as_mut().unwrap();

                let diagnostic_saved = storage.append_gnss_diagnostic(&diagnostic_record);

                println!("append_gnss_diagnostic() returned: {}", diagnostic_saved);
            }

            /*
             * Publish when:
             *
             * 1. The motion state changed.
             * 2. The adaptive reporting interval elapsed.
             * 3. A heartbeat is due.
             */
            let should_publish = motion_state_changed || reporting_due;

            if should_publish {
                if motion_state_changed {
                    println!("Publishing immediately because motion state changed.");
                } else {
                    println!("Publishing because reporting interval elapsed.");
                }

                let imu_data = drivers::vibration::read_imu_data(&mut i2c);

                let imu_data = match imu_data {
                    Some(data) => {
                        println!("MPU6050 measurement obtained.");

                        data
                    }

                    None => {
                        println!("WARNING: MPU6050 measurement unavailable.");
                        println!("Publishing GPS telemetry using default IMU values.");

                        drivers::vibration::ImuData {
                            accel_x_g: 0.0,
                            accel_y_g: 0.0,
                            accel_z_g: 0.0,

                            gyro_x_dps: 0.0,
                            gyro_y_dps: 0.0,
                            gyro_z_dps: 0.0,

                            temperature_c: 0.0,
                        }
                    }
                };

                let cloud_contact_succeeded = telemetry::publisher::publish_live_fix(
                    &mut modem,
                    &delay,
                    runtime_identity.device_code(),
                    &gps_info,
                    &imu_data,
                    persistent_storage.as_mut(),
                );

                /*
                 * Record the attempt time whether the live upload
                 * succeeds or is queued for replay.
                 *
                 * This prevents a failed network connection from
                 * causing another long publish attempt every 5 seconds.
                 */
                last_report_time = Instant::now();

                if cloud_contact_succeeded {
                    println!("Cloud telemetry publish succeeded.");
                } else {
                    println!("========================");
                    println!("CLOUD CONTACT FAILED");
                    println!("Running immediate network diagnostics...");
                    println!("========================");

                    network::diagnostics::run_network_diagnostics(&mut modem, &delay);

                    last_network_diagnostics = Instant::now();
                }
            } else {
                println!("Cloud publish skipped for this GNSS sample.");
            }

            /*
             * Update the state only after completing the decision.
             *
             * This allows the current sample to detect a transition
             * from the previous state.
             */
            current_motion_state = new_motion_state;
        } else {
            println!("Could not obtain or parse GPS response.");

            println!("GNSS will be sampled again in 1 seconds.");
        }

        /*
         * This is now strictly the GNSS sampling interval.
         *
         * It no longer changes according to the reporting policy.
         */
        println!(
            "Waiting {} seconds before the next GNSS sample.",
            GNSS_SAMPLE_INTERVAL_MS / 1_000
        );

        delay.delay_millis(GNSS_SAMPLE_INTERVAL_MS);
    }
}
