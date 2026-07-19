use heapless::String;

use crate::scheduler::reporting::MotionState;
use crate::telemetry::record::TelemetryRecord;

pub struct GnssDiagnosticRecord<'a> {
    pub timestamp: &'a str,

    pub latitude: f64,
    pub longitude: f64,

    pub speed_knots: f64,
    pub speed_kmh: f64,

    pub heading_degrees: f64,

    pub motion_state: MotionState,
    pub reporting_interval_seconds: u32,
}

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

pub fn format_gnss_diagnostic_record(record: &GnssDiagnosticRecord<'_>) -> String<384> {
    let mut line = String::<384>::new();

    let _ = core::fmt::write(
        &mut line,
        format_args!(
            "{{\
                \"timestamp\":\"{}\",\
                \"latitude\":{},\
                \"longitude\":{},\
                \"speed_knots\":{},\
                \"speed_kmh\":{},\
                \"heading_degrees\":{},\
                \"motion_state\":\"{:?}\",\
                \"reporting_interval_seconds\":{}\
            }}\r\n",
            record.timestamp,
            record.latitude,
            record.longitude,
            record.speed_knots,
            record.speed_kmh,
            record.heading_degrees,
            record.motion_state,
            record.reporting_interval_seconds,
        ),
    );

    line
}
