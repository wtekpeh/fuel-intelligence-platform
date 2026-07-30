/// Statistical summary calculated from a completed collection
/// of operational behaviour samples.
///
/// This structure is independent of persistence. It represents
/// the complete statistical result produced by the domain-level
/// behaviour profile builder.
#[derive(Debug, Clone, PartialEq)]
pub struct BehaviourProfileStatistics {
    /// Number of behaviour samples used in the calculation.
    pub sample_count: usize,

    /// Arithmetic mean of all vibration scores.
    pub average_vibration_score: f64,

    /// Lowest observed vibration score.
    pub minimum_vibration_score: f64,

    /// Highest observed vibration score.
    pub maximum_vibration_score: f64,

    /// Population variance of the observed vibration scores.
    pub vibration_variance: f64,

    /// Population standard deviation of the observed
    /// vibration scores.
    pub vibration_standard_deviation: f64,

    /// Arithmetic mean of all motion ratios.
    pub average_motion_ratio: f64,

    /// Lowest observed motion ratio.
    pub minimum_motion_ratio: f64,

    /// Highest observed motion ratio.
    pub maximum_motion_ratio: f64,

    /// Arithmetic mean of all movement-confidence values.
    pub average_confidence: f64,

    /// Fraction of samples whose rolling evidence reported
    /// sustained motion.
    ///
    /// The value is always expected to be in the inclusive
    /// range 0.0 to 1.0.
    pub sustained_motion_ratio: f64,

    /// Arithmetic mean of available GPS speed measurements.
    ///
    /// This is `None` when none of the collected behaviour
    /// samples contains GPS speed.
    pub average_gps_speed_kmh: Option<f64>,
}

impl BehaviourProfileStatistics {
    /// Returns the vibration interval observed during learning.
    pub fn vibration_range(&self) -> f64 {
        self.maximum_vibration_score - self.minimum_vibration_score
    }

    /// Returns the motion-ratio interval observed during learning.
    pub fn motion_ratio_range(&self) -> f64 {
        self.maximum_motion_ratio - self.minimum_motion_ratio
    }

    /// Returns true when at least half of the collected samples
    /// reported sustained motion.
    pub fn has_predominantly_sustained_motion(&self) -> bool {
        self.sustained_motion_ratio >= 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn statistics() -> BehaviourProfileStatistics {
        BehaviourProfileStatistics {
            sample_count: 30,
            average_vibration_score: 3.0,
            minimum_vibration_score: 1.0,
            maximum_vibration_score: 5.0,
            vibration_variance: 2.0,
            vibration_standard_deviation: 2.0_f64.sqrt(),
            average_motion_ratio: 0.6,
            minimum_motion_ratio: 0.2,
            maximum_motion_ratio: 0.9,
            average_confidence: 0.8,
            sustained_motion_ratio: 0.7,
            average_gps_speed_kmh: Some(25.0),
        }
    }

    #[test]
    fn calculates_vibration_range() {
        let statistics = statistics();

        assert_eq!(statistics.vibration_range(), 4.0);
    }

    #[test]
    fn calculates_motion_ratio_range() {
        let statistics = statistics();

        assert!((statistics.motion_ratio_range() - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn detects_predominantly_sustained_motion() {
        let statistics = statistics();

        assert!(statistics.has_predominantly_sustained_motion());
    }

    #[test]
    fn rejects_minor_sustained_motion_as_predominant() {
        let mut statistics = statistics();
        statistics.sustained_motion_ratio = 0.49;

        assert!(!statistics.has_predominantly_sustained_motion());
    }
}
