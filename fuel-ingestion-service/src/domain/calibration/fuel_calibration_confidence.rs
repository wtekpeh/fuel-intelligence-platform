use serde::{Deserialize, Serialize};

/// Represents ORBI's confidence in the current fuel calibration.
///
/// Confidence is derived from the percentage of the declared tank
/// capacity that has been physically verified through guided
/// calibration.
///
/// Because the input is a percentage rather than an absolute litre
/// quantity, the same confidence policy scales naturally across
/// different tank sizes.
///
/// Current policy:
///
/// - 0%   .. <25%  -> Low
/// - 25%  .. <75%  -> Medium
/// - 75%  .. <100% -> High
/// - 100%           -> Verified
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FuelCalibrationConfidence {
    /// Less than 25% of the declared tank capacity has been verified.
    Low,

    /// At least 25%, but less than 75%, has been verified.
    Medium,

    /// At least 75%, but less than the complete tank range, has been verified.
    High,

    /// The complete declared tank range has been physically verified.
    Verified,
}

impl FuelCalibrationConfidence {
    /// Derives calibration confidence from verified tank coverage.
    ///
    /// `coverage_percentage` is already normalized against the declared
    /// tank capacity, so confidence does not depend on absolute tank size.
    ///
    /// Examples:
    ///
    /// - 50 verified litres of a 200 L tank = 25% -> Medium
    /// - 50 verified litres of a 60 L tank  = 83.33% -> High
    /// - complete verified coverage         = 100% -> Verified
    pub fn from_coverage_percentage(coverage_percentage: f64) -> Self {
        if coverage_percentage >= 100.0 {
            Self::Verified
        } else if coverage_percentage >= 75.0 {
            Self::High
        } else if coverage_percentage >= 25.0 {
            Self::Medium
        } else {
            Self::Low
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_coverage_is_low_confidence() {
        assert_eq!(
            FuelCalibrationConfidence::from_coverage_percentage(0.0),
            FuelCalibrationConfidence::Low
        );
    }

    #[test]
    fn coverage_below_twenty_five_percent_is_low_confidence() {
        assert_eq!(
            FuelCalibrationConfidence::from_coverage_percentage(24.99),
            FuelCalibrationConfidence::Low
        );
    }

    #[test]
    fn twenty_five_percent_coverage_is_medium_confidence() {
        assert_eq!(
            FuelCalibrationConfidence::from_coverage_percentage(25.0),
            FuelCalibrationConfidence::Medium
        );
    }

    #[test]
    fn coverage_below_seventy_five_percent_is_medium_confidence() {
        assert_eq!(
            FuelCalibrationConfidence::from_coverage_percentage(74.99),
            FuelCalibrationConfidence::Medium
        );
    }

    #[test]
    fn seventy_five_percent_coverage_is_high_confidence() {
        assert_eq!(
            FuelCalibrationConfidence::from_coverage_percentage(75.0),
            FuelCalibrationConfidence::High
        );
    }

    #[test]
    fn partial_coverage_below_one_hundred_percent_is_high_confidence() {
        assert_eq!(
            FuelCalibrationConfidence::from_coverage_percentage(99.99),
            FuelCalibrationConfidence::High
        );
    }

    #[test]
    fn complete_coverage_is_verified_confidence() {
        assert_eq!(
            FuelCalibrationConfidence::from_coverage_percentage(100.0),
            FuelCalibrationConfidence::Verified
        );
    }

    #[test]
    fn real_test_tank_coverage_is_high_confidence() {
        let tank_capacity_litres: f64 = 1.5;
        let verified_from_litres: f64 = 0.25;
        let verified_to_litres: f64 = 1.5;

        let verified_range_litres = verified_to_litres - verified_from_litres;

        let coverage_percentage = verified_range_litres / tank_capacity_litres * 100.0;

        assert!((coverage_percentage - 83.33333333333334).abs() < 0.000_001);

        assert_eq!(
            FuelCalibrationConfidence::from_coverage_percentage(coverage_percentage),
            FuelCalibrationConfidence::High
        );
    }
}
