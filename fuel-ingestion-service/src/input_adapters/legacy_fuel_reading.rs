use crate::domain::telemetry::models::{
    DiagnosticTelemetry, FuelTelemetry, PositionTelemetry, TelemetryReading,
};
use crate::models::FuelReading;

/// Converts the existing legacy ingestion model into the new
/// measurement-oriented telemetry domain model.
///
/// This allows the current simulator and ingestion API to continue
/// operating while the backend gradually adopts Telemetry V2.
impl From<&FuelReading> for TelemetryReading {
    fn from(reading: &FuelReading) -> Self {
        Self {
            device_id: reading.device_id.clone(),
            recorded_at: reading.timestamp,

            position: Some(PositionTelemetry {
                latitude: reading.latitude,
                longitude: reading.longitude,
                altitude: None,
                heading: None,
                speed_kmh: None,
                satellite_count: None,
                hdop: None,
            }),

            fuel: Some(FuelTelemetry {
                litres: reading.fuel_level_litres,
                percentage: reading.fuel_level_percentage,
                sensor_value: None,
                temperature: None,
            }),

            // The legacy FuelReading contains only a derived vibration level
            // and motion flag. Telemetry V2 expects raw accelerometer and
            // gyroscope measurements, so these values are not mapped into
            // ImuTelemetry.
            imu: None,

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

/// Converts a collection of legacy readings into Telemetry V2
/// domain readings.
pub fn map_legacy_readings(readings: &[FuelReading]) -> Vec<TelemetryReading> {
    readings.iter().map(TelemetryReading::from).collect()
}
