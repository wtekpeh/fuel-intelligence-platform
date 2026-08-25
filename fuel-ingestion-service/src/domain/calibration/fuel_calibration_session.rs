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

    /// Absolute fuel quantity at the beginning of the session.
    ///
    /// This may be unknown when calibration begins. For example, an
    /// installer may arrive at a partially filled tank without knowing
    /// the exact quantity currently present.
    pub starting_litres: Option<f64>,

    /// Absolute fuel quantity at the end of the session.
    ///
    /// This remains unknown until the session obtains an absolute
    /// calibration anchor such as:
    ///
    /// - confirmed empty tank;
    /// - confirmed full tank;
    /// - independently measured fuel quantity.
    pub ending_litres: Option<f64>,

    /// Number of verified calibration points captured.
    pub captured_point_count: usize,
}

impl FuelCalibrationSession {
    /// Returns the verified litres covered during this session.
    pub fn verified_range(&self) -> Option<f64> {
        match (self.starting_litres, self.ending_litres) {
            (Some(starting_litres), Some(ending_litres)) => {
                /*
                 * Guided calibration may proceed in either direction:
                 *
                 * filling:
                 *     20 L -> 80 L
                 *
                 * draining:
                 *     80 L -> 20 L
                 *
                 * Coverage is the absolute quantity traversed.
                 */
                Some((ending_litres - starting_litres).abs())
            }

            _ => None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.status == FuelCalibrationSessionStatus::Active
    }

    pub fn is_paused(&self) -> bool {
        self.status == FuelCalibrationSessionStatus::Paused
    }

    pub fn is_abandoned(&self) -> bool {
        self.status == FuelCalibrationSessionStatus::Abandoned
    }

    pub fn is_completed(&self) -> bool {
        self.status == FuelCalibrationSessionStatus::Completed && self.completed_at.is_some()
    }

    /// Validates the lifecycle and verified range of this guided
    /// calibration session.
    pub fn validate(&self) -> Result<()> {
        /*
         * Starting and ending absolute quantities are optional while a
         * calibration session is active or paused.
         *
         * When present, however, they must always contain physically
         * meaningful values.
         */
        if let Some(starting_litres) = self.starting_litres {
            if !starting_litres.is_finite() || starting_litres < 0.0 {
                return Err(anyhow!(
                    "Calibration session starting litres must be a finite non-negative value."
                ));
            }
        }

        if let Some(ending_litres) = self.ending_litres {
            if !ending_litres.is_finite() || ending_litres < 0.0 {
                return Err(anyhow!(
                    "Calibration session ending litres must be a finite non-negative value."
                ));
            }
        }

        /*
         * A completion timestamp, whenever present, must not precede
         * the beginning of the calibration session.
         */
        if let Some(completed_at) = self.completed_at {
            if completed_at < self.started_at {
                return Err(anyhow!(
                    "Calibration session completion time must not be before its start time."
                ));
            }
        }

        /*
         * Validate lifecycle-specific rules.
         */
        match self.status {
            FuelCalibrationSessionStatus::Active => {
                /*
                 * An active session is still collecting evidence.
                 *
                 * Its absolute starting and ending fuel quantities may
                 * legitimately still be unknown.
                 */
                if self.completed_at.is_some() {
                    return Err(anyhow!(
                        "An active calibration session must not have a completion time."
                    ));
                }
            }

            FuelCalibrationSessionStatus::Paused => {
                /*
                 * A paused session remains unfinished and may be resumed
                 * later.
                 *
                 * Its absolute fuel quantities may also still be unresolved.
                 */
                if self.completed_at.is_some() {
                    return Err(anyhow!(
                        "A paused calibration session must not have a completion time."
                    ));
                }
            }

            FuelCalibrationSessionStatus::Abandoned => {
                /*
                 * An abandoned session is permanently unfinished.
                 *
                 * It remains stored for historical/audit purposes but does not
                 * receive a completion timestamp and does not become verified
                 * calibration evidence.
                 */
                if self.completed_at.is_some() {
                    return Err(anyhow!(
                        "An abandoned calibration session must not have a completion time."
                    ));
                }
            }

            FuelCalibrationSessionStatus::Completed => {
                /*
                 * A completed session must have an explicit completion
                 * timestamp.
                 */
                if self.completed_at.is_none() {
                    return Err(anyhow!(
                        "A completed calibration session must have a completion time."
                    ));
                }

                /*
                 * By completion time, an absolute calibration anchor must
                 * have resolved both ends of the session into litres.
                 */
                if self.starting_litres.is_none() || self.ending_litres.is_none() {
                    return Err(anyhow!(
                        "A completed calibration session must have resolved starting and ending litres."
                    ));
                }

                /*
                 * One observation cannot define a usable calibration range.
                 */
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
            starting_litres: Some(20.0),
            ending_litres: Some(40.0),
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
    fn draining_session_allows_ending_litres_below_starting_litres() {
        let mut session = base_session(FuelCalibrationSessionStatus::Active);

        session.starting_litres = Some(50.0);
        session.ending_litres = Some(20.0);

        assert!(session.validate().is_ok());
        assert_eq!(session.verified_range(), Some(30.0));
    }

    #[test]
    fn completed_draining_session_is_valid() {
        let mut session = base_session(FuelCalibrationSessionStatus::Completed);

        session.starting_litres = Some(100.0);
        session.ending_litres = Some(0.0);

        session.completed_at = Some(session.started_at + Duration::minutes(10));

        session.captured_point_count = 2;

        assert!(session.validate().is_ok());
        assert!(session.is_completed());
        assert_eq!(session.verified_range(), Some(100.0));
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

        assert_eq!(session.verified_range(), Some(20.0));
    }

    #[test]
    fn active_session_allows_unknown_starting_and_ending_litres() {
        let mut session = base_session(FuelCalibrationSessionStatus::Active);

        session.starting_litres = None;
        session.ending_litres = None;

        assert!(session.validate().is_ok());
        assert_eq!(session.verified_range(), None);
    }

    #[test]
    fn paused_session_allows_unknown_starting_and_ending_litres() {
        let mut session = base_session(FuelCalibrationSessionStatus::Paused);

        session.starting_litres = None;
        session.ending_litres = None;

        assert!(session.validate().is_ok());
        assert_eq!(session.verified_range(), None);
    }

    #[test]
    fn completed_session_requires_resolved_litre_values() {
        let mut session = base_session(FuelCalibrationSessionStatus::Completed);

        session.completed_at = Some(session.started_at + Duration::minutes(10));
        session.captured_point_count = 2;

        session.starting_litres = None;
        session.ending_litres = None;

        let error = session
            .validate()
            .expect_err("completed session with unresolved litres should fail");

        assert_eq!(
            error.to_string(),
            "A completed calibration session must have resolved starting and ending litres."
        );
    }

    #[test]
    fn abandoned_session_without_completion_time_is_valid() {
        let session = base_session(FuelCalibrationSessionStatus::Abandoned);

        assert!(session.validate().is_ok());

        assert!(!session.is_active());
        assert!(!session.is_paused());
        assert!(session.is_abandoned());
        assert!(!session.is_completed());
    }

    #[test]
    fn abandoned_session_must_not_have_completion_time() {
        let mut session = base_session(FuelCalibrationSessionStatus::Abandoned);

        session.completed_at = Some(session.started_at + Duration::minutes(10));

        let error = session
            .validate()
            .expect_err("abandoned session with completion time should fail");

        assert_eq!(
            error.to_string(),
            "An abandoned calibration session must not have a completion time."
        );
    }

    #[test]
    fn abandoned_session_allows_unknown_litre_values() {
        let mut session = base_session(FuelCalibrationSessionStatus::Abandoned);

        session.starting_litres = None;
        session.ending_litres = None;

        assert!(session.validate().is_ok());
        assert_eq!(session.verified_range(), None);
        assert!(session.is_abandoned());
    }
}
