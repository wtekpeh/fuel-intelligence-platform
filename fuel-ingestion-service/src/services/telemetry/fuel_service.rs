use anyhow::Result;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::telemetry::imu_interpreter::ImuInterpretation;
use crate::models::FuelReading;
use crate::repository::{self, NewDeviceStateEvent, NewSensorReading};
use crate::services::device_state::{
    calculate_distance_meters, calculate_speed_kmh, classify_device_state,
};
use crate::services::telemetry::persistence::persist_sensor_reading;

/// Persists fuel telemetry and records the operational state derived
/// from the current and previous telemetry readings.
///
/// Raw accelerometer and gyroscope measurements are interpreted by the
/// backend. The physical firmware does not determine vibration level,
/// motion state, or operational device state.
pub async fn persist_fuel_reading(
    db_pool: &PgPool,
    device_id: Uuid,
    fuel_sensor_id: Uuid,
    reading: &FuelReading,
    previous_reading: Option<&FuelReading>,
    imu_interpretation: &ImuInterpretation,
) -> Result<()> {
    let (previous_latitude, previous_longitude, distance_meters, speed_kmh) =
        calculate_movement(reading, previous_reading);

    let device_state = classify_device_state(
        Some("ONLINE"),
        Some(imu_interpretation.vibration_score),
        Some(imu_interpretation.motion_detected),
        previous_latitude,
        previous_longitude,
        Some(reading.latitude),
        Some(reading.longitude),
    );

    repository::create_device_state_event(
        db_pool,
        NewDeviceStateEvent {
            device_id,
            sensor_id: Some(fuel_sensor_id),

            state: device_state.as_str().to_string(),

            recorded_at: reading.timestamp,

            vibration_level: Some(imu_interpretation.vibration_score),
            motion_detected: Some(imu_interpretation.motion_detected),

            distance_meters,
            speed_kmh,

            latitude: Some(reading.latitude),
            longitude: Some(reading.longitude),

            source: "telemetry".to_string(),

            message: Some(format!(
                "State classified from telemetry. State: {:?}, vibration score: {:.2}, \
                 movement confidence: {:.2}",
                device_state,
                imu_interpretation.vibration_score,
                imu_interpretation.movement_confidence,
            )),
        },
    )
    .await?;

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

            vibration_level: Some(imu_interpretation.vibration_score),
            motion_detected: Some(imu_interpretation.motion_detected),

            raw_payload,
        },
    )
    .await
}

fn calculate_movement(
    reading: &FuelReading,
    previous_reading: Option<&FuelReading>,
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    let Some(previous_reading) = previous_reading else {
        return (None, None, None, None);
    };

    let distance_meters = calculate_distance_meters(
        previous_reading.latitude,
        previous_reading.longitude,
        reading.latitude,
        reading.longitude,
    );

    let time_seconds = (reading.timestamp - previous_reading.timestamp)
        .num_seconds()
        .max(0) as f64;

    let speed_kmh = calculate_speed_kmh(distance_meters, time_seconds);

    (
        Some(previous_reading.latitude),
        Some(previous_reading.longitude),
        Some(distance_meters),
        Some(speed_kmh),
    )
}
