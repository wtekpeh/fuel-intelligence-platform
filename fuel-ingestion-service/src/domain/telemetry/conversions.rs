use crate::domain::telemetry::models::{
    CalibratedFuelTelemetry, DiagnosticTelemetry, FuelTelemetry, ImuTelemetry, PositionTelemetry,
    RawFuelTelemetry, TelemetryReading,
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

            fuel: match (
                reading.fuel_distance_smooth_cm,
                reading.fuel_distance_realtime_cm,
                reading.fuel_distance_raw_cm,
                reading.fuel_sensor_temperature_c,
                reading.fuel_sensor_status_1,
                reading.fuel_sensor_status_2,
                reading.fuel_raw_data_validity,
            ) {
                (
                    Some(smooth_distance_cm),
                    Some(realtime_distance_cm),
                    Some(raw_distance_cm),
                    Some(temperature_c),
                    Some(status_byte_1),
                    Some(status_byte_2),
                    Some(raw_data_validity),
                ) => Some(FuelTelemetry {
                    raw: RawFuelTelemetry {
                        smooth_distance_cm,
                        realtime_distance_cm,
                        raw_distance_cm,
                        temperature_c,
                        status_byte_1,
                        status_byte_2,
                        raw_data_validity,
                    },

                    calibrated: match (reading.fuel_level_litres, reading.fuel_level_percentage) {
                        (Some(litres), Some(percentage)) => {
                            Some(CalibratedFuelTelemetry { litres, percentage })
                        }

                        _ => None,
                    },
                }),

                _ => None,
            },

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
