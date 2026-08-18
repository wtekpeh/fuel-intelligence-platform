use anyhow::{Context, Result, anyhow};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::calibration::{FuelCalibration, ImuCalibration};
use crate::models::{CreateSensorCalibrationRequest, SensorCalibration};
use crate::repository;
use crate::services::calibration_type::CalibrationType;

pub async fn create_calibration(
    db_pool: &PgPool,
    sensor_id: Uuid,
    request: CreateSensorCalibrationRequest,
) -> Result<Uuid> {
    if !repository::sensor_exists(db_pool, sensor_id).await? {
        return Err(anyhow!("Sensor not found."));
    }

    let calibration_type = parse_calibration_type(&request.calibration_type)?;

    validate_calibration_values(calibration_type, &request.calibration_values)?;

    let normalized_request = CreateSensorCalibrationRequest {
        calibration_type: calibration_type.as_str().to_string(),
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

    let calibration_type = parse_calibration_type(calibration_type)?;

    repository::get_active_sensor_calibration(db_pool, sensor_id, calibration_type.as_str()).await
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

fn parse_calibration_type(calibration_type: &str) -> Result<CalibrationType> {
    CalibrationType::parse(calibration_type).ok_or_else(|| {
        anyhow!(
            "Calibration type '{}' is not supported. Expected imu or fuel.",
            calibration_type.trim()
        )
    })
}

fn validate_calibration_values(
    calibration_type: CalibrationType,
    calibration_values: &Value,
) -> Result<()> {
    match calibration_type {
        CalibrationType::Imu => {
            let calibration: ImuCalibration = serde_json::from_value(calibration_values.clone())
                .context("Calibration values are not a valid IMU calibration.")?;

            calibration
                .validate()
                .context("Calibration values contain an invalid IMU calibration.")?;
        }

        CalibrationType::Fuel => {
            let calibration: FuelCalibration =
                serde_json::from_value(calibration_values.clone())
                    .context("Calibration values are not a valid fuel calibration.")?;

            calibration
                .validate_lookup_table()
                .context("Calibration values contain an invalid fuel calibration.")?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn valid_fuel_calibration_values_are_accepted() {
        let calibration_values = json!({
            "tank_capacity_litres": 200.0,
            "points": [
                {
                    "level_cm": 10.0,
                    "litres": 20.0
                },
                {
                    "level_cm": 30.0,
                    "litres": 60.0
                },
                {
                    "level_cm": 100.0,
                    "litres": 200.0
                }
            ]
        });

        let result = validate_calibration_values(CalibrationType::Fuel, &calibration_values);

        assert!(result.is_ok());
    }

    #[test]
    fn invalid_fuel_lookup_table_is_rejected_before_persistence() {
        let calibration_values = json!({
            "tank_capacity_litres": 200.0,
            "points": [
                {
                    "level_cm": 10.0,
                    "litres": 20.0
                },
                {
                    "level_cm": 30.0,
                    "litres": 60.0
                },
                {
                    "level_cm": 100.0,
                    "litres": 180.0
                }
            ]
        });

        let error = validate_calibration_values(CalibrationType::Fuel, &calibration_values)
            .expect_err("invalid fuel lookup table should be rejected");

        assert!(
            error
                .to_string()
                .contains("Calibration values contain an invalid fuel calibration.")
        );

        assert_eq!(
            error.root_cause().to_string(),
            "The final calibration point must equal the declared tank capacity."
        );
    }

    #[test]
    fn malformed_fuel_calibration_values_are_rejected_before_domain_validation() {
        let calibration_values = json!({
            "tank_capacity_litres": 200.0
        });

        let error = validate_calibration_values(CalibrationType::Fuel, &calibration_values)
            .expect_err("incomplete fuel calibration should be rejected");

        assert!(
            error
                .to_string()
                .contains("Calibration values are not a valid fuel calibration.")
        );
    }

    #[test]
    fn calibration_type_parsing_normalizes_supported_values() {
        assert_eq!(
            parse_calibration_type("FUEL").expect("FUEL should parse"),
            CalibrationType::Fuel
        );

        assert_eq!(
            parse_calibration_type(" fuel ").expect("fuel with whitespace should parse"),
            CalibrationType::Fuel
        );

        assert_eq!(
            parse_calibration_type("IMU").expect("IMU should parse"),
            CalibrationType::Imu
        );

        assert_eq!(
            parse_calibration_type(" imu ").expect("imu with whitespace should parse"),
            CalibrationType::Imu
        );
    }

    #[test]
    fn unsupported_calibration_type_is_rejected() {
        let error = parse_calibration_type("temperature")
            .expect_err("unsupported calibration type should fail");

        assert_eq!(
            error.to_string(),
            "Calibration type 'temperature' is not supported. Expected imu or fuel."
        );
    }
}
