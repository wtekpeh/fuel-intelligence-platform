use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents one guided calibration session.
///
/// A profile may contain multiple sessions accumulated over the
/// lifetime of the vehicle. Each session contributes additional
/// verified calibration points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelCalibrationSession {
    /// Unique session identifier.
    pub id: Uuid,

    /// When the guided calibration began.
    pub started_at: DateTime<Utc>,

    /// When the session finished.
    pub completed_at: Option<DateTime<Utc>>,

    /// Fuel quantity at the beginning of this session.
    pub starting_litres: f64,

    /// Fuel quantity when the session ended.
    pub ending_litres: f64,

    /// Number of verified calibration points captured.
    pub captured_point_count: usize,
}

impl FuelCalibrationSession {
    /// Returns true when the calibration session has completed.
    pub fn is_completed(&self) -> bool {
        self.completed_at.is_some()
    }

    /// Returns the verified litres covered during this session.
    pub fn verified_range(&self) -> f64 {
        (self.ending_litres - self.starting_litres).max(0.0)
    }
}
