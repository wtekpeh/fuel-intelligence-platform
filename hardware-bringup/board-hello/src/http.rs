use esp_hal::delay::Delay;
use esp_println::println;

use crate::modem::Modem;

fn post_json<const N: usize>(
    modem: &mut Modem,
    delay: &Delay,
    url_command: &[u8],
    url_label: &str,
    payload: &heapless::String<N>,
    data_label: &str,
    action_label: &str,
    read_label: &str,
    sent_message: &str,
    final_httpterm_delay: u32,
) {
    modem.send_command_and_print_response(b"AT+HTTPTERM\r\n", "AT+HTTPTERM", delay);
    delay.delay_millis(1000);

    modem.send_command_and_print_response(b"AT+HTTPINIT\r\n", "AT+HTTPINIT", delay);
    delay.delay_millis(1000);

    modem.send_command_and_print_response(url_command, url_label, delay);
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

    modem.send_command_and_print_response(data_command.as_bytes(), data_label, delay);

    delay.delay_millis(1000);

    for byte in payload.as_bytes() {
        let _ = modem.uart.write(&[*byte]);
        delay.delay_millis(1);
    }

    println!("{}", sent_message);

    delay.delay_millis(final_httpterm_delay);

    modem.send_command_and_print_response(b"AT+HTTPACTION=1\r\n", action_label, delay);

    delay.delay_millis(15000);

    modem.send_command_and_print_response(b"AT+HTTPREAD\r\n", read_label, delay);

    delay.delay_millis(1000);

    modem.send_command_and_print_response(b"AT+HTTPTERM\r\n", "AT+HTTPTERM", delay);

    delay.delay_millis(1000);
}

pub fn send_payload(modem: &mut Modem, delay: &Delay, payload: &heapless::String<1024>) {
    post_json(
        modem,
        delay,
        b"AT+HTTPPARA=\"URL\",\"http://rust-api.williamtekpeh.com/api/fuel-readings/batch\"\r\n",
        "AT+HTTPPARA URL",
        payload,
        "AT+HTTPDATA",
        "AT+HTTPACTION POST",
        "AT+HTTPREAD",
        "Sent HTTP JSON PAYLOAD",
        5000,
    );
}

pub fn send_heartbeat(modem: &mut Modem, delay: &Delay, payload: &heapless::String<256>) {
    println!("========================");
    println!("SENDING ORBI HEARTBEAT");
    println!("========================");
    println!("{}", payload);

    post_json(
        modem,
        delay,
        b"AT+HTTPPARA=\"URL\",\"http://rust-api.williamtekpeh.com/api/heartbeat\"\r\n",
        "AT+HTTPPARA HEARTBEAT URL",
        payload,
        "AT+HTTPDATA HEARTBEAT",
        "AT+HTTPACTION HEARTBEAT POST",
        "AT+HTTPREAD HEARTBEAT",
        "Heartbeat JSON sent to modem.",
        3000,
    );
}
