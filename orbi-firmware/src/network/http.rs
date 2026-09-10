use embassy_time::{Duration, Timer};
use esp_hal::delay::Delay;
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

fn contains_bytes(buffer: &[u8], pattern: &[u8]) -> bool {
    if pattern.is_empty() || pattern.len() > buffer.len() {
        return false;
    }

    buffer
        .windows(pattern.len())
        .any(|window| window == pattern)
}

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

async fn collect_http_action_response(
    modem: &mut Modem<'_>,
    action_label: &str,
) -> Option<([u8; HTTP_ACTION_BUFFER_SIZE], usize)> {
    if !modem.send_command(b"AT+HTTPACTION=1\r\n", action_label) {
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
    delay: &Delay,
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

        modem.send_command_and_print_response(b"AT+HTTPTERM\r\n", "AT+HTTPTERM", delay);

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

    let action_response = collect_http_action_response(modem, action_label).await;

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

pub async fn send_payload<const N: usize>(
    modem: &mut Modem<'_>,
    delay: &Delay,
    payload: &heapless::String<N>,
) -> bool {
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
    )
    .await
}

pub async fn send_heartbeat(
    modem: &mut Modem<'_>,
    delay: &Delay,
    payload: &heapless::String<256>,
) -> bool {
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
    )
    .await
}
