use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::repository::{
    create_sensor_health_event, get_recent_sensor_readings, recent_sensor_health_event_exists,
};

const FROZEN_SENSOR_READING_COUNT: i64 = 5;
const FROZEN_SENSOR_MAX_VARIATION: f64 = 0.05;
const SENSOR_HEALTH_EVENT_SUPPRESSION_WINDOW_SECONDS: i64 = 300;

pub async fn detect_frozen_fuel_sensor(
    db_pool: &PgPool,
    device_id: Uuid,
    sensor_id: Uuid,
) -> Result<()> {
    let readings =
        get_recent_sensor_readings(db_pool, sensor_id, FROZEN_SENSOR_READING_COUNT).await?;

    if readings.len() < FROZEN_SENSOR_READING_COUNT as usize {
        return Ok(());
    }

    let newest = &readings[0];
    let oldest = &readings[readings.len() - 1];

    let min_value = readings
        .iter()
        .map(|reading| reading.value)
        .fold(f64::INFINITY, f64::min);

    let max_value = readings
        .iter()
        .map(|reading| reading.value)
        .fold(f64::NEG_INFINITY, f64::max);

    let variation = max_value - min_value;

    if variation > FROZEN_SENSOR_MAX_VARIATION {
        return Ok(());
    }

    let already_exists = recent_sensor_health_event_exists(
        db_pool,
        sensor_id,
        "SENSOR_FROZEN",
        SENSOR_HEALTH_EVENT_SUPPRESSION_WINDOW_SECONDS,
    )
    .await?;

    if already_exists {
        return Ok(());
    }

    create_sensor_health_event(
        db_pool,
        device_id,
        sensor_id,
        "SENSOR_FROZEN",
        "medium",
        "Fuel sensor value remained almost unchanged across recent readings.",
        Some(oldest.recorded_at),
        Some(newest.recorded_at),
    )
    .await?;

    println!("SENSOR HEALTH EVENT DETECTED: SENSOR_FROZEN");

    Ok(())
}
