use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::FuelCalibrationSessionStatus;

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

    /// Current lifecycle state of this guided calibration session.
    pub status: FuelCalibrationSessionStatus,

    /// Fuel quantity at the beginning of this session.
    pub starting_litres: f64,

    /// Fuel quantity when the session ended.
    pub ending_litres: f64,

    /// Number of verified calibration points captured.
    pub captured_point_count: usize,
}

impl FuelCalibrationSession {
    /// Returns the verified litres covered during this session.
    pub fn verified_range(&self) -> f64 {
        (self.ending_litres - self.starting_litres).max(0.0)
    }

    pub fn is_active(&self) -> bool {
        self.status == FuelCalibrationSessionStatus::Active
    }

    pub fn is_paused(&self) -> bool {
        self.status == FuelCalibrationSessionStatus::Paused
    }

    pub fn is_completed(&self) -> bool {
        self.status == FuelCalibrationSessionStatus::Completed && self.completed_at.is_some()
    }

    /// Validates the lifecycle and verified range of this guided
    /// calibration session.
    pub fn validate(&self) -> Result<()> {
        if !self.starting_litres.is_finite() || self.starting_litres < 0.0 {
            return Err(anyhow!(
                "Calibration session starting litres must be a finite non-negative value."
            ));
        }

        if !self.ending_litres.is_finite() || self.ending_litres < 0.0 {
            return Err(anyhow!(
                "Calibration session ending litres must be a finite non-negative value."
            ));
        }

        if self.ending_litres < self.starting_litres {
            return Err(anyhow!(
                "Calibration session ending litres must not be below starting litres."
            ));
        }

        if let Some(completed_at) = self.completed_at
            && completed_at < self.started_at
        {
            return Err(anyhow!(
                "Calibration session completion time must not be before its start time."
            ));
        }

        match self.status {
            FuelCalibrationSessionStatus::Active => {
                if self.completed_at.is_some() {
                    return Err(anyhow!(
                        "An active calibration session must not have a completion time."
                    ));
                }
            }

            FuelCalibrationSessionStatus::Paused => {
                if self.completed_at.is_some() {
                    return Err(anyhow!(
                        "A paused calibration session must not have a completion time."
                    ));
                }
            }

            FuelCalibrationSessionStatus::Completed => {
                if self.completed_at.is_none() {
                    return Err(anyhow!(
                        "A completed calibration session must have a completion time."
                    ));
                }

                if self.captured_point_count < 2 {
                    return Err(anyhow!(
                        "A completed calibration session must contain at least two verified points."
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn base_session(status: FuelCalibrationSessionStatus) -> FuelCalibrationSession {
        let started_at = Utc::now();

        FuelCalibrationSession {
            id: Uuid::new_v4(),
            started_at,
            completed_at: None,
            status,
            starting_litres: 20.0,
            ending_litres: 40.0,
            captured_point_count: 1,
        }
    }

    #[test]
    fn active_session_without_completion_time_is_valid() {
        let session = base_session(FuelCalibrationSessionStatus::Active);

        assert!(session.validate().is_ok());
        assert!(session.is_active());
        assert!(!session.is_paused());
        assert!(!session.is_completed());
    }

    #[test]
    fn paused_session_without_completion_time_is_valid() {
        let session = base_session(FuelCalibrationSessionStatus::Paused);

        assert!(session.validate().is_ok());
        assert!(!session.is_active());
        assert!(session.is_paused());
        assert!(!session.is_completed());
    }

    #[test]
    fn completed_session_requires_completion_time() {
        let mut session = base_session(FuelCalibrationSessionStatus::Completed);
        session.captured_point_count = 2;

        let error = session
            .validate()
            .expect_err("completed session without completion time should fail");

        assert_eq!(
            error.to_string(),
            "A completed calibration session must have a completion time."
        );
    }

    #[test]
    fn completed_session_requires_at_least_two_verified_points() {
        let mut session = base_session(FuelCalibrationSessionStatus::Completed);
        session.completed_at = Some(session.started_at + Duration::minutes(10));

        let error = session
            .validate()
            .expect_err("completed session with fewer than two points should fail");

        assert_eq!(
            error.to_string(),
            "A completed calibration session must contain at least two verified points."
        );
    }

    #[test]
    fn completed_session_with_two_points_is_valid() {
        let mut session = base_session(FuelCalibrationSessionStatus::Completed);
        session.completed_at = Some(session.started_at + Duration::minutes(10));
        session.captured_point_count = 2;

        assert!(session.validate().is_ok());
        assert!(session.is_completed());
    }

    #[test]
    fn active_session_must_not_have_completion_time() {
        let mut session = base_session(FuelCalibrationSessionStatus::Active);
        session.completed_at = Some(session.started_at + Duration::minutes(10));

        let error = session
            .validate()
            .expect_err("active session with completion time should fail");

        assert_eq!(
            error.to_string(),
            "An active calibration session must not have a completion time."
        );
    }

    #[test]
    fn paused_session_must_not_have_completion_time() {
        let mut session = base_session(FuelCalibrationSessionStatus::Paused);
        session.completed_at = Some(session.started_at + Duration::minutes(10));

        let error = session
            .validate()
            .expect_err("paused session with completion time should fail");

        assert_eq!(
            error.to_string(),
            "A paused calibration session must not have a completion time."
        );
    }

    #[test]
    fn ending_litres_must_not_be_below_starting_litres() {
        let mut session = base_session(FuelCalibrationSessionStatus::Active);
        session.starting_litres = 50.0;
        session.ending_litres = 40.0;

        let error = session
            .validate()
            .expect_err("ending litres below starting litres should fail");

        assert_eq!(
            error.to_string(),
            "Calibration session ending litres must not be below starting litres."
        );
    }

    #[test]
    fn completion_time_must_not_be_before_start_time() {
        let mut session = base_session(FuelCalibrationSessionStatus::Completed);
        session.completed_at = Some(session.started_at - Duration::minutes(1));
        session.captured_point_count = 2;

        let error = session
            .validate()
            .expect_err("completion before start should fail");

        assert_eq!(
            error.to_string(),
            "Calibration session completion time must not be before its start time."
        );
    }

    #[test]
    fn verified_range_returns_positive_session_coverage() {
        let session = base_session(FuelCalibrationSessionStatus::Active);

        assert_eq!(session.verified_range(), 20.0);
    }
}
