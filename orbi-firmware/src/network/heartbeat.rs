use heapless::String;

pub fn build_heartbeat_payload(device_code: &str, timestamp: &str) -> String<256> {
    let mut payload = String::<256>::new();

    let _ = core::fmt::write(
        &mut payload,
        format_args!(
            "{{\
                \"device_id\":\"{}\",\
                \"timestamp\":\"{}\"\
            }}",
            device_code, timestamp,
        ),
    );

    payload
}
