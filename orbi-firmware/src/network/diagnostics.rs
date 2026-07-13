use esp_hal::delay::Delay;

use crate::drivers::Modem;

pub fn run_network_diagnostics(modem: &mut Modem, delay: &Delay) {
    modem.send_command_and_print_response(b"AT\r\n", "AT", delay);
    delay.delay_millis(1000);

    modem.send_command_and_print_response(b"AT+CPIN?\r\n", "AT+CPIN?", delay);
    delay.delay_millis(1000);

    modem.send_command_and_print_response(b"AT+CSQ\r\n", "AT+CSQ", delay);
    delay.delay_millis(1000);

    modem.send_command_and_print_response(b"AT+CEREG?\r\n", "AT+CEREG?", delay);
    delay.delay_millis(1000);

    modem.send_command_and_print_response(b"AT+CGATT?\r\n", "AT+CGATT?", delay);
    delay.delay_millis(1000);

    modem.send_command_and_print_response(b"AT+CGPADDR\r\n", "AT+CGPADDR", delay);
    delay.delay_millis(1000);
}
