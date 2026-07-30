use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::Vector3Calibration;

/// Strongly typed calibration parameters for a three-axis IMU.
///
/// Bias values represent fixed offsets removed from sensor measurements.
///
/// Scale values represent per-axis correction multipliers applied after
/// bias removal.
///
/// The intended correction formula is:
///
/// corrected = (measured - bias) * scale
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImuCalibration {
    pub accelerometer_bias: Vector3Calibration,
    pub accelerometer_scale: Vector3Calibration,

    pub gyroscope_bias: Vector3Calibration,
    pub gyroscope_scale: Vector3Calibration,
}

impl ImuCalibration {
    /// Creates an IMU calibration from explicitly supplied parameters.
    pub const fn new(
        accelerometer_bias: Vector3Calibration,
        accelerometer_scale: Vector3Calibration,
        gyroscope_bias: Vector3Calibration,
        gyroscope_scale: Vector3Calibration,
    ) -> Self {
        Self {
            accelerometer_bias,
            accelerometer_scale,
            gyroscope_bias,
            gyroscope_scale,
        }
    }

    /// Creates a neutral calibration that leaves all measurements unchanged.
    ///
    /// Bias defaults to zero and scale defaults to one:
    ///
    /// corrected = (measured - 0) * 1
    pub const fn identity() -> Self {
        Self::new(
            Vector3Calibration::zero(),
            Vector3Calibration::one(),
            Vector3Calibration::zero(),
            Vector3Calibration::one(),
        )
    }

    /// Validates all IMU calibration parameters.
    ///
    /// Every value must be finite, and scale factors must not be zero.
    pub fn validate(&self) -> Result<()> {
        self.accelerometer_bias
            .validate_finite("accelerometer_bias")?;

        self.accelerometer_scale
            .validate_finite("accelerometer_scale")?;

        self.accelerometer_scale
            .validate_non_zero("accelerometer_scale")?;

        self.gyroscope_bias.validate_finite("gyroscope_bias")?;

        self.gyroscope_scale.validate_finite("gyroscope_scale")?;

        self.gyroscope_scale.validate_non_zero("gyroscope_scale")?;

        Ok(())
    }
}

impl Default for ImuCalibration {
    fn default() -> Self {
        Self::identity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_calibration_uses_neutral_values() {
        let calibration = ImuCalibration::identity();

        assert_eq!(calibration.accelerometer_bias, Vector3Calibration::zero());

        assert_eq!(calibration.accelerometer_scale, Vector3Calibration::one());

        assert_eq!(calibration.gyroscope_bias, Vector3Calibration::zero());
        assert_eq!(calibration.gyroscope_scale, Vector3Calibration::one());
    }

    #[test]
    fn identity_calibration_passes_validation() {
        let calibration = ImuCalibration::identity();

        assert!(calibration.validate().is_ok());
    }

    #[test]
    fn valid_custom_calibration_passes_validation() {
        let calibration = ImuCalibration::new(
            Vector3Calibration::new(0.01, -0.02, 0.03),
            Vector3Calibration::new(1.01, 0.99, 1.02),
            Vector3Calibration::new(0.5, -0.4, 0.2),
            Vector3Calibration::new(1.0, 1.01, 0.98),
        );

        assert!(calibration.validate().is_ok());
    }

    #[test]
    fn non_finite_accelerometer_bias_fails_validation() {
        let calibration = ImuCalibration::new(
            Vector3Calibration::new(f64::NAN, 0.0, 0.0),
            Vector3Calibration::one(),
            Vector3Calibration::zero(),
            Vector3Calibration::one(),
        );

        let error = calibration
            .validate()
            .expect_err("non-finite accelerometer bias should fail");

        assert_eq!(
            error.to_string(),
            "accelerometer_bias.x must be a finite number."
        );
    }

    #[test]
    fn zero_accelerometer_scale_fails_validation() {
        let calibration = ImuCalibration::new(
            Vector3Calibration::zero(),
            Vector3Calibration::new(1.0, 0.0, 1.0),
            Vector3Calibration::zero(),
            Vector3Calibration::one(),
        );

        let error = calibration
            .validate()
            .expect_err("zero accelerometer scale should fail");

        assert_eq!(error.to_string(), "accelerometer_scale.y must not be zero.");
    }

    #[test]
    fn non_finite_gyroscope_bias_fails_validation() {
        let calibration = ImuCalibration::new(
            Vector3Calibration::zero(),
            Vector3Calibration::one(),
            Vector3Calibration::new(0.0, f64::NEG_INFINITY, 0.0),
            Vector3Calibration::one(),
        );

        let error = calibration
            .validate()
            .expect_err("non-finite gyroscope bias should fail");

        assert_eq!(
            error.to_string(),
            "gyroscope_bias.y must be a finite number."
        );
    }

    #[test]
    fn zero_gyroscope_scale_fails_validation() {
        let calibration = ImuCalibration::new(
            Vector3Calibration::zero(),
            Vector3Calibration::one(),
            Vector3Calibration::zero(),
            Vector3Calibration::new(1.0, 1.0, 0.0),
        );

        let error = calibration
            .validate()
            .expect_err("zero gyroscope scale should fail");

        assert_eq!(error.to_string(), "gyroscope_scale.z must not be zero.");
    }
}
