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
