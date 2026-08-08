use serde::{Deserialize, Serialize};

/// Describes how much of the tank has been verified by guided
/// calibration sessions.
///
/// Coverage is based on verified calibration points rather than
/// estimated values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelCalibrationCoverage {
    /// Lowest verified quantity of fuel.
    pub verified_from_litres: f64,

    /// Highest verified quantity of fuel.
    pub verified_to_litres: f64,

    /// Percentage of the declared tank capacity that has been
    /// verified through calibration.
    pub coverage_percentage: f64,
}

impl FuelCalibrationCoverage {
    pub fn is_complete(&self) -> bool {
        self.coverage_percentage >= 100.0
    }

    pub fn is_partial(&self) -> bool {
        self.coverage_percentage < 100.0
    }
}
