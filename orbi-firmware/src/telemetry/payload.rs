use crate::telemetry::record::TelemetryRecord;
use heapless::String;

pub fn build_gps_only_payload(reading: &TelemetryRecord<'_>) -> String<1024> {
    let mut payload = String::<1024>::new();

    let _ = core::fmt::write(
        &mut payload,
        format_args!(
            "{{\
        \"device_id\":\"{}\",\
        \"synced_at\":\"{}\",\
        \"readings\":[{{\
            \"device_id\":\"{}\",\
            \"timestamp\":\"{}\",\
            \"fuel_level_litres\":{},\
            \"fuel_level_percentage\":{},\
            \"latitude\":{},\
            \"longitude\":{},\
            \"vibration_level\":{},\
            \"motion_detected\":{},\
            \"simulation_mode\":\"{}\"\
        }}]\
    }}",
            reading.device_id,
            reading.timestamp,
            reading.device_id,
            reading.timestamp,
            reading.fuel_level_litres,
            reading.fuel_level_percentage,
            reading.latitude,
            reading.longitude,
            reading.vibration_level,
            reading.motion_detected,
            reading.simulation_mode,
        ),
    );

    payload
}

fn extract_json_string<'a>(json: &'a str, field_name: &str) -> Option<&'a str> {
    let mut pattern = String::<64>::new();

    let _ = core::fmt::write(&mut pattern, format_args!("\"{}\":\"", field_name));

    let field_start = json.find(pattern.as_str())?;
    let value_start = field_start + pattern.len();
    let remaining = json.get(value_start..)?;
    let value_end = remaining.find('"')?;

    remaining.get(..value_end)
}

pub fn extract_replay_identity<'a>(queued_record: &'a str) -> Option<(&'a str, &'a str)> {
    let device_id = extract_json_string(queued_record, "device_id")?;

    let timestamp = extract_json_string(queued_record, "timestamp")?;

    Some((device_id, timestamp))
}

pub fn build_replay_batch_payload(queued_record: &str) -> Option<String<1024>> {
    let device_id = extract_json_string(queued_record, "device_id")?;

    let timestamp = extract_json_string(queued_record, "timestamp")?;

    let mut payload = String::<1024>::new();

    core::fmt::write(
        &mut payload,
        format_args!(
            "{{\
                \"device_id\":\"{}\",\
                \"synced_at\":\"{}\",\
                \"readings\":[{}]\
            }}",
            device_id, timestamp, queued_record,
        ),
    )
    .ok()?;

    Some(payload)
}
