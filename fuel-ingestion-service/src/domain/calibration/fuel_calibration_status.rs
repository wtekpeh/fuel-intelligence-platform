use serde::{Deserialize, Serialize};

/// Represents the lifecycle of a fuel calibration profile.
///
/// A calibration becomes more complete over time as additional
/// verified refuelling sessions are performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FuelCalibrationStatus {
    /// Calibration has been created but contains no verified points.
    Draft,

    /// Calibration is active but only part of the tank has been verified.
    Progressive,

    /// The calibration covers the intended operating range and has
    /// passed validation.
    Validated,

    /// Calibration has been approved for production use.
    Production,

    /// Replaced by a newer calibration profile.
    Superseded,
}
