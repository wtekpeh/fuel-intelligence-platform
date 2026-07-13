use heapless::String;

use crate::device::DEVICE_IDENTITY;

pub fn build_heartbeat_payload(timestamp: &str) -> String<256> {
    let mut payload = String::<256>::new();

    let _ = core::fmt::write(
        &mut payload,
        format_args!(
            "{{\
                \"device_id\":\"{}\",\
                \"timestamp\":\"{}\"\
            }}",
            DEVICE_IDENTITY.device_code, timestamp,
        ),
    );

    payload
}
