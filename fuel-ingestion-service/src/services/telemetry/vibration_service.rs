use anyhow::Result;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::telemetry::imu_interpreter::ImuInterpretation;
use crate::models::FuelReading;
use crate::repository::NewSensorReading;
use crate::services::telemetry::persistence::persist_sensor_reading;

/// Persists the physical vibration observation associated with one incoming
/// ORBI telemetry reading.
///
/// Motion Intelligence consumes the interpreted IMU measurement for
/// operational-state classification, while this service preserves the
/// continuous vibration observation in sensor_readings.
///
/// Keeping this persistence associated with the dedicated VIBRATION sensor
/// preserves the separation between FUEL, GPS, and VIBRATION capabilities.
pub async fn persist_vibration_reading(
    db_pool: &PgPool,
    device_id: Uuid,
    vibration_sensor_id: Uuid,
    reading: &FuelReading,
    imu_interpretation: &ImuInterpretation,
) -> Result<()> {
    let raw_payload: Value = serde_json::to_value(reading)?;

    persist_sensor_reading(
        db_pool,
        NewSensorReading {
            sensor_id: vibration_sensor_id,
            device_id,
            recorded_at: reading.timestamp,

            // The normalized vibration score is the primary numeric
            // observation for the VIBRATION sensor.
            value: imu_interpretation.vibration_score,
            unit: "vibration_score".to_string(),

            // Position belongs to the same physical telemetry observation and
            // is useful when reconstructing operational telemetry later.
            latitude: Some(reading.latitude),
            longitude: Some(reading.longitude),

            vibration_level: Some(imu_interpretation.vibration_score),
            motion_detected: Some(imu_interpretation.motion_detected),

            raw_payload,
        },
    )
    .await
}
