use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::repository::{
    create_fuel_event, get_previous_sensor_reading, get_recent_sensor_readings,
    recent_event_type_exists, recent_similar_event_exists,
};

const THEFT_DROP_THRESHOLD: f64 = 20.0;
const REFILL_INCREASE_THRESHOLD: f64 = 20.0;
const LEAK_TOTAL_DROP_THRESHOLD: f64 = 10.0;
const LEAK_CONSECUTIVE_READINGS: usize = 5;
const EVENT_SUPPRESSION_WINDOW_SECONDS: i64 = 300;
const THEFT_LEAK_CORRELATION_WINDOW_SECONDS: i64 = 900;

pub async fn detect_fuel_event(db_pool: &PgPool, device_id: Uuid, sensor_id: Uuid) -> Result<()> {
    let current = get_previous_sensor_reading(db_pool, sensor_id).await?;

    let Some((previous, current)) = current else {
        return Ok(());
    };

    let difference = current.value - previous.value;

    let duration_seconds = (current.recorded_at - previous.recorded_at).num_seconds();

    let sync_delay_seconds = (Utc::now() - current.recorded_at).num_seconds().max(0);

    let is_delayed_detection = sync_delay_seconds > 300;

    if difference <= -THEFT_DROP_THRESHOLD {
        let already_exists = recent_similar_event_exists(
            db_pool,
            sensor_id,
            "THEFT",
            EVENT_SUPPRESSION_WINDOW_SECONDS,
        )
        .await?;

        if already_exists {
            return Ok(());
        }

        create_fuel_event(
            db_pool,
            device_id,
            sensor_id,
            "THEFT",
            current.recorded_at,
            previous.value,
            current.value,
            difference.abs(),
            duration_seconds,
            current.latitude,
            current.longitude,
            is_delayed_detection,
            sync_delay_seconds,
            "high",
            format!(
                "Possible fuel theft detected. Fuel dropped by {:.2} litres.",
                difference.abs()
            ),
        )
        .await?;

        println!("THEFT EVENT DETECTED");
    }

    if difference >= REFILL_INCREASE_THRESHOLD {
        let already_exists = recent_similar_event_exists(
            db_pool,
            sensor_id,
            "REFILL",
            EVENT_SUPPRESSION_WINDOW_SECONDS,
        )
        .await?;

        if already_exists {
            return Ok(());
        }

        create_fuel_event(
            db_pool,
            device_id,
            sensor_id,
            "REFILL",
            current.recorded_at,
            previous.value,
            current.value,
            difference.abs(),
            duration_seconds,
            current.latitude,
            current.longitude,
            is_delayed_detection,
            sync_delay_seconds,
            "medium",
            format!(
                "Fuel refill detected. Fuel increased by {:.2} litres.",
                difference.abs()
            ),
        )
        .await?;

        println!("REFILL EVENT DETECTED");
    }

    Ok(())
}

pub async fn detect_possible_leak(
    db_pool: &PgPool,
    device_id: Uuid,
    sensor_id: Uuid,
) -> Result<()> {
    let readings =
        get_recent_sensor_readings(db_pool, sensor_id, LEAK_CONSECUTIVE_READINGS as i64).await?;

    if readings.len() < LEAK_CONSECUTIVE_READINGS {
        return Ok(());
    }

    let mut continuously_dropping = true;

    for window in readings.windows(2) {
        let current = &window[0];
        let previous = &window[1];

        if current.value >= previous.value {
            continuously_dropping = false;
            break;
        }
    }

    if !continuously_dropping {
        return Ok(());
    }

    let recent_theft_exists = recent_event_type_exists(
        db_pool,
        sensor_id,
        "THEFT",
        THEFT_LEAK_CORRELATION_WINDOW_SECONDS,
    )
    .await?;

    if recent_theft_exists {
        // Suppress leak detection when a recent theft event already explains the fuel instability.

        return Ok(());
    }

    let newest = &readings[0];
    let oldest = &readings[readings.len() - 1];

    let total_drop = oldest.value - newest.value;

    if total_drop < LEAK_TOTAL_DROP_THRESHOLD {
        return Ok(());
    }

    let sync_delay_seconds = (Utc::now() - newest.recorded_at).num_seconds().max(0);

    let is_delayed_detection = sync_delay_seconds > 300;

    let already_exists =
        recent_similar_event_exists(db_pool, sensor_id, "LEAK", EVENT_SUPPRESSION_WINDOW_SECONDS)
            .await?;

    if already_exists {
        return Ok(());
    }

    create_fuel_event(
        db_pool,
        device_id,
        sensor_id,
        "LEAK",
        newest.recorded_at,
        oldest.value,
        newest.value,
        total_drop.abs(),
        (newest.recorded_at - oldest.recorded_at).num_seconds(),
        newest.latitude,
        newest.longitude,
        is_delayed_detection,
        sync_delay_seconds,
        "medium",
        format!(
            "Possible fuel leak detected. Fuel gradually dropped by {:.2} litres.",
            total_drop.abs()
        ),
    )
    .await?;

    println!("LEAK EVENT DETECTED");

    Ok(())
}
