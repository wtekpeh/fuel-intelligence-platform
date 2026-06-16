use esp_hal::delay::Delay;
use esp_println::println;

use crate::modem::Modem;

pub fn send_payload(modem: &mut Modem, delay: &Delay, payload: &heapless::String<1024>) {
    modem.send_command_and_print_response(b"AT+HTTPTERM\r\n", "AT+HTTPTERM", delay);
    delay.delay_millis(5000);

    modem.send_command_and_print_response(b"AT+HTTPINIT\r\n", "AT+HTTPINIT", delay);
    delay.delay_millis(1000);

    modem.send_command_and_print_response(
        b"AT+HTTPPARA=\"URL\",\"http://rust-api.williamtekpeh.com/api/fuel-readings/batch\"\r\n",
        "AT+HTTPPARA URL",
        delay,
    );
    delay.delay_millis(1000);

    modem.send_command_and_print_response(
        b"AT+HTTPPARA=\"CONTENT\",\"application/json\"\r\n",
        "AT+HTTPPARA CONTENT",
        delay,
    );
    delay.delay_millis(1000);

    let mut data_command = heapless::String::<64>::new();

    let _ = core::fmt::write(
        &mut data_command,
        format_args!("AT+HTTPDATA={},10000\r\n", payload.len()),
    );

    modem.send_command_and_print_response(data_command.as_bytes(), "AT+HTTPDATA", delay);

    delay.delay_millis(1000);

    for byte in payload.as_bytes() {
        let _ = modem.uart.write(&[*byte]);
        delay.delay_millis(1);
    }

    println!("Sent HTTP JSON PAYLOAD");
    delay.delay_millis(5000);

    modem.send_command_and_print_response(b"AT+HTTPACTION=1\r\n", "AT+HTTPACTION POST", delay);

    delay.delay_millis(15000);

    modem.send_command_and_print_response(b"AT+HTTPREAD\r\n", "AT+HTTPREAD", delay);

    delay.delay_millis(1000);

    modem.send_command_and_print_response(b"AT+HTTPTERM\r\n", "AT+HTTPTERM", delay);
    delay.delay_millis(1000);
}
