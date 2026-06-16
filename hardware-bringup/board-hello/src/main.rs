#![no_std]
#![no_main]

mod board;
mod gnss;
mod http;
mod i2c_scan;
mod modem;

use board::BoardPins;
use esp_backtrace as _;
use esp_hal::delay::Delay;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::main;
use modem::Modem;
mod telemetry_payload;
use esp_println::{print, println};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    println!("A7670E modem AT test starting from modules...");

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
        device_id: "ORBI-GPS-001",
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
        modem.send_command_and_print_response(b"AT\r\n", "AT", &delay);
        delay.delay_millis(1000);

        modem.send_command_and_print_response(b"AT+CPIN?\r\n", "AT+CPIN?", &delay);
        delay.delay_millis(1000);

        modem.send_command_and_print_response(b"AT+CSQ\r\n", "AT+CSQ", &delay);
        delay.delay_millis(1000);

        modem.send_command_and_print_response(b"AT+CEREG?\r\n", "AT+CEREG?", &delay);
        delay.delay_millis(1000);

        modem.send_command_and_print_response(b"AT+CGATT?\r\n", "AT+CGATT?", &delay);
        delay.delay_millis(1000);

        modem.send_command_and_print_response(b"AT+CGPADDR\r\n", "AT+CGPADDR", &delay);
        delay.delay_millis(1000);

        modem.send_command_and_print_response(b"AT+CGNSSPWR=1\r\n", "AT+CGNSSPWR=1", &delay);
        delay.delay_millis(2000);

        modem.send_command_and_print_response(b"AT+CGNSSPWR?\r\n", "AT+CGNSSPWR?", &delay);
        delay.delay_millis(1000);

        modem.send_command_and_print_response(b"AT+CGNSINF\r\n", "AT+CGNSINF", &delay);
        delay.delay_millis(1000);

        modem.send_command_and_print_response(b"AT+CGPS=1\r\n", "AT+CGPS=1", &delay);
        delay.delay_millis(2000);

        let gps_response =
            modem.send_command_and_collect_response(b"AT+CGPSINFO\r\n", "AT+CGPSINFO", &delay);

        if let Some(response_buffer) = gps_response {
            println!("Raw GPS response buffer:");
            for byte in response_buffer {
                if byte != 0 {
                    print!("{}", byte as char);
                }
            }
            println!();

            if let Some(gps_info) = gnss::parse_cgpsinfo_response(&response_buffer) {
                println!("========================");
                println!("LIVE GPS PARSED");
                println!("========================");
                println!("Latitude: {}", gps_info.latitude);
                println!("Longitude: {}", gps_info.longitude);
                println!("Speed: {}", gps_info.speed);
                println!("Timestamp: {}", gps_info.timestamp);

                let live_reading = telemetry_payload::GpsReading {
                    device_id: "ORBI-GPS-001",
                    timestamp: gps_info.timestamp.as_str(),
                    latitude: gps_info.latitude,
                    longitude: gps_info.longitude,
                    speed: gps_info.speed,
                    heading: 0.0,
                };

                let live_payload = telemetry_payload::build_gps_only_payload(&live_reading);

                println!("========================");
                println!("LIVE GPS PAYLOAD");
                println!("========================");
                println!("{}", live_payload);
                http::send_payload(&mut modem, &delay, &live_payload);
            } else {
                println!("Could not parse GPS response.");
            }
        }
        delay.delay_millis(5000);
    }
}
