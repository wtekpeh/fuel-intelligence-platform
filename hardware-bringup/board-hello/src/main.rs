#![no_std]
#![no_main]

mod board;
mod http;
mod i2c_scan;
mod modem;

use board::BoardPins;
use esp_backtrace as _;
use esp_hal::delay::Delay;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::main;
use esp_println::println;
use modem::Modem;

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

        modem.send_command_and_print_response(b"AT+CGPSINFO\r\n", "AT+CGPSINFO", &delay);
        delay.delay_millis(5000);

        modem.send_command_and_print_response(b"AT+CAGPS\r\n", "AT+CAGPS", &delay);
        delay.delay_millis(1000);

        http::run_http_diagnostic(&mut modem, &delay);

        delay.delay_millis(10000);
    }
}
