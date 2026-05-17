use crate::config::AppConfig;
use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::repository::{
    create_fuel_event, get_latest_device_state, get_previous_sensor_reading,
    get_recent_sensor_readings, recent_event_type_exists, recent_similar_event_exists,
};

use crate::services::telemetry_filter::{
    TelemetryQualityStatus, detect_impossible_fuel_jump, evaluate_fuel_quality_window,
    validate_fuel_range,
};

use crate::services::confidence_scoring::score_fuel_event_confidence;

const THEFT_DROP_THRESHOLD: f64 = 20.0;
const REFILL_INCREASE_THRESHOLD: f64 = 20.0;
const LEAK_TOTAL_DROP_THRESHOLD: f64 = 10.0;
const LEAK_CONSECUTIVE_READINGS: usize = 5;
const EVENT_SUPPRESSION_WINDOW_SECONDS: i64 = 300;
const THEFT_LEAK_CORRELATION_WINDOW_SECONDS: i64 = 900;

pub async fn detect_fuel_event(
    db_pool: &PgPool,
    config: &AppConfig,
    device_id: Uuid,
    sensor_id: Uuid,
) -> Result<()> {
    let current = get_previous_sensor_reading(db_pool, sensor_id).await?;

    let Some((previous, current)) = current else {
        return Ok(());
    };

    let difference = current.value - previous.value;

    let jump_quality = detect_impossible_fuel_jump(
        previous.value,
        current.value,
        config.max_allowed_fuel_jump_litres,
    );

    let recent_readings = get_recent_sensor_readings(db_pool, sensor_id, 7).await?;

    let baseline_values: Vec<f64> = recent_readings
        .iter()
        .skip(1)
        .take(5)
        .map(|reading| reading.value)
        .collect();

    let candidate_values = vec![current.value];

    let quality_summary = evaluate_fuel_quality_window(
        &baseline_values,
        &candidate_values,
        config.fuel_rolling_window_size,
        config.fuel_iqr_multiplier,
    );

    let duration_seconds = (current.recorded_at - previous.recorded_at).num_seconds();

    let sync_delay_seconds = (Utc::now() - current.recorded_at).num_seconds().max(0);

    let is_delayed_detection = sync_delay_seconds > 300;

    let latest_device_state = get_latest_device_state(db_pool, device_id)
        .await?
        .unwrap_or_else(|| "UNKNOWN".to_string());

    let fuel_range_validation =
        validate_fuel_range(current.value, config.default_tank_capacity_litres);

    if fuel_range_validation.status == TelemetryQualityStatus::Invalid {
        println!(
            "Skipping fuel event detection due to invalid fuel reading: {:?}",
            fuel_range_validation.reason
        );

        return Ok(());
    }

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

        let confidence = score_fuel_event_confidence(
            "THEFT",
            &latest_device_state,
            quality_summary.outlier_count,
            quality_summary.candidate_count,
            jump_quality.reason.is_some(),
            is_delayed_detection,
        );

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
                "Possible fuel theft detected while device state was {}. Fuel dropped by {:.2} litres. Rolling median: {:?}, IQR: {:?}, outlier count: {}, candidate count: {},  Jump quality: {:?}. Confidence: {:?}.",
                latest_device_state,
                difference.abs(),
                quality_summary.rolling_median,
                quality_summary.iqr,
                quality_summary.outlier_count,
                quality_summary.candidate_count,
                jump_quality.reason,
                confidence,
            ),
            Some(format!("{:?}", confidence)),
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

        let confidence = score_fuel_event_confidence(
            "REFILL",
            &latest_device_state,
            quality_summary.outlier_count,
            quality_summary.candidate_count,
            jump_quality.reason.is_some(),
            is_delayed_detection,
        );

        let refill_interpretation = match latest_device_state.as_str() {
            "MOVING" => "Suspicious fuel increase detected while moving",
            "IDLE" | "PARKED" => "Fuel refill detected",
            "OFFLINE" => "Fuel increase detected while device was offline",
            _ => "Fuel increase detected",
        };

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
               "{} while device state was {}. Fuel increased by {:.2} litres. Rolling median: {:?}, IQR: {:?}, outlier count: {}, candidate count: {}, Jump quality: {:?}. Confidence: {:?}.",
                refill_interpretation,
                latest_device_state,
                difference.abs(),
                quality_summary.rolling_median,
                quality_summary.iqr,
                quality_summary.outlier_count,
                quality_summary.candidate_count,
                jump_quality.reason,
                confidence,
            ),
            Some(format!("{:?}", confidence)),
        )
        .await?;

        println!("REFILL EVENT DETECTED");
    }

    Ok(())
}

pub async fn detect_possible_leak(
    db_pool: &PgPool,
    config: &AppConfig,
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

    let latest_device_state = get_latest_device_state(db_pool, device_id)
        .await?
        .unwrap_or_else(|| "UNKNOWN".to_string());

    let baseline_values: Vec<f64> = readings
        .iter()
        .skip(1)
        .map(|reading| reading.value)
        .collect();

    let candidate_values = vec![newest.value];

    let quality_summary = evaluate_fuel_quality_window(
        &baseline_values,
        &candidate_values,
        config.fuel_rolling_window_size,
        config.fuel_iqr_multiplier,
    );

    let already_exists =
        recent_similar_event_exists(db_pool, sensor_id, "LEAK", EVENT_SUPPRESSION_WINDOW_SECONDS)
            .await?;

    if already_exists {
        return Ok(());
    }

    let confidence = score_fuel_event_confidence(
        "LEAK",
        &latest_device_state,
        quality_summary.outlier_count,
        quality_summary.candidate_count,
        false,
        is_delayed_detection,
    );

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
            "Possible fuel leak detected while device state was {}. Fuel gradually dropped by {:.2} litres. Rolling median: {:?}, IQR: {:?}, outlier count: {}, candidate count: {}, Confidence: {:?}.",
            latest_device_state,
            total_drop.abs(),
            quality_summary.rolling_median,
            quality_summary.iqr,
            quality_summary.outlier_count,
            quality_summary.candidate_count,
            confidence,
        ),
        Some(format!("{:?}", confidence)),
    )
    .await?;

    println!("LEAK EVENT DETECTED");

    Ok(())
}
