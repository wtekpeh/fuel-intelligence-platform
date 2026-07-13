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
