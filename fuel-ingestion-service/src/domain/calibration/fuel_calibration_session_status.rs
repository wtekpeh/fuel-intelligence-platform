use serde::{Deserialize, Serialize};

/// Represents the lifecycle state of one guided fuel-calibration session.
///
/// This is separate from `FuelCalibrationStatus`, which represents the
/// lifecycle of the overall calibration profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FuelCalibrationSessionStatus {
    /// The installer is currently capturing verified calibration points.
    Active,

    /// The session has been intentionally stopped and can be resumed later.
    Paused,

    /// The calibration attempt has been intentionally abandoned.
    ///
    /// The session remains available for audit/history but must not:
    ///
    /// - collect additional points;
    /// - be resumed;
    /// - be completed;
    /// - contribute verified profile coverage.
    Abandoned,

    /// The session has been completed and its verified points can
    /// contribute to calibration-profile coverage.
    Completed,
}
