use esp_hal::delay::Delay;
use esp_println::println;

use crate::drivers::Modem;

#[derive(Debug)]
pub struct NetworkState {
    pub sim_ready: bool,
    pub registered: bool,
    pub attached: bool,
    pub has_ip: bool,
}

impl NetworkState {
    pub const fn is_ready(&self) -> bool {
        self.sim_ready && self.registered && self.attached && self.has_ip
    }
}

fn response_contains(response: &[u8], expected: &[u8]) -> bool {
    response
        .windows(expected.len())
        .any(|window| window == expected)
}

fn has_assigned_ip(response: &[u8]) -> bool {
    const PREFIX: &[u8] = b"+CGPADDR: 1,";

    let Some(prefix_start) = response
        .windows(PREFIX.len())
        .position(|window| window == PREFIX)
    else {
        return false;
    };

    let value_start = prefix_start + PREFIX.len();
    let Some(address_bytes) = response.get(value_start..) else {
        return false;
    };

    let address_end = address_bytes
        .iter()
        .position(|byte| *byte == b'\r' || *byte == b'\n')
        .unwrap_or(address_bytes.len());

    let address = &address_bytes[..address_end];

    !address.is_empty() && address.iter().any(|byte| *byte == b'.') && address != b"0.0.0.0"
}

fn collect_command(
    modem: &mut Modem,
    delay: &Delay,
    command: &[u8],
    label: &str,
) -> Option<([u8; 256], usize)> {
    modem.send_command_and_collect_response(command, label, delay)
}

pub fn read_network_state(modem: &mut Modem, delay: &Delay) -> NetworkState {
    println!("========================");
    println!("ORBI NETWORK STATE");
    println!("========================");

    let sim_ready = collect_command(modem, delay, b"AT+CPIN?\r\n", "AT+CPIN?")
        .map(|(buffer, bytes_read)| response_contains(&buffer[..bytes_read], b"+CPIN: READY"))
        .unwrap_or(false);

    delay.delay_millis(1000);

    let registered = collect_command(modem, delay, b"AT+CEREG?\r\n", "AT+CEREG?")
        .map(|(buffer, bytes_read)| {
            let response = &buffer[..bytes_read];

            response_contains(response, b"+CEREG: 0,1")
                || response_contains(response, b"+CEREG: 0,5")
                || response_contains(response, b"+CEREG: 1,1")
                || response_contains(response, b"+CEREG: 1,5")
        })
        .unwrap_or(false);

    delay.delay_millis(1000);

    let attached = collect_command(modem, delay, b"AT+CGATT?\r\n", "AT+CGATT?")
        .map(|(buffer, bytes_read)| response_contains(&buffer[..bytes_read], b"+CGATT: 1"))
        .unwrap_or(false);

    delay.delay_millis(1000);

    let has_ip = collect_command(modem, delay, b"AT+CGPADDR\r\n", "AT+CGPADDR")
        .map(|(buffer, bytes_read)| has_assigned_ip(&buffer[..bytes_read]))
        .unwrap_or(false);

    let state = NetworkState {
        sim_ready,
        registered,
        attached,
        has_ip,
    };

    println!("SIM ready: {}", state.sim_ready);
    println!("LTE registered: {}", state.registered);
    println!("Packet data attached: {}", state.attached);
    println!("IP assigned: {}", state.has_ip);
    println!("Network ready: {}", state.is_ready());

    state
}
