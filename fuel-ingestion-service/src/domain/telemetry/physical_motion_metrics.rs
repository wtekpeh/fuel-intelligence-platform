/// Physical motion measurements derived directly from one IMU reading.
///
/// These values are calculated before operational deadbands, thresholds,
/// classifications, or confidence rules are applied.
///
/// Keeping them separate from `ImuInterpretation` allows behaviour learning
/// to analyse the real physical signal even when the operational vibration
/// score is clamped to zero.
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicalMotionMetrics {
    /// Magnitude of the three-axis accelerometer vector, measured in g.
    pub acceleration_magnitude_g: f64,

    /// Absolute difference between acceleration magnitude and normal gravity.
    ///
    /// This value is preserved before the acceleration noise deadband is
    /// applied.
    pub gravity_deviation_g: f64,

    /// Magnitude of the three-axis gyroscope vector, measured in degrees
    /// per second.
    ///
    /// This value is preserved before the gyroscope noise deadband is
    /// applied.
    pub rotation_magnitude_dps: f64,
}

impl PhysicalMotionMetrics {
    pub fn new(
        acceleration_magnitude_g: f64,
        gravity_deviation_g: f64,
        rotation_magnitude_dps: f64,
    ) -> Self {
        Self {
            acceleration_magnitude_g,
            gravity_deviation_g,
            rotation_magnitude_dps,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_physical_motion_metrics() {
        let metrics = PhysicalMotionMetrics::new(1.06, 0.06, 2.5);

        assert_eq!(metrics.acceleration_magnitude_g, 1.06);
        assert_eq!(metrics.gravity_deviation_g, 0.06);
        assert_eq!(metrics.rotation_magnitude_dps, 2.5);
    }
}
