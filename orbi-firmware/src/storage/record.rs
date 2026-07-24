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

pub fn format_telemetry_record(record: &TelemetryRecord<'_>) -> String<768> {
    let mut line = String::<768>::new();

    let _ = core::fmt::write(
        &mut line,
        format_args!(
            "{{\
                \"device_id\":\"{}\",\
                \"timestamp\":\"{}\",\
                \"latitude\":{},\
                \"longitude\":{},\
                \"speed\":{},\
                \"heading\":{},\
                \"fuel_level_litres\":{},\
                \"fuel_level_percentage\":{},\
                \"accel_x_g\":{},\
                \"accel_y_g\":{},\
                \"accel_z_g\":{},\
                \"gyro_x_dps\":{},\
                \"gyro_y_dps\":{},\
                \"gyro_z_dps\":{},\
                \"imu_temperature_c\":{},\
                \"simulation_mode\":\"{}\"\
            }}\r\n",
            record.device_id,
            record.timestamp,
            record.latitude,
            record.longitude,
            record.speed,
            record.heading,
            record.fuel_level_litres,
            record.fuel_level_percentage,
            record.accel_x_g,
            record.accel_y_g,
            record.accel_z_g,
            record.gyro_x_dps,
            record.gyro_y_dps,
            record.gyro_z_dps,
            record.imu_temperature_c,
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
