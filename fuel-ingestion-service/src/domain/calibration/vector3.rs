use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

/// Represents a three-axis calibration value.
///
/// This type is reusable for:
///
/// - accelerometer bias,
/// - accelerometer scale,
/// - gyroscope bias,
/// - gyroscope scale,
/// - future magnetometer calibration.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector3Calibration {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3Calibration {
    /// Creates a new three-axis calibration value.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Returns a zero-valued vector.
    ///
    /// This is appropriate for default sensor bias values.
    pub const fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Returns a unit-valued vector.
    ///
    /// This is appropriate for default sensor scale values.
    pub const fn one() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }

    /// Ensures every axis contains a finite numeric value.
    ///
    /// NaN and positive or negative infinity are rejected because
    /// they would corrupt downstream telemetry interpretation.
    pub fn validate_finite(&self, field_name: &str) -> Result<()> {
        if !self.x.is_finite() {
            return Err(anyhow!("{field_name}.x must be a finite number."));
        }

        if !self.y.is_finite() {
            return Err(anyhow!("{field_name}.y must be a finite number."));
        }

        if !self.z.is_finite() {
            return Err(anyhow!("{field_name}.z must be a finite number."));
        }

        Ok(())
    }

    /// Ensures no axis contains a zero scale factor.
    ///
    /// This validation is intended for vectors used as sensor scale
    /// factors. Bias vectors are allowed to contain zero values.
    pub fn validate_non_zero(&self, field_name: &str) -> Result<()> {
        if self.x == 0.0 {
            return Err(anyhow!("{field_name}.x must not be zero."));
        }

        if self.y == 0.0 {
            return Err(anyhow!("{field_name}.y must not be zero."));
        }

        if self.z == 0.0 {
            return Err(anyhow!("{field_name}.z must not be zero."));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_returns_zero_for_every_axis() {
        let vector = Vector3Calibration::zero();

        assert_eq!(vector.x, 0.0);
        assert_eq!(vector.y, 0.0);
        assert_eq!(vector.z, 0.0);
    }

    #[test]
    fn one_returns_one_for_every_axis() {
        let vector = Vector3Calibration::one();

        assert_eq!(vector.x, 1.0);
        assert_eq!(vector.y, 1.0);
        assert_eq!(vector.z, 1.0);
    }

    #[test]
    fn finite_values_pass_validation() {
        let vector = Vector3Calibration::new(0.1, -0.2, 1.0);

        assert!(vector.validate_finite("accelerometer_bias").is_ok());
    }

    #[test]
    fn nan_value_fails_validation() {
        let vector = Vector3Calibration::new(f64::NAN, 0.0, 0.0);

        let error = vector
            .validate_finite("accelerometer_bias")
            .expect_err("NaN should fail validation");

        assert_eq!(
            error.to_string(),
            "accelerometer_bias.x must be a finite number."
        );
    }

    #[test]
    fn infinite_value_fails_validation() {
        let vector = Vector3Calibration::new(1.0, f64::INFINITY, 1.0);

        let error = vector
            .validate_finite("gyroscope_scale")
            .expect_err("infinity should fail validation");

        assert_eq!(
            error.to_string(),
            "gyroscope_scale.y must be a finite number."
        );
    }

    #[test]
    fn non_zero_scale_passes_validation() {
        let vector = Vector3Calibration::new(1.0, 0.99, 1.02);

        assert!(vector.validate_non_zero("accelerometer_scale").is_ok());
    }

    #[test]
    fn zero_scale_fails_validation() {
        let vector = Vector3Calibration::new(1.0, 0.0, 1.0);

        let error = vector
            .validate_non_zero("accelerometer_scale")
            .expect_err("zero scale should fail validation");

        assert_eq!(error.to_string(), "accelerometer_scale.y must not be zero.");
    }
}
