use embassy_time::{Duration, Timer};
use esp_println::println;

use crate::drivers::Modem;

/*
 * HTTPACTION is asynchronous.
 *
 * Poll every 250 ms and stop as soon as the modem returns
 * the +HTTPACTION result.
 */
const HTTP_ACTION_POLL_INTERVAL_MS: u32 = 250;
const HTTP_ACTION_MAX_POLLS: usize = 60;
const HTTP_ACTION_BUFFER_SIZE: usize = 512;

/*
 * Small settling delays remain for modem reliability.
 *
 * The previous implementation waited one second after almost every
 * command and five seconds before checking HTTPACTION. Those fixed
 * delays made each telemetry upload unnecessarily slow.
 */
const COMMAND_SETTLE_DELAY_MS: u32 = 200;
const PAYLOAD_SETTLE_DELAY_MS: u32 = 250;
const CLEANUP_SETTLE_DELAY_MS: u32 = 200;

const RUNTIME_STATE_URL_COMMAND_CAPACITY: usize = 192;

fn contains_bytes(buffer: &[u8], pattern: &[u8]) -> bool {
    if pattern.is_empty() || pattern.len() > buffer.len() {
        return false;
    }

    buffer
        .windows(pattern.len())
        .any(|window| window == pattern)
}

fn extract_http_status(response: &[u8]) -> Option<u16> {
    /*
     * A7670 HTTPACTION result format:
     *
     * +HTTPACTION: <method>,<status>,<length>
     *
     * Examples:
     *
     * GET:
     * +HTTPACTION: 0,200,25
     *
     * POST:
     * +HTTPACTION: 1,200,42
     *
     * The previous implementation searched specifically for
     * "+HTTPACTION: 1,", which meant only POST responses could
     * be parsed. Runtime-state retrieval uses GET (method 0),
     * so status parsing must not depend on the HTTP method.
     */
    const PREFIX: &[u8] = b"+HTTPACTION:";

    let prefix_start = response
        .windows(PREFIX.len())
        .position(|window| window == PREFIX)?;

    let mut index = prefix_start + PREFIX.len();

    /*
     * Skip optional spaces after "+HTTPACTION:".
     */
    while response.get(index) == Some(&b' ') {
        index += 1;
    }

    /*
     * Skip the HTTP method field until its comma.
     *
     * Currently:
     *
     * 0 = GET
     * 1 = POST
     *
     * We deliberately do not care which method produced the
     * response because this function's only responsibility is
     * extracting the HTTP status code.
     */
    while let Some(byte) = response.get(index) {
        if *byte == b',' {
            index += 1;
            break;
        }

        index += 1;
    }

    /*
     * The next three bytes must be the HTTP status code.
     */
    let status_bytes = response.get(index..index + 3)?;

    if !status_bytes.iter().all(u8::is_ascii_digit) {
        return None;
    }

    let status = ((status_bytes[0] - b'0') as u16 * 100)
        + ((status_bytes[1] - b'0') as u16 * 10)
        + (status_bytes[2] - b'0') as u16;

    Some(status)
}

fn build_runtime_state_url_command(
    device_code: &str,
) -> Option<heapless::String<RUNTIME_STATE_URL_COMMAND_CAPACITY>> {
    let mut command = heapless::String::<RUNTIME_STATE_URL_COMMAND_CAPACITY>::new();

    if core::fmt::write(
        &mut command,
        format_args!(
            "AT+HTTPPARA=\"URL\",\"http://rust-api.williamtekpeh.com/api/devices/{}/runtime-state\"\r\n",
            device_code
        ),
    )
    .is_err()
    {
        println!("Failed to build runtime-state URL command.");

        return None;
    }

    Some(command)
}

fn parse_calibration_mode(response: &[u8]) -> Option<bool> {
    /*
     * Runtime-state responses currently contain one backend-controlled
     * boolean:
     *
     * {"calibration_mode":true}
     *
     * Keep parsing deliberately narrow. If the expected field cannot be
     * found, return None rather than guessing a device mode.
     */
    if contains_bytes(response, b"\"calibration_mode\":true") {
        return Some(true);
    }

    if contains_bytes(response, b"\"calibration_mode\":false") {
        return Some(false);
    }

    None
}

async fn collect_http_action_response(
    modem: &mut Modem<'_>,
    action_command: &[u8],
    action_label: &str,
) -> Option<([u8; HTTP_ACTION_BUFFER_SIZE], usize)> {
    if !modem.send_command(action_command, action_label) {
        println!("Failed to send HTTPACTION command.");

        return None;
    }

    let mut combined_response = [0u8; HTTP_ACTION_BUFFER_SIZE];
    let mut total_bytes_read = 0usize;

    for poll_number in 1..=HTTP_ACTION_MAX_POLLS {
        Timer::after(Duration::from_millis(HTTP_ACTION_POLL_INTERVAL_MS as u64)).await;

        let Some((response_buffer, bytes_read)) = modem.read_response() else {
            continue;
        };

        if bytes_read == 0 {
            continue;
        }

        let remaining_capacity = HTTP_ACTION_BUFFER_SIZE - total_bytes_read;

        if remaining_capacity == 0 {
            println!("HTTPACTION response buffer is full.");

            break;
        }

        let bytes_to_copy = core::cmp::min(bytes_read, remaining_capacity);

        combined_response[total_bytes_read..total_bytes_read + bytes_to_copy]
            .copy_from_slice(&response_buffer[..bytes_to_copy]);

        total_bytes_read += bytes_to_copy;

        if contains_bytes(&combined_response[..total_bytes_read], b"+HTTPACTION:") {
            println!(
                "HTTPACTION completed after {} poll(s), approximately {} ms.",
                poll_number,
                poll_number * HTTP_ACTION_POLL_INTERVAL_MS as usize
            );

            return Some((combined_response, total_bytes_read));
        }
    }

    if total_bytes_read > 0 {
        println!(
            "HTTPACTION timed out after receiving {} byte(s).",
            total_bytes_read
        );

        Some((combined_response, total_bytes_read))
    } else {
        println!("HTTPACTION timed out without receiving a response.");

        None
    }
}

async fn post_json<const N: usize>(
    modem: &mut Modem<'_>,
    url_command: &[u8],
    url_label: &str,
    payload: &heapless::String<N>,
    data_label: &str,
    action_label: &str,
    read_label: &str,
    sent_message: &str,
) -> bool {
    /*
     * Ensure a stale HTTP session does not interfere with the
     * new transaction.
     */
    modem
        .send_command_and_print_response_async(b"AT+HTTPTERM\r\n", "AT+HTTPTERM")
        .await;

    Timer::after(Duration::from_millis(COMMAND_SETTLE_DELAY_MS as u64)).await;

    modem
        .send_command_and_print_response_async(b"AT+HTTPINIT\r\n", "AT+HTTPINIT")
        .await;

    Timer::after(Duration::from_millis(COMMAND_SETTLE_DELAY_MS as u64)).await;

    modem
        .send_command_and_print_response_async(url_command, url_label)
        .await;

    Timer::after(Duration::from_millis(COMMAND_SETTLE_DELAY_MS as u64)).await;

    modem
        .send_command_and_print_response_async(
            b"AT+HTTPPARA=\"CONTENT\",\"application/json\"\r\n",
            "AT+HTTPPARA CONTENT",
        )
        .await;

    Timer::after(Duration::from_millis(COMMAND_SETTLE_DELAY_MS as u64)).await;

    let mut data_command = heapless::String::<64>::new();

    if core::fmt::write(
        &mut data_command,
        format_args!("AT+HTTPDATA={},10000\r\n", payload.len()),
    )
    .is_err()
    {
        println!("Failed to build AT+HTTPDATA command.");

        modem
            .send_command_and_print_response_async(b"AT+HTTPTERM\r\n", "AT+HTTPTERM")
            .await;

        return false;
    }

    modem
        .send_command_and_print_response_async(data_command.as_bytes(), data_label)
        .await;

    Timer::after(Duration::from_millis(COMMAND_SETTLE_DELAY_MS as u64)).await;

    /*
     * Keep the existing paced UART payload transmission for now.
     *
     * Removing this delay as well could overrun the UART depending on
     * how Modem::uart.write() is implemented. It adds only about one
     * millisecond per payload byte and can be optimized separately
     * after the HTTP timing has been verified.
     */
    for byte in payload.as_bytes() {
        if modem.uart.write(&[*byte]).is_err() {
            println!("Failed while writing HTTP payload to modem.");

            modem
                .send_command_and_print_response_async(b"AT+HTTPTERM\r\n", "AT+HTTPTERM")
                .await;

            return false;
        }

        /*
         * Preserve the existing one-millisecond UART pacing,
         * but make that wait cooperative so Embassy can schedule
         * other ready tasks while the payload is being transmitted.
         */
        Timer::after(Duration::from_millis(1)).await;
    }

    println!("{}", sent_message);

    /*
     * The previous implementation waited three or five seconds here.
     *
     * HTTPACTION already has a response-driven polling loop, so only
     * a short settling delay is required before polling begins.
     */

    Timer::after(Duration::from_millis(PAYLOAD_SETTLE_DELAY_MS as u64)).await;

    let action_response =
        collect_http_action_response(modem, b"AT+HTTPACTION=1\r\n", action_label).await;

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

    Timer::after(Duration::from_millis(CLEANUP_SETTLE_DELAY_MS as u64)).await;

    modem
        .send_command_and_print_response_async(b"AT+HTTPREAD\r\n", read_label)
        .await;

    Timer::after(Duration::from_millis(CLEANUP_SETTLE_DELAY_MS as u64)).await;

    modem
        .send_command_and_print_response_async(b"AT+HTTPTERM\r\n", "AT+HTTPTERM")
        .await;

    Timer::after(Duration::from_millis(CLEANUP_SETTLE_DELAY_MS as u64)).await;

    upload_succeeded
}

pub async fn get_runtime_calibration_mode(
    modem: &mut Modem<'_>,
    device_code: &str,
) -> Option<bool> {
    println!("========================");
    println!("GETTING ORBI RUNTIME STATE");
    println!("========================");

    let url_command = build_runtime_state_url_command(device_code)?;

    /*
     * Start from a clean HTTP session.
     */
    modem
        .send_command_and_print_response_async(b"AT+HTTPTERM\r\n", "AT+HTTPTERM")
        .await;

    Timer::after(Duration::from_millis(COMMAND_SETTLE_DELAY_MS as u64)).await;

    modem
        .send_command_and_print_response_async(b"AT+HTTPINIT\r\n", "AT+HTTPINIT")
        .await;

    Timer::after(Duration::from_millis(COMMAND_SETTLE_DELAY_MS as u64)).await;

    modem
        .send_command_and_print_response_async(
            url_command.as_bytes(),
            "AT+HTTPPARA RUNTIME STATE URL",
        )
        .await;

    Timer::after(Duration::from_millis(COMMAND_SETTLE_DELAY_MS as u64)).await;

    /*
     * HTTPACTION=0 performs an HTTP GET.
     */
    let action_response = collect_http_action_response(
        modem,
        b"AT+HTTPACTION=0\r\n",
        "AT+HTTPACTION RUNTIME STATE GET",
    )
    .await;

    let request_succeeded = match action_response {
        Some((response_buffer, bytes_read)) => {
            match extract_http_status(&response_buffer[..bytes_read]) {
                Some(status) if (200..300).contains(&status) => {
                    println!("Runtime-state HTTP status: {}", status);
                    true
                }

                Some(status) => {
                    println!("Runtime-state HTTP request failed with status: {}", status);
                    false
                }

                None => {
                    println!("Could not parse runtime-state HTTP status.");
                    false
                }
            }
        }

        None => {
            println!("No runtime-state HTTPACTION response received.");
            false
        }
    };

    if !request_succeeded {
        modem
            .send_command_and_print_response_async(b"AT+HTTPTERM\r\n", "AT+HTTPTERM")
            .await;

        return None;
    }

    /*
     * Request the HTTP response body.
     *
     * Unlike telemetry POSTs, this body is meaningful to the firmware:
     *
     * {"calibration_mode":true}
     */
    let body_response = modem
        .send_command_and_collect_response_async(b"AT+HTTPREAD\r\n", "AT+HTTPREAD RUNTIME STATE")
        .await;

    modem
        .send_command_and_print_response_async(b"AT+HTTPTERM\r\n", "AT+HTTPTERM")
        .await;

    let Some((response_buffer, bytes_read)) = body_response else {
        println!("Runtime-state response body was not received.");
        return None;
    };

    let calibration_mode = parse_calibration_mode(&response_buffer[..bytes_read]);

    match calibration_mode {
        Some(true) => {
            println!("Backend runtime state: calibration mode ON.");
        }

        Some(false) => {
            println!("Backend runtime state: calibration mode OFF.");
        }

        None => {
            println!("Could not parse calibration mode from runtime-state response.");
        }
    }

    calibration_mode
}

pub async fn send_payload<const N: usize>(
    modem: &mut Modem<'_>,
    payload: &heapless::String<N>,
) -> bool {
    post_json(
        modem,
        b"AT+HTTPPARA=\"URL\",\"http://rust-api.williamtekpeh.com/api/fuel-readings/batch\"\r\n",
        "AT+HTTPPARA URL",
        payload,
        "AT+HTTPDATA",
        "AT+HTTPACTION POST",
        "AT+HTTPREAD",
        "Sent HTTP JSON PAYLOAD",
    )
    .await
}

pub async fn send_heartbeat(modem: &mut Modem<'_>, payload: &heapless::String<256>) -> bool {
    println!("========================");
    println!("SENDING ORBI HEARTBEAT");
    println!("========================");
    println!("{}", payload);

    post_json(
        modem,
        b"AT+HTTPPARA=\"URL\",\"http://rust-api.williamtekpeh.com/api/heartbeat\"\r\n",
        "AT+HTTPPARA HEARTBEAT URL",
        payload,
        "AT+HTTPDATA HEARTBEAT",
        "AT+HTTPACTION HEARTBEAT POST",
        "AT+HTTPREAD HEARTBEAT",
        "Heartbeat JSON sent to modem.",
    )
    .await
}
