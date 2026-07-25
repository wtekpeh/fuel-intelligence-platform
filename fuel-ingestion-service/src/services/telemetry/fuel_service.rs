use anyhow::Result;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::FuelReading;
use crate::repository::NewSensorReading;
use crate::services::telemetry::persistence::persist_sensor_reading;

/// Persists fuel telemetry for devices that include the Fuel Intelligence
/// capability.
///
/// Motion classification, device-state history, operational transitions,
/// and operational intelligence are handled separately by motion_service.
pub async fn persist_fuel_reading(
    db_pool: &PgPool,
    device_id: Uuid,
    fuel_sensor_id: Uuid,
    reading: &FuelReading,
) -> Result<()> {
    let raw_payload: Value = serde_json::to_value(reading)?;

    persist_sensor_reading(
        db_pool,
        NewSensorReading {
            sensor_id: fuel_sensor_id,
            device_id,
            recorded_at: reading.timestamp,

            value: reading.fuel_level_litres,
            unit: "litres".to_string(),

            latitude: Some(reading.latitude),
            longitude: Some(reading.longitude),

            // Motion evidence is now persisted through the shared vibration
            // and operational-intelligence pipeline rather than being owned
            // by the fuel sensor reading.
            vibration_level: None,
            motion_detected: None,

            raw_payload,
        },
    )
    .await
}
