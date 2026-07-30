use anyhow::{Result, anyhow};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{CreateSensorCalibrationRequest, SensorCalibration};
use crate::repository;

pub async fn create_calibration(
    db_pool: &PgPool,
    sensor_id: Uuid,
    request: CreateSensorCalibrationRequest,
) -> Result<Uuid> {
    if !repository::sensor_exists(db_pool, sensor_id).await? {
        return Err(anyhow!("Sensor not found."));
    }

    let calibration_type = normalize_calibration_type(&request.calibration_type)?;

    validate_calibration_values(&request.calibration_values)?;

    let normalized_request = CreateSensorCalibrationRequest {
        calibration_type,
        calibration_values: request.calibration_values,
    };

    repository::create_sensor_calibration(db_pool, sensor_id, &normalized_request).await
}

pub async fn get_active_calibration(
    db_pool: &PgPool,
    sensor_id: Uuid,
    calibration_type: &str,
) -> Result<Option<SensorCalibration>> {
    if !repository::sensor_exists(db_pool, sensor_id).await? {
        return Err(anyhow!("Sensor not found."));
    }

    let calibration_type = normalize_calibration_type(calibration_type)?;

    repository::get_active_sensor_calibration(db_pool, sensor_id, &calibration_type).await
}

pub async fn list_calibration_history(
    db_pool: &PgPool,
    sensor_id: Uuid,
) -> Result<Vec<SensorCalibration>> {
    if !repository::sensor_exists(db_pool, sensor_id).await? {
        return Err(anyhow!("Sensor not found."));
    }

    repository::list_sensor_calibrations(db_pool, sensor_id).await
}

fn normalize_calibration_type(calibration_type: &str) -> Result<String> {
    let normalized = calibration_type.trim().to_uppercase();

    if normalized.is_empty() {
        return Err(anyhow!("Calibration category must not be empty."));
    }

    if normalized.len() > 100 {
        return Err(anyhow!(
            "Calibration category must not exceed 100 characters."
        ));
    }

    if !normalized
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '_' || character == '-')
    {
        return Err(anyhow!(
            "Calibration category may contain only letters, numbers, underscores, and hyphens."
        ));
    }

    Ok(normalized)
}

fn validate_calibration_values(calibration_values: &Value) -> Result<()> {
    let values = calibration_values
        .as_object()
        .ok_or_else(|| anyhow!("Calibration values must be a JSON object."))?;

    if values.is_empty() {
        return Err(anyhow!(
            "Calibration values must contain at least one field."
        ));
    }

    Ok(())
}
