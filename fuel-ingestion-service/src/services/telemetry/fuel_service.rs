use anyhow::Result;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::telemetry::models::FuelTelemetry;
use crate::models::FuelReading;
use crate::repository::NewSensorReading;
use crate::services::telemetry::persistence::persist_sensor_reading;

/// Persists fuel telemetry for devices that include the Fuel Intelligence
/// capability.
///
/// The original legacy reading is retained as the raw payload so that the
/// complete physical telemetry received from the device remains available.
///
/// The persisted fuel value itself comes from the enriched canonical fuel
/// telemetry. This ensures that physical KUM distance measurements are first
/// converted through the backend tank-calibration pipeline before a litres
/// reading is stored.
///
/// Motion classification, device-state history, operational transitions,
/// and operational intelligence are handled separately by motion_service.
pub async fn persist_fuel_reading(
    db_pool: &PgPool,
    device_id: Uuid,
    fuel_sensor_id: Uuid,
    reading: &FuelReading,
    fuel: &FuelTelemetry,
) -> Result<()> {
    let raw_payload: Value = serde_json::to_value(reading)?;

    /*
     * Physical firmware submits raw fuel measurements such as ultrasonic
     * distance in centimetres.
     *
     * The backend enrichment pipeline converts those measurements into
     * calibrated litres and percentage using the active tank calibration.
     *
     * If calibration cannot produce a trustworthy result, calibrated remains
     * None and no litres reading is persisted.
     */
    let Some(calibrated_fuel) = fuel.calibrated.as_ref() else {
        return Ok(());
    };

    persist_sensor_reading(
        db_pool,
        NewSensorReading {
            sensor_id: fuel_sensor_id,
            device_id,
            recorded_at: reading.timestamp,

            value: calibrated_fuel.litres,
            unit: "litres".to_string(),

            latitude: Some(reading.latitude),
            longitude: Some(reading.longitude),

            vibration_level: None,
            motion_detected: None,

            raw_payload,
        },
    )
    .await
}
