use crate::domain::telemetry::models::{
    DiagnosticTelemetry, FuelTelemetry, ImuTelemetry, PositionTelemetry, TelemetryReading,
};
use crate::models::FuelReading;

/// Converts the existing ingestion model into the canonical,
/// measurement-oriented ORBI telemetry domain model.
///
/// This compatibility layer allows the current ingestion endpoint,
/// simulator, and physical firmware to continue using FuelReading
/// while backend services gradually migrate to Telemetry V2.
impl From<&FuelReading> for TelemetryReading {
    fn from(reading: &FuelReading) -> Self {
        Self {
            device_id: reading.device_id.clone(),
            recorded_at: reading.timestamp,

            position: Some(PositionTelemetry {
                latitude: reading.latitude,
                longitude: reading.longitude,
                altitude: None,
                heading: Some(reading.heading),
                speed_kmh: Some(reading.speed),
                satellite_count: None,
                hdop: None,
            }),

            fuel: Some(FuelTelemetry {
                litres: reading.fuel_level_litres,
                percentage: reading.fuel_level_percentage,
                sensor_value: None,
                temperature: None,
            }),

            imu: Some(ImuTelemetry {
                accel_x: reading.accel_x_g,
                accel_y: reading.accel_y_g,
                accel_z: reading.accel_z_g,

                gyro_x: reading.gyro_x_dps,
                gyro_y: reading.gyro_y_dps,
                gyro_z: reading.gyro_z_dps,

                temperature: Some(reading.imu_temperature_c),
            }),

            power: None,

            diagnostics: Some(DiagnosticTelemetry {
                firmware_version: None,
                signal_strength: None,
                queued_records: None,
                modem_temperature: None,
            }),
        }
    }
}

/// Converts a collection of legacy ingestion readings into canonical
/// Telemetry V2 domain readings.
pub fn map_legacy_readings(readings: &[FuelReading]) -> Vec<TelemetryReading> {
    readings.iter().map(TelemetryReading::from).collect()
}
