use esp_hal::delay::Delay;

use crate::modem::Modem;

pub fn run_http_diagnostic(modem: &mut Modem, delay: &Delay) {
    modem.send_command_and_print_response(b"AT+HTTPTERM\r\n", "AT+HTTPTERM", delay);
    delay.delay_millis(1000);

    modem.send_command_and_print_response(b"AT+HTTPINIT\r\n", "AT+HTTPINIT", delay);
    delay.delay_millis(1000);

    modem.send_command_and_print_response(
        b"AT+HTTPPARA=\"URL\",\"http://example.com\"\r\n",
        "AT+HTTPPARA URL",
        delay,
    );
    delay.delay_millis(1000);

    modem.send_command_and_print_response(b"AT+HTTPACTION=0\r\n", "AT+HTTPACTION GET", delay);
    delay.delay_millis(10000);

    modem.send_command_and_print_response(b"AT+HTTPREAD\r\n", "AT+HTTPREAD", delay);
    delay.delay_millis(1000);

    modem.send_command_and_print_response(b"AT+HTTPTERM\r\n", "AT+HTTPTERM", delay);
    delay.delay_millis(1000);
}
