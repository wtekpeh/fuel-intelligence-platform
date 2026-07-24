use crate::domain::telemetry::models::ImuTelemetry;

/// Small accelerometer deviations can come from sensor noise,
/// mounting tolerances, temperature, and calibration differences.
///
/// Acceleration magnitude must exceed this deviation from gravity
/// before it contributes to the vibration score.
const ACCEL_NOISE_DEADBAND_G: f64 = 0.08;

/// Small gyroscope measurements can occur while the device is
/// physically stationary.
///
/// Gyroscope magnitude must exceed this value before it contributes
/// to the vibration score.
const GYRO_NOISE_DEADBAND_DPS: f64 = 3.0;

/// The interpreted vibration score is constrained to a stable
/// platform-wide range.
const MAX_VIBRATION_SCORE: f64 = 10.0;

/// A first-pass motion threshold.
///
/// This is an initial engineering threshold and must later be
/// calibrated using measurements from an installed vehicle.
const MOTION_SCORE_THRESHOLD: f64 = 2.0;

/// Hardware-independent interpretation of one IMU measurement.
///
/// These values are derived by the backend from raw accelerometer
/// and gyroscope measurements. Firmware must not calculate them.
#[derive(Debug, Clone, PartialEq)]
pub struct ImuInterpretation {
    /// Magnitude of the three-axis accelerometer vector.
    pub acceleration_magnitude_g: f64,

    /// Difference between acceleration magnitude and normal gravity.
    pub dynamic_acceleration_g: f64,

    /// Magnitude of the three-axis gyroscope vector.
    pub rotation_magnitude_dps: f64,

    /// Normalized vibration metric between 0.0 and 10.0.
    pub vibration_score: f64,

    /// Backend-derived indication that meaningful motion exists.
    pub motion_detected: bool,

    /// Initial confidence estimate between 0.0 and 1.0.
    pub movement_confidence: f64,
}

/// Interprets raw IMU measurements into hardware-independent
/// operational motion evidence.
///
/// This function does not classify the complete device state.
/// GPS and device health are evaluated separately by the device-state
/// classifier.
pub fn interpret_imu(imu: &ImuTelemetry) -> ImuInterpretation {
    let acceleration_magnitude_g = vector_magnitude(imu.accel_x, imu.accel_y, imu.accel_z);

    let rotation_magnitude_dps = vector_magnitude(imu.gyro_x, imu.gyro_y, imu.gyro_z);

    // A stationary accelerometer normally measures approximately 1 g
    // because of gravity, regardless of its mounting orientation.
    let gravity_deviation_g = (acceleration_magnitude_g - 1.0).abs();

    // Remove expected sensor noise before generating operational evidence.
    let dynamic_acceleration_g = (gravity_deviation_g - ACCEL_NOISE_DEADBAND_G).max(0.0);

    let effective_rotation_dps = (rotation_magnitude_dps - GYRO_NOISE_DEADBAND_DPS).max(0.0);

    // Convert the two different physical quantities into one bounded
    // compatibility score.
    //
    // These weights are intentionally centralized here so that they can
    // later be calibrated without changing the device-state classifier.
    let vibration_score = (dynamic_acceleration_g * 20.0 + effective_rotation_dps / 10.0)
        .clamp(0.0, MAX_VIBRATION_SCORE);

    let motion_detected = vibration_score >= MOTION_SCORE_THRESHOLD;

    let movement_confidence = (vibration_score / 4.0).clamp(0.0, 1.0);

    ImuInterpretation {
        acceleration_magnitude_g,
        dynamic_acceleration_g,
        rotation_magnitude_dps,
        vibration_score,
        motion_detected,
        movement_confidence,
    }
}

fn vector_magnitude(x: f64, y: f64, z: f64) -> f64 {
    (x.powi(2) + y.powi(2) + z.powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stationary_imu_is_not_interpreted_as_motion() {
        let imu = ImuTelemetry {
            accel_x: 0.0,
            accel_y: 0.0,
            accel_z: 1.0,

            gyro_x: 0.0,
            gyro_y: 0.0,
            gyro_z: 0.0,

            temperature: Some(27.0),
        };

        let interpretation = interpret_imu(&imu);

        assert_eq!(interpretation.vibration_score, 0.0);
        assert!(!interpretation.motion_detected);
        assert_eq!(interpretation.movement_confidence, 0.0);
    }

    #[test]
    fn device_orientation_does_not_create_false_motion() {
        let imu = ImuTelemetry {
            accel_x: 1.0,
            accel_y: 0.0,
            accel_z: 0.0,

            gyro_x: 0.0,
            gyro_y: 0.0,
            gyro_z: 0.0,

            temperature: Some(27.0),
        };

        let interpretation = interpret_imu(&imu);

        assert_eq!(interpretation.vibration_score, 0.0);
        assert!(!interpretation.motion_detected);
    }

    #[test]
    fn significant_acceleration_is_interpreted_as_motion() {
        let imu = ImuTelemetry {
            accel_x: 0.8,
            accel_y: 0.8,
            accel_z: 0.8,

            gyro_x: 8.0,
            gyro_y: 4.0,
            gyro_z: 3.0,

            temperature: Some(27.0),
        };

        let interpretation = interpret_imu(&imu);

        assert!(interpretation.vibration_score >= MOTION_SCORE_THRESHOLD);
        assert!(interpretation.motion_detected);
        assert!(interpretation.movement_confidence > 0.0);
    }

    #[test]
    fn vibration_score_is_limited_to_ten() {
        let imu = ImuTelemetry {
            accel_x: 10.0,
            accel_y: 10.0,
            accel_z: 10.0,

            gyro_x: 500.0,
            gyro_y: 500.0,
            gyro_z: 500.0,

            temperature: Some(27.0),
        };

        let interpretation = interpret_imu(&imu);

        assert_eq!(interpretation.vibration_score, MAX_VIBRATION_SCORE);
        assert_eq!(interpretation.movement_confidence, 1.0);
        assert!(interpretation.motion_detected);
    }
}
