use crate::domain::telemetry::models::ImuTelemetry;

use super::ImuCalibration;

/// Applies typed IMU calibration parameters to canonical IMU telemetry.
///
/// The engine does not mutate the original telemetry sample.
///
/// Correction formula:
///
/// corrected = (measured - bias) * scale
pub struct ImuCalibrationEngine;

impl ImuCalibrationEngine {
    /// Applies accelerometer and gyroscope calibration to one IMU sample.
    pub fn apply(imu: &ImuTelemetry, calibration: &ImuCalibration) -> ImuTelemetry {
        ImuTelemetry {
            accel_x: Self::correct_axis(
                imu.accel_x,
                calibration.accelerometer_bias.x,
                calibration.accelerometer_scale.x,
            ),

            accel_y: Self::correct_axis(
                imu.accel_y,
                calibration.accelerometer_bias.y,
                calibration.accelerometer_scale.y,
            ),

            accel_z: Self::correct_axis(
                imu.accel_z,
                calibration.accelerometer_bias.z,
                calibration.accelerometer_scale.z,
            ),

            gyro_x: Self::correct_axis(
                imu.gyro_x,
                calibration.gyroscope_bias.x,
                calibration.gyroscope_scale.x,
            ),

            gyro_y: Self::correct_axis(
                imu.gyro_y,
                calibration.gyroscope_bias.y,
                calibration.gyroscope_scale.y,
            ),

            gyro_z: Self::correct_axis(
                imu.gyro_z,
                calibration.gyroscope_bias.z,
                calibration.gyroscope_scale.z,
            ),

            // Temperature is not altered by the current IMU calibration model.
            temperature: imu.temperature,
        }
    }

    /// Applies bias removal followed by scale correction.
    fn correct_axis(measured: f64, bias: f64, scale: f64) -> f64 {
        (measured - bias) * scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::domain::calibration::Vector3Calibration;

    fn sample_imu() -> ImuTelemetry {
        ImuTelemetry {
            accel_x: 1.0,
            accel_y: -2.0,
            accel_z: 0.5,

            gyro_x: 10.0,
            gyro_y: -5.0,
            gyro_z: 2.0,

            temperature: Some(27.5),
        }
    }

    #[test]
    fn identity_calibration_leaves_imu_unchanged() {
        let imu = sample_imu();
        let calibration = ImuCalibration::identity();

        let corrected = ImuCalibrationEngine::apply(&imu, &calibration);

        assert_eq!(corrected.accel_x, imu.accel_x);
        assert_eq!(corrected.accel_y, imu.accel_y);
        assert_eq!(corrected.accel_z, imu.accel_z);

        assert_eq!(corrected.gyro_x, imu.gyro_x);
        assert_eq!(corrected.gyro_y, imu.gyro_y);
        assert_eq!(corrected.gyro_z, imu.gyro_z);

        assert_eq!(corrected.temperature, imu.temperature);
    }

    #[test]
    fn accelerometer_bias_is_removed() {
        let imu = sample_imu();

        let calibration = ImuCalibration::new(
            Vector3Calibration::new(0.1, -0.2, 0.3),
            Vector3Calibration::one(),
            Vector3Calibration::zero(),
            Vector3Calibration::one(),
        );

        let corrected = ImuCalibrationEngine::apply(&imu, &calibration);

        assert!((corrected.accel_x - 0.9).abs() < f64::EPSILON);
        assert!((corrected.accel_y - -1.8).abs() < f64::EPSILON);
        assert!((corrected.accel_z - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn gyroscope_bias_is_removed() {
        let imu = sample_imu();

        let calibration = ImuCalibration::new(
            Vector3Calibration::zero(),
            Vector3Calibration::one(),
            Vector3Calibration::new(1.0, -1.0, 0.5),
            Vector3Calibration::one(),
        );

        let corrected = ImuCalibrationEngine::apply(&imu, &calibration);

        assert!((corrected.gyro_x - 9.0).abs() < f64::EPSILON);
        assert!((corrected.gyro_y - -4.0).abs() < f64::EPSILON);
        assert!((corrected.gyro_z - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn accelerometer_scale_is_applied() {
        let imu = sample_imu();

        let calibration = ImuCalibration::new(
            Vector3Calibration::zero(),
            Vector3Calibration::new(2.0, 0.5, -1.0),
            Vector3Calibration::zero(),
            Vector3Calibration::one(),
        );

        let corrected = ImuCalibrationEngine::apply(&imu, &calibration);

        assert!((corrected.accel_x - 2.0).abs() < f64::EPSILON);
        assert!((corrected.accel_y - -1.0).abs() < f64::EPSILON);
        assert!((corrected.accel_z - -0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn gyroscope_scale_is_applied() {
        let imu = sample_imu();

        let calibration = ImuCalibration::new(
            Vector3Calibration::zero(),
            Vector3Calibration::one(),
            Vector3Calibration::zero(),
            Vector3Calibration::new(0.5, 2.0, -1.0),
        );

        let corrected = ImuCalibrationEngine::apply(&imu, &calibration);

        assert!((corrected.gyro_x - 5.0).abs() < f64::EPSILON);
        assert!((corrected.gyro_y - -10.0).abs() < f64::EPSILON);
        assert!((corrected.gyro_z - -2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn bias_is_removed_before_scale_is_applied() {
        let imu = sample_imu();

        let calibration = ImuCalibration::new(
            Vector3Calibration::new(0.5, -1.0, 0.25),
            Vector3Calibration::new(2.0, 3.0, 4.0),
            Vector3Calibration::new(2.0, -1.0, 1.0),
            Vector3Calibration::new(0.5, 2.0, 3.0),
        );

        let corrected = ImuCalibrationEngine::apply(&imu, &calibration);

        assert!((corrected.accel_x - 1.0).abs() < f64::EPSILON);
        assert!((corrected.accel_y - -3.0).abs() < f64::EPSILON);
        assert!((corrected.accel_z - 1.0).abs() < f64::EPSILON);

        assert!((corrected.gyro_x - 4.0).abs() < f64::EPSILON);
        assert!((corrected.gyro_y - -8.0).abs() < f64::EPSILON);
        assert!((corrected.gyro_z - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn temperature_is_preserved() {
        let imu = sample_imu();

        let calibration = ImuCalibration::new(
            Vector3Calibration::new(0.1, 0.1, 0.1),
            Vector3Calibration::new(1.1, 1.1, 1.1),
            Vector3Calibration::new(0.2, 0.2, 0.2),
            Vector3Calibration::new(0.9, 0.9, 0.9),
        );

        let corrected = ImuCalibrationEngine::apply(&imu, &calibration);

        assert_eq!(corrected.temperature, Some(27.5));
    }

    #[test]
    fn original_imu_is_not_modified() {
        let imu = sample_imu();

        let original_accel_x = imu.accel_x;
        let original_gyro_x = imu.gyro_x;
        let original_temperature = imu.temperature;

        let calibration = ImuCalibration::new(
            Vector3Calibration::new(0.5, 0.5, 0.5),
            Vector3Calibration::new(2.0, 2.0, 2.0),
            Vector3Calibration::new(1.0, 1.0, 1.0),
            Vector3Calibration::new(0.5, 0.5, 0.5),
        );

        let _corrected = ImuCalibrationEngine::apply(&imu, &calibration);

        assert_eq!(imu.accel_x, original_accel_x);
        assert_eq!(imu.gyro_x, original_gyro_x);
        assert_eq!(imu.temperature, original_temperature);
    }
}
