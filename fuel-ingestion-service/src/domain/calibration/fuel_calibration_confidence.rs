use serde::{Deserialize, Serialize};

/// Represents ORBI's confidence in the current fuel calibration.
///
/// Confidence increases as the calibration gains verified coverage
/// and additional guided calibration sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FuelCalibrationConfidence {
    /// Very little verified data.
    Low,

    /// Partial verified coverage.
    Medium,

    /// Sufficient verified coverage for normal production use.
    High,

    /// Complete verified coverage with excellent confidence.
    Verified,
}
