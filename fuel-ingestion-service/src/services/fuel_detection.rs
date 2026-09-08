use crate::config::AppConfig;
use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::repository::{
    create_alert, create_fuel_event, get_latest_device_state, get_previous_sensor_reading,
    get_recent_sensor_readings, recent_event_type_exists, recent_similar_event_exists,
};

use crate::services::telemetry_filter::{
    TelemetryQualityStatus, detect_impossible_fuel_jump, evaluate_fuel_quality_window,
    validate_fuel_range,
};

use crate::services::fuel_calibration_service::FuelCalibrationService;
use crate::services::fuel_event_severity::calculate_fuel_event_severity;

use crate::services::alert_hub::AlertHub;
use crate::services::alert_rules::evaluate_alert_rule;
use crate::services::confidence_scoring::score_fuel_event_confidence;
use crate::services::fuel_event_correlation::correlate_fuel_event;

const LEAK_CONSECUTIVE_READINGS: usize = 5;
const EVENT_SUPPRESSION_WINDOW_SECONDS: i64 = 300;
const THEFT_LEAK_CORRELATION_WINDOW_SECONDS: i64 = 900;

fn recent_reading_limit_for_baseline(fuel_rolling_window_size: usize) -> i64 {
    (fuel_rolling_window_size + 1) as i64
}

fn recent_reading_limit_for_leak(fuel_rolling_window_size: usize) -> i64 {
    (fuel_rolling_window_size + 1).max(LEAK_CONSECUTIVE_READINGS) as i64
}

pub async fn detect_fuel_event(
    db_pool: &PgPool,
    alert_hub: &AlertHub,
    config: &AppConfig,
    fuel_calibration_service: &FuelCalibrationService,
    device_id: Uuid,
    sensor_id: Uuid,
) -> Result<()> {
    let Some(fuel_calibration) = fuel_calibration_service
        .get_active_calibration(sensor_id)
        .await?
    else {
        println!(
            "Skipping fuel event detection because sensor {} has no active fuel calibration.",
            sensor_id
        );

        return Ok(());
    };

    let tank_capacity_litres = fuel_calibration.tank_capacity_litres;

    let theft_drop_threshold_litres = tank_capacity_litres * config.fuel_theft_threshold_fraction;

    let refill_increase_threshold_litres =
        tank_capacity_litres * config.fuel_refill_threshold_fraction;

    let current = get_previous_sensor_reading(db_pool, sensor_id).await?;

    let Some((previous, current)) = current else {
        return Ok(());
    };

    let previous_fuel_range_validation = validate_fuel_range(previous.value, tank_capacity_litres);

    if previous_fuel_range_validation.status == TelemetryQualityStatus::Invalid {
        println!(
            "Skipping fuel event detection due to invalid previous fuel reading: {:?}",
            previous_fuel_range_validation.reason
        );

        return Ok(());
    }

    let current_fuel_range_validation = validate_fuel_range(current.value, tank_capacity_litres);

    if current_fuel_range_validation.status == TelemetryQualityStatus::Invalid {
        println!(
            "Skipping fuel event detection due to invalid current fuel reading: {:?}",
            current_fuel_range_validation.reason
        );

        return Ok(());
    }

    let difference = current.value - previous.value;

    let max_allowed_fuel_jump_litres = tank_capacity_litres * config.max_allowed_fuel_jump_fraction;

    let jump_quality =
        detect_impossible_fuel_jump(previous.value, current.value, max_allowed_fuel_jump_litres);

    if jump_quality.status == TelemetryQualityStatus::Invalid {
        println!(
            "Skipping fuel event detection because fuel jump validation is invalid: {:?}",
            jump_quality.reason
        );

        return Ok(());
    }

    let recent_reading_limit = recent_reading_limit_for_baseline(config.fuel_rolling_window_size);

    let recent_readings =
        get_recent_sensor_readings(db_pool, sensor_id, recent_reading_limit).await?;

    let mut baseline_values: Vec<f64> = recent_readings
        .iter()
        .skip(1)
        .take(config.fuel_rolling_window_size)
        .map(|reading| reading.value)
        .collect();

    // Repository readings are returned newest-first because the query uses
    // ORDER BY recorded_at DESC.
    //
    // The rolling-median helper expects chronological ordering, with the newest
    // values at the end of the slice.
    baseline_values.reverse();

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

    if difference <= -theft_drop_threshold_litres {
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
            &latest_device_state,
            quality_summary.outlier_count > 0,
            jump_quality.status == TelemetryQualityStatus::Suspicious,
            is_delayed_detection,
        );

        let correlation = correlate_fuel_event(
            "THEFT",
            &latest_device_state,
            latest_device_state == "MOVING",
        );

        let event_severity = calculate_fuel_event_severity(difference.abs(), tank_capacity_litres);

        let fuel_event_id = create_fuel_event(
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
            event_severity.as_str(),
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
            Some(format!("{:?}", correlation.status)),
Some(correlation.reason),
        )
        .await?;

        let alert_decision = evaluate_alert_rule("THEFT", &confidence, &correlation.status);

        if alert_decision.should_alert {
            let alert = create_alert(
                db_pool,
                Some(fuel_event_id),
                "THEFT".to_string(),
                format!("{:?}", alert_decision.severity),
                alert_decision.reason,
            )
            .await?;

            alert_hub.broadcast_alert(alert);
        }

        println!("THEFT EVENT DETECTED");
    }

    if difference >= refill_increase_threshold_litres {
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
            &latest_device_state,
            quality_summary.outlier_count > 0,
            jump_quality.status == TelemetryQualityStatus::Suspicious,
            is_delayed_detection,
        );

        let refill_interpretation = match latest_device_state.as_str() {
            "MOVING" => "Suspicious fuel increase detected while moving",
            "IDLE" | "PARKED" => "Fuel refill detected",
            "OFFLINE" => "Fuel increase detected while device was offline",
            _ => "Fuel increase detected",
        };

        let correlation = correlate_fuel_event(
            "REFILL",
            &latest_device_state,
            latest_device_state == "MOVING",
        );

        let event_severity = calculate_fuel_event_severity(difference.abs(), tank_capacity_litres);

        let fuel_event_id = create_fuel_event(
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
            event_severity.as_str(),
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
            Some(format!("{:?}", correlation.status)),
Some(correlation.reason),
        )
        .await?;

        let alert_decision = evaluate_alert_rule("REFILL", &confidence, &correlation.status);

        if alert_decision.should_alert {
            let alert = create_alert(
                db_pool,
                Some(fuel_event_id),
                "REFILL".to_string(),
                format!("{:?}", alert_decision.severity),
                alert_decision.reason,
            )
            .await?;

            alert_hub.broadcast_alert(alert);
        }

        println!("REFILL EVENT DETECTED");
    }

    Ok(())
}

pub async fn detect_possible_leak(
    db_pool: &PgPool,
    alert_hub: &AlertHub,
    config: &AppConfig,
    fuel_calibration_service: &FuelCalibrationService,
    device_id: Uuid,
    sensor_id: Uuid,
) -> Result<()> {
    let Some(fuel_calibration) = fuel_calibration_service
        .get_active_calibration(sensor_id)
        .await?
    else {
        println!(
            "Skipping fuel leak detection because sensor {} has no active fuel calibration.",
            sensor_id
        );

        return Ok(());
    };

    let tank_capacity_litres = fuel_calibration.tank_capacity_litres;

    let leak_total_drop_threshold_litres =
        tank_capacity_litres * config.fuel_leak_threshold_fraction;

    let recent_reading_limit = recent_reading_limit_for_leak(config.fuel_rolling_window_size);

    let readings = get_recent_sensor_readings(db_pool, sensor_id, recent_reading_limit).await?;

    if readings.len() < LEAK_CONSECUTIVE_READINGS {
        return Ok(());
    }

    // Leak detection itself still uses exactly the latest
    // LEAK_CONSECUTIVE_READINGS readings.
    //
    // The repository returns newest-first.
    let leak_readings = &readings[..LEAK_CONSECUTIVE_READINGS];

    let mut continuously_dropping = true;

    for window in leak_readings.windows(2) {
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

    let newest = &leak_readings[0];
    let oldest = &leak_readings[leak_readings.len() - 1];

    let total_drop = oldest.value - newest.value;

    if total_drop < leak_total_drop_threshold_litres {
        return Ok(());
    }

    let sync_delay_seconds = (Utc::now() - newest.recorded_at).num_seconds().max(0);

    let is_delayed_detection = sync_delay_seconds > 300;

    let latest_device_state = get_latest_device_state(db_pool, device_id)
        .await?
        .unwrap_or_else(|| "UNKNOWN".to_string());

    if latest_device_state != "PARKED" && latest_device_state != "IDLE" {
        println!(
            "Skipping fuel leak detection because device state {} does not provide stationary leak context.",
            latest_device_state
        );

        return Ok(());
    }

    let mut baseline_values: Vec<f64> = readings
        .iter()
        .skip(1)
        .take(config.fuel_rolling_window_size)
        .map(|reading| reading.value)
        .collect();

    // Repository readings are newest-first.
    // Quality-window helpers expect chronological ordering.
    baseline_values.reverse();

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
        &latest_device_state,
        quality_summary.outlier_count > 0,
        false,
        is_delayed_detection,
    );

    let correlation = correlate_fuel_event(
        "LEAK",
        &latest_device_state,
        latest_device_state == "MOVING",
    );

    let event_severity = calculate_fuel_event_severity(total_drop.abs(), tank_capacity_litres);

    let fuel_event_id = create_fuel_event(
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
        event_severity.as_str(),
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
        Some(format!("{:?}", correlation.status)),
Some(correlation.reason),
    )
    .await?;

    let alert_decision = evaluate_alert_rule("LEAK", &confidence, &correlation.status);

    if alert_decision.should_alert {
        let alert = create_alert(
            db_pool,
            Some(fuel_event_id),
            "LEAK".to_string(),
            format!("{:?}", alert_decision.severity),
            alert_decision.reason,
        )
        .await?;
        alert_hub.broadcast_alert(alert);
    }

    println!("LEAK EVENT DETECTED");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_fetch_limit_includes_current_reading() {
        let limit = recent_reading_limit_for_baseline(5);

        assert_eq!(limit, 6);
    }

    #[test]
    fn baseline_fetch_limit_scales_with_configured_window() {
        let limit = recent_reading_limit_for_baseline(10);

        assert_eq!(limit, 11);
    }

    #[test]
    fn leak_fetch_limit_preserves_required_consecutive_readings() {
        let limit = recent_reading_limit_for_leak(3);

        assert_eq!(limit, LEAK_CONSECUTIVE_READINGS as i64);
    }

    #[test]
    fn leak_fetch_limit_scales_with_quality_window() {
        let limit = recent_reading_limit_for_leak(10);

        assert_eq!(limit, 11);
    }
}
