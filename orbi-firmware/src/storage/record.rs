use heapless::String;

use crate::telemetry::record::TelemetryRecord;

pub fn format_telemetry_record(record: &TelemetryRecord<'_>) -> String<512> {
    let mut line = String::<512>::new();

    let _ = core::fmt::write(
        &mut line,
        format_args!(
            "{{\
                \"device_id\":\"{}\",\
                \"timestamp\":\"{}\",\
                \"latitude\":{},\
                \"longitude\":{},\
                \"fuel_level_litres\":{},\
                \"fuel_level_percentage\":{},\
                \"vibration_level\":{},\
                \"motion_detected\":{},\
                \"speed\":{},\
                \"heading\":{},\
                \"simulation_mode\":\"{}\"\
            }}\r\n",
            record.device_id,
            record.timestamp,
            record.latitude,
            record.longitude,
            record.fuel_level_litres,
            record.fuel_level_percentage,
            record.vibration_level,
            record.motion_detected,
            record.speed,
            record.heading,
            record.simulation_mode,
        ),
    );

    line
}
