use anyhow::Result;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::FuelReading;
use crate::repository::NewSensorReading;
use crate::services::telemetry::persistence::persist_sensor_reading;

pub async fn persist_gps_reading(
    db_pool: &PgPool,
    device_id: Uuid,
    gps_sensor_id: Uuid,
    reading: &FuelReading,
) -> Result<()> {
    let raw_payload: Value = serde_json::to_value(reading)?;

    persist_sensor_reading(
        db_pool,
        NewSensorReading {
            sensor_id: gps_sensor_id,
            device_id,
            recorded_at: reading.timestamp,

            // The current sensor_readings schema requires a numeric value.
            // GPS truth is stored in latitude/longitude, so value remains 0.0.
            value: 0.0,
            unit: "coordinates".to_string(),

            latitude: Some(reading.latitude),
            longitude: Some(reading.longitude),

            vibration_level: None,
            motion_detected: Some(reading.motion_detected),

            raw_payload,
        },
    )
    .await
}
