use embassy_time::{Duration, Timer};
use esp_println::{print, println};

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

/// Metadata returned by the A7670 after an HTTP transaction.
///
/// A typical modem response is:
///
/// +HTTPACTION: 0,200,25
///
/// where:
///
/// - 0   = HTTP method used by the modem
/// - 200 = HTTP status code
/// - 25  = number of response-body bytes available to read
struct HttpActionResult {
    status: u16,
    body_length: usize,
}

fn extract_http_action_result(response: &[u8]) -> Option<HttpActionResult> {
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
     * Skip the HTTP method field.
     *
     * Examples:
     *
     * 0 = GET
     * 1 = POST
     */
    while response.get(index) != Some(&b',') {
        response.get(index)?;

        index += 1;
    }

    /*
     * Move past the comma separating method and status.
     */
    index += 1;

    /*
     * Parse the HTTP status field.
     *
     * HTTP status codes are three digits, for example:
     *
     * 200
     * 404
     * 500
     */
    let status_bytes = response.get(index..index + 3)?;

    if !status_bytes.iter().all(u8::is_ascii_digit) {
        return None;
    }

    let status = ((status_bytes[0] - b'0') as u16 * 100)
        + ((status_bytes[1] - b'0') as u16 * 10)
        + (status_bytes[2] - b'0') as u16;

    index += 3;

    /*
     * The status field must be followed by the comma that begins
     * the response-body length field.
     */
    if response.get(index) != Some(&b',') {
        return None;
    }

    index += 1;

    /*
     * Parse the body length.
     *
     * Unlike the status code, the length is variable-width:
     *
     * 0
     * 25
     * 73
     * 1024
     *
     * Stop as soon as we reach a non-digit such as '\r' or '\n'.
     */
    let mut body_length = 0usize;
    let mut body_length_digits = 0usize;

    while let Some(byte) = response.get(index) {
        if !byte.is_ascii_digit() {
            break;
        }

        body_length = body_length
            .checked_mul(10)?
            .checked_add((byte - b'0') as usize)?;

        body_length_digits += 1;
        index += 1;
    }

    /*
     * A missing length is malformed rather than equivalent to zero.
     *
     * "+HTTPACTION: 0,200," must therefore fail parsing.
     */
    if body_length_digits == 0 {
        return None;
    }

    Some(HttpActionResult {
        status,
        body_length,
    })
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

            /*
             * Temporary diagnostic:
             *
             * Show the complete modem response so we can inspect the
             * +HTTPACTION method, HTTP status and response-body length.
             */
            println!("RAW HTTPACTION RESPONSE:");

            for byte in combined_response.iter().take(total_bytes_read) {
                if *byte >= 32 && *byte <= 126 {
                    print!("{}", *byte as char);
                } else if *byte == b'\r' {
                    print!("\\r");
                } else if *byte == b'\n' {
                    print!("\\n");
                } else {
                    print!("[{}]", *byte);
                }
            }

            println!();

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

        match extract_http_action_result(response) {
            Some(action_result) => {
                println!("HTTP status: {}", action_result.status);

                if (200..300).contains(&action_result.status) {
                    println!("HTTP upload succeeded.");

                    true
                } else {
                    println!("HTTP upload failed.");

                    false
                }
            }

            None => {
                println!("Could not parse HTTPACTION result.");

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

    /*
     * A successful runtime-state GET must preserve the response-body
     * length reported by the modem.
     *
     * For example:
     *
     * +HTTPACTION: 0,200,25
     *
     * means the request succeeded and 25 response bytes are available
     * for AT+HTTPREAD.
     */
    let response_body_length = match action_response {
        Some((response_buffer, bytes_read)) => {
            match extract_http_action_result(&response_buffer[..bytes_read]) {
                Some(action_result) if (200..300).contains(&action_result.status) => {
                    println!("Runtime-state HTTP status: {}", action_result.status);
                    println!(
                        "Runtime-state response body length: {} byte(s).",
                        action_result.body_length
                    );

                    Some(action_result.body_length)
                }

                Some(action_result) => {
                    println!(
                        "Runtime-state HTTP request failed with status: {}",
                        action_result.status
                    );

                    None
                }

                None => {
                    println!("Could not parse runtime-state HTTPACTION result.");

                    None
                }
            }
        }

        None => {
            println!("No runtime-state HTTPACTION response received.");

            None
        }
    };

    let Some(response_body_length) = response_body_length else {
        modem
            .send_command_and_print_response_async(b"AT+HTTPTERM\r\n", "AT+HTTPTERM")
            .await;

        return None;
    };

    /*
     * Ask the modem for exactly the number of response-body bytes
     * reported by +HTTPACTION.
     *
     * Example:
     *
     * +HTTPACTION: 0,200,25
     *
     * becomes:
     *
     * AT+HTTPREAD=0,25
     *
     * Do not hard-code the body length because the backend runtime-state
     * response may grow as additional runtime controls are introduced.
     */
    let mut read_command = heapless::String::<32>::new();

    if core::fmt::write(
        &mut read_command,
        format_args!("AT+HTTPREAD=0,{}\r\n", response_body_length),
    )
    .is_err()
    {
        println!("Failed to build runtime-state HTTPREAD command.");

        modem
            .send_command_and_print_response_async(b"AT+HTTPTERM\r\n", "AT+HTTPTERM")
            .await;

        return None;
    }

    println!(
        "Runtime-state response body length: {} byte(s).",
        response_body_length
    );

    let body_response = modem
        .send_command_and_collect_fragmented_response_async(
            read_command.as_bytes(),
            "AT+HTTPREAD RUNTIME STATE",
        )
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
