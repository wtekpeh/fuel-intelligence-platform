use esp_hal::delay::Delay;
use esp_println::println;

use crate::drivers::Modem;

fn extract_http_status(response: &[u8]) -> Option<u16> {
    const PREFIX: &[u8] = b"+HTTPACTION: 1,";

    let prefix_start = response
        .windows(PREFIX.len())
        .position(|window| window == PREFIX)?;

    let status_start = prefix_start + PREFIX.len();
    let status_bytes = response.get(status_start..status_start + 3)?;

    if !status_bytes.iter().all(u8::is_ascii_digit) {
        return None;
    }

    let status = ((status_bytes[0] - b'0') as u16 * 100)
        + ((status_bytes[1] - b'0') as u16 * 10)
        + (status_bytes[2] - b'0') as u16;

    Some(status)
}

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
) -> bool {
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

    let action_response =
        modem.send_command_and_collect_response(b"AT+HTTPACTION=1\r\n", action_label, delay);

    let upload_succeeded = if let Some((response_buffer, bytes_read)) = action_response {
        let response = &response_buffer[..bytes_read];

        match extract_http_status(response) {
            Some(status) => {
                println!("HTTP status: {}", status);

                if (200..300).contains(&status) {
                    println!("HTTP upload succeeded.");
                    true
                } else {
                    println!("HTTP upload failed.");
                    false
                }
            }

            None => {
                println!("Could not parse HTTPACTION status.");
                false
            }
        }
    } else {
        println!("No HTTPACTION response received.");
        false
    };

    delay.delay_millis(1000);

    modem.send_command_and_print_response(b"AT+HTTPREAD\r\n", read_label, delay);

    delay.delay_millis(1000);

    modem.send_command_and_print_response(b"AT+HTTPTERM\r\n", "AT+HTTPTERM", delay);

    delay.delay_millis(1000);

    upload_succeeded
}

pub fn send_payload(modem: &mut Modem, delay: &Delay, payload: &heapless::String<1024>) -> bool {
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
    )
}

pub fn send_heartbeat(modem: &mut Modem, delay: &Delay, payload: &heapless::String<256>) -> bool {
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
    )
}
