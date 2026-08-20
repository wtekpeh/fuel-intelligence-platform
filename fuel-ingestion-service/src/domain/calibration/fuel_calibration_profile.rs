use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    FuelCalibration, FuelCalibrationConfidence, FuelCalibrationCoverage, FuelCalibrationSession,
    FuelCalibrationStatus,
};

/// Represents the managed calibration profile for one installed
/// fuel sensor and tank combination.
///
/// The profile owns:
///
/// - the currently verified lookup table;
/// - calibration coverage;
/// - lifecycle status;
/// - confidence classification;
/// - the history of guided calibration sessions.
///
/// Interpolation uses `calibration`, while the remaining fields
/// describe how trustworthy and complete that calibration currently is.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelCalibrationProfile {
    /// Unique identifier for this calibration profile.
    pub id: Uuid,

    /// Provisioned fuel-sensor instance that owns the calibration.
    pub sensor_id: Uuid,

    /// Current verified lookup-table calibration.
    pub calibration: FuelCalibration,

    /// Current lifecycle state.
    pub status: FuelCalibrationStatus,

    /// Verified portion of the tank represented by the lookup table.
    pub coverage: FuelCalibrationCoverage,

    /// Business-level confidence classification.
    pub confidence: FuelCalibrationConfidence,

    /// Guided calibration sessions that contributed verified points.
    pub sessions: Vec<FuelCalibrationSession>,

    /// When the profile was first created.
    pub created_at: DateTime<Utc>,

    /// When the profile was most recently updated.
    pub updated_at: DateTime<Utc>,
}

impl FuelCalibrationProfile {
    /// Validates the complete calibration profile.
    ///
    /// The lookup table validates the mathematical calibration data.
    ///
    /// The profile additionally protects lifecycle rules involving:
    ///
    /// - coverage;
    /// - confidence;
    /// - lifecycle status;
    /// - calibration-session history.
    pub fn validate(&self) -> Result<()> {
        const PERCENTAGE_TOLERANCE: f64 = 0.001;

        /*
         * First validate the underlying lookup table.
         */
        self.calibration.validate_lookup_table()?;

        /*
         * Every guided calibration session must be internally valid before
         * the profile can be considered valid.
         *
         * This protects the profile from containing impossible session states,
         * such as:
         *
         * - completed sessions without completion timestamps;
         * - active or paused sessions with completion timestamps;
         * - invalid verified litre ranges;
         * - completed sessions with too few verified points.
         */
        for session in &self.sessions {
            session.validate()?;
        }

        /*
         * Coverage quantities must be finite and remain inside the
         * declared tank-capacity range.
         */
        if !self.coverage.verified_from_litres.is_finite()
            || self.coverage.verified_from_litres < 0.0
            || self.coverage.verified_from_litres > self.calibration.tank_capacity_litres
        {
            return Err(anyhow!(
                "Verified starting quantity must be between zero and the tank capacity."
            ));
        }

        if !self.coverage.verified_to_litres.is_finite()
            || self.coverage.verified_to_litres < 0.0
            || self.coverage.verified_to_litres > self.calibration.tank_capacity_litres
        {
            return Err(anyhow!(
                "Verified ending quantity must be between zero and the tank capacity."
            ));
        }

        if self.coverage.verified_to_litres < self.coverage.verified_from_litres {
            return Err(anyhow!(
                "Verified ending quantity must not be below the verified starting quantity."
            ));
        }

        /*
         * Coverage percentage must be a meaningful percentage.
         */
        if !self.coverage.coverage_percentage.is_finite()
            || self.coverage.coverage_percentage < 0.0
            || self.coverage.coverage_percentage > 100.0
        {
            return Err(anyhow!(
                "Calibration coverage percentage must be between zero and one hundred."
            ));
        }

        /*
         * Recalculate coverage from the verified litre range.
         *
         * The stored percentage must agree with the verified range rather
         * than being accepted blindly from an API request or database row.
         */
        let verified_range_litres =
            self.coverage.verified_to_litres - self.coverage.verified_from_litres;

        let calculated_coverage_percentage =
            verified_range_litres / self.calibration.tank_capacity_litres * 100.0;

        if (self.coverage.coverage_percentage - calculated_coverage_percentage).abs()
            > PERCENTAGE_TOLERANCE
        {
            return Err(anyhow!(
                "Stored calibration coverage does not match the verified fuel range."
            ));
        }

        /*
         * A draft profile must not already contain guided sessions.
         */
        if self.status == FuelCalibrationStatus::Draft && !self.sessions.is_empty() {
            return Err(anyhow!(
                "A draft calibration profile must not contain completed calibration sessions."
            ));
        }

        /*
         * Validated and production profiles require at least one completed
         * guided calibration session.
         */
        if matches!(
            self.status,
            FuelCalibrationStatus::Validated | FuelCalibrationStatus::Production
        ) && !self
            .sessions
            .iter()
            .any(FuelCalibrationSession::is_completed)
        {
            return Err(anyhow!(
                "Validated and production calibrations require at least one completed session."
            ));
        }

        /*
         * A production calibration cannot still be classified as low
         * confidence.
         */
        if self.status == FuelCalibrationStatus::Production
            && self.confidence == FuelCalibrationConfidence::Low
        {
            return Err(anyhow!(
                "A production calibration cannot have low confidence."
            ));
        }

        /*
         * A profile marked fully verified must actually cover the full
         * declared tank range.
         */
        if self.confidence == FuelCalibrationConfidence::Verified && !self.coverage.is_complete() {
            return Err(anyhow!(
                "Verified calibration confidence requires complete tank coverage."
            ));
        }

        Ok(())
    }
    /// Returns the number of guided calibration sessions currently
    /// associated with the profile.
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Returns true when the profile has been approved for normal
    /// production use.
    pub fn is_production_ready(&self) -> bool {
        self.status == FuelCalibrationStatus::Production
    }

    /// Returns true when further verified calibration coverage is needed.
    pub fn requires_progressive_calibration(&self) -> bool {
        self.coverage.is_partial()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_calibration() -> FuelCalibration {
        FuelCalibration {
            tank_capacity_litres: 200.0,
            points: vec![
                super::super::FuelCalibrationPoint {
                    level_cm: 10.0,
                    litres: 20.0,
                },
                super::super::FuelCalibrationPoint {
                    level_cm: 100.0,
                    litres: 200.0,
                },
            ],
        }
    }

    fn paused_session() -> FuelCalibrationSession {
        FuelCalibrationSession {
            id: Uuid::new_v4(),
            started_at: Utc::now(),
            completed_at: None,
            status: super::super::FuelCalibrationSessionStatus::Paused,
            starting_litres: 20.0,
            ending_litres: 80.0,
            captured_point_count: 1,
        }
    }

    fn completed_session() -> FuelCalibrationSession {
        let started_at = Utc::now();

        FuelCalibrationSession {
            id: Uuid::new_v4(),
            started_at,
            completed_at: Some(started_at + chrono::Duration::minutes(10)),
            status: super::super::FuelCalibrationSessionStatus::Completed,
            starting_litres: 20.0,
            ending_litres: 80.0,
            captured_point_count: 2,
        }
    }

    fn progressive_profile() -> FuelCalibrationProfile {
        FuelCalibrationProfile {
            id: Uuid::new_v4(),
            sensor_id: Uuid::new_v4(),
            calibration: valid_calibration(),
            status: FuelCalibrationStatus::Progressive,
            coverage: FuelCalibrationCoverage {
                verified_from_litres: 20.0,
                verified_to_litres: 80.0,
                coverage_percentage: 30.0,
            },
            confidence: FuelCalibrationConfidence::Medium,
            sessions: vec![paused_session()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn progressive_profile_allows_valid_paused_session() {
        let profile = progressive_profile();

        assert!(profile.validate().is_ok());
        assert!(profile.requires_progressive_calibration());
    }

    #[test]
    fn invalid_session_makes_profile_invalid() {
        let mut profile = progressive_profile();

        profile.sessions[0].completed_at = Some(Utc::now());

        let error = profile
            .validate()
            .expect_err("invalid session should invalidate profile");

        assert_eq!(
            error.to_string(),
            "A paused calibration session must not have a completion time."
        );
    }

    #[test]
    fn validated_profile_requires_completed_session() {
        let mut profile = progressive_profile();
        profile.status = FuelCalibrationStatus::Validated;

        let error = profile
            .validate()
            .expect_err("validated profile without completed session should fail");

        assert_eq!(
            error.to_string(),
            "Validated and production calibrations require at least one completed session."
        );
    }

    #[test]
    fn validated_profile_accepts_completed_session() {
        let mut profile = progressive_profile();

        profile.status = FuelCalibrationStatus::Validated;
        profile.sessions = vec![completed_session()];

        assert!(profile.validate().is_ok());
    }

    #[test]
    fn production_profile_rejects_low_confidence() {
        let mut profile = progressive_profile();

        profile.status = FuelCalibrationStatus::Production;
        profile.confidence = FuelCalibrationConfidence::Low;
        profile.sessions = vec![completed_session()];

        let error = profile
            .validate()
            .expect_err("production profile with low confidence should fail");

        assert_eq!(
            error.to_string(),
            "A production calibration cannot have low confidence."
        );
    }

    #[test]
    fn verified_confidence_requires_complete_coverage() {
        let mut profile = progressive_profile();

        profile.confidence = FuelCalibrationConfidence::Verified;

        let error = profile
            .validate()
            .expect_err("verified confidence with partial coverage should fail");

        assert_eq!(
            error.to_string(),
            "Verified calibration confidence requires complete tank coverage."
        );
    }
}
