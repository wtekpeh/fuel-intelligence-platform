use anyhow::{Context, Result, anyhow};

use crate::models::SensorCalibration;
use crate::services::calibration_type::CalibrationType;

use super::{FuelCalibration, ImuCalibration};

/// Converts persisted calibration records into strongly typed
/// domain calibration objects.
///
/// The factory owns:
///
/// - calibration type verification,
/// - JSON deserialization,
/// - domain validation.
///
/// It does not load records from the database and does not apply
/// calibration values to telemetry.
pub struct CalibrationFactory;

impl CalibrationFactory {
    /// Converts a persisted sensor calibration into a typed IMU calibration.
    pub fn imu(calibration: &SensorCalibration) -> Result<ImuCalibration> {
        Self::ensure_calibration_type(calibration, CalibrationType::Imu.as_str())?;

        let typed_calibration: ImuCalibration =
            serde_json::from_value(calibration.calibration_values.clone())
                .context("Failed to decode IMU calibration values.")?;

        typed_calibration
            .validate()
            .context("Invalid IMU calibration values.")?;

        Ok(typed_calibration)
    }

    pub fn fuel(calibration: &SensorCalibration) -> Result<FuelCalibration> {
        Self::ensure_calibration_type(calibration, CalibrationType::Fuel.as_str())?;

        let typed_calibration: FuelCalibration =
            serde_json::from_value(calibration.calibration_values.clone())
                .context("Failed to deserialize fuel calibration.")?;

        typed_calibration
            .validate_lookup_table()
            .context("Invalid fuel calibration lookup table.")?;

        Ok(typed_calibration)
    }

    /// Ensures the persisted record contains the expected calibration type.
    fn ensure_calibration_type(calibration: &SensorCalibration, expected_type: &str) -> Result<()> {
        if calibration.calibration_type != expected_type {
            return Err(anyhow!(
                "Expected {expected_type} calibration, but found {}.",
                calibration.calibration_type
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{TimeZone, Utc};
    use serde_json::json;
    use uuid::Uuid;

    fn sensor_calibration(
        calibration_type: &str,
        calibration_values: serde_json::Value,
    ) -> SensorCalibration {
        let timestamp = Utc
            .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
            .single()
            .expect("test timestamp should be valid");

        SensorCalibration {
            id: Uuid::new_v4(),
            sensor_id: Uuid::new_v4(),
            calibration_type: calibration_type.to_string(),
            calibration_values,
            is_active: true,
            calibrated_at: timestamp,
            created_at: timestamp,
            updated_at: timestamp,
        }
    }

    fn valid_imu_values() -> serde_json::Value {
        json!({
            "accelerometer_bias": {
                "x": 0.01,
                "y": -0.02,
                "z": 0.03
            },
            "accelerometer_scale": {
                "x": 1.01,
                "y": 0.99,
                "z": 1.02
            },
            "gyroscope_bias": {
                "x": 0.5,
                "y": -0.4,
                "z": 0.2
            },
            "gyroscope_scale": {
                "x": 1.0,
                "y": 1.01,
                "z": 0.98
            }
        })
    }

    #[test]
    fn valid_imu_calibration_is_decoded() {
        let calibration = sensor_calibration("IMU", valid_imu_values());

        let decoded =
            CalibrationFactory::imu(&calibration).expect("valid IMU calibration should decode");

        assert_eq!(decoded.accelerometer_bias.x, 0.01);
        assert_eq!(decoded.accelerometer_bias.y, -0.02);
        assert_eq!(decoded.accelerometer_bias.z, 0.03);

        assert_eq!(decoded.accelerometer_scale.x, 1.01);
        assert_eq!(decoded.accelerometer_scale.y, 0.99);
        assert_eq!(decoded.accelerometer_scale.z, 1.02);

        assert_eq!(decoded.gyroscope_bias.x, 0.5);
        assert_eq!(decoded.gyroscope_bias.y, -0.4);
        assert_eq!(decoded.gyroscope_bias.z, 0.2);

        assert_eq!(decoded.gyroscope_scale.x, 1.0);
        assert_eq!(decoded.gyroscope_scale.y, 1.01);
        assert_eq!(decoded.gyroscope_scale.z, 0.98);
    }

    #[test]
    fn wrong_calibration_type_is_rejected() {
        let calibration = sensor_calibration("FUEL", valid_imu_values());

        let error = CalibrationFactory::imu(&calibration)
            .expect_err("non-IMU calibration type should fail");

        assert_eq!(
            error.to_string(),
            "Expected IMU calibration, but found FUEL."
        );
    }

    #[test]
    fn missing_required_field_is_rejected() {
        let calibration = sensor_calibration(
            "IMU",
            json!({
                "accelerometer_bias": {
                    "x": 0.0,
                    "y": 0.0,
                    "z": 0.0
                }
            }),
        );

        let error = CalibrationFactory::imu(&calibration)
            .expect_err("incomplete IMU calibration should fail");

        assert!(
            error
                .to_string()
                .contains("Failed to decode IMU calibration values.")
        );
    }

    #[test]
    fn invalid_json_shape_is_rejected() {
        let calibration = sensor_calibration("IMU", json!([1, 2, 3]));

        let error = CalibrationFactory::imu(&calibration)
            .expect_err("array calibration values should fail");

        assert!(
            error
                .to_string()
                .contains("Failed to decode IMU calibration values.")
        );
    }

    #[test]
    fn zero_scale_is_rejected_by_domain_validation() {
        let calibration = sensor_calibration(
            "IMU",
            json!({
                "accelerometer_bias": {
                    "x": 0.0,
                    "y": 0.0,
                    "z": 0.0
                },
                "accelerometer_scale": {
                    "x": 1.0,
                    "y": 0.0,
                    "z": 1.0
                },
                "gyroscope_bias": {
                    "x": 0.0,
                    "y": 0.0,
                    "z": 0.0
                },
                "gyroscope_scale": {
                    "x": 1.0,
                    "y": 1.0,
                    "z": 1.0
                }
            }),
        );

        let error =
            CalibrationFactory::imu(&calibration).expect_err("zero scale should fail validation");

        assert!(
            error
                .to_string()
                .contains("Invalid IMU calibration values.")
        );

        let source = error.root_cause().to_string();

        assert_eq!(source, "accelerometer_scale.y must not be zero.");
    }
}
