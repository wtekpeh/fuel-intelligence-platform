use core::fmt::Write;
use heapless::String;

pub fn build_heartbeat_payload(device_id: &str) -> String<256> {
    let mut payload = String::<256>::new();

    write!(payload, "{{\"device_id\":\"{}\"}}", device_id)
        .expect("Heartbeat payload buffer is too small");

    payload
}
