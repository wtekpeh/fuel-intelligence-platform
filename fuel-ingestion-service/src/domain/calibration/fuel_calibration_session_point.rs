use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents one physical observation captured during a guided
/// fuel-calibration session.
///
/// A point may initially be unanchored. In that state, ORBI knows:
///
/// - the KUM distance observed at the tank;
/// - how much fuel has been added or removed relative to the
///   beginning of the session.
///
/// Once an absolute anchor becomes available, the point can be
/// resolved into an absolute fuel quantity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelCalibrationSessionPoint {
    pub id: Uuid,

    /// Physical KUM distance measured for this observation.
    pub level_cm: f64,

    /// Signed cumulative fuel change relative to the beginning of
    /// the session.
    ///
    /// Examples:
    ///
    ///  0.0  = initial observation
    /// 20.0  = twenty litres have been added
    /// 40.0  = forty litres have been added
    /// -20.0 = twenty litres have been removed
    pub cumulative_change_litres: f64,

    /// Absolute quantity represented by this point.
    ///
    /// This remains `None` until an absolute calibration anchor has
    /// allowed the session to resolve the observation into litres.
    pub resolved_litres: Option<f64>,

    pub captured_at: DateTime<Utc>,
}

impl FuelCalibrationSessionPoint {
    pub fn validate(&self) -> Result<()> {
        if !self.level_cm.is_finite() || self.level_cm < 0.0 {
            return Err(anyhow!(
                "Calibration point level must be a finite non-negative value."
            ));
        }

        if !self.cumulative_change_litres.is_finite() {
            return Err(anyhow!(
                "Calibration point cumulative fuel change must be finite."
            ));
        }

        if let Some(resolved_litres) = self.resolved_litres {
            if !resolved_litres.is_finite() || resolved_litres < 0.0 {
                return Err(anyhow!(
                    "Calibration point resolved litres must be a finite non-negative value."
                ));
            }
        }

        Ok(())
    }

    pub fn is_resolved(&self) -> bool {
        self.resolved_litres.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_point() -> FuelCalibrationSessionPoint {
        FuelCalibrationSessionPoint {
            id: Uuid::new_v4(),
            level_cm: 18.0,
            cumulative_change_litres: 0.0,
            resolved_litres: None,
            captured_at: Utc::now(),
        }
    }

    #[test]
    fn unresolved_point_is_valid() {
        let point = base_point();

        assert!(point.validate().is_ok());
        assert!(!point.is_resolved());
    }

    #[test]
    fn point_allows_positive_cumulative_fuel_change() {
        let mut point = base_point();

        point.cumulative_change_litres = 40.0;

        assert!(point.validate().is_ok());
    }

    #[test]
    fn point_allows_negative_cumulative_fuel_change() {
        let mut point = base_point();

        point.cumulative_change_litres = -20.0;

        assert!(point.validate().is_ok());
    }

    #[test]
    fn resolved_point_is_valid() {
        let mut point = base_point();

        point.resolved_litres = Some(140.0);

        assert!(point.validate().is_ok());
        assert!(point.is_resolved());
    }

    #[test]
    fn negative_resolved_litres_are_rejected() {
        let mut point = base_point();

        point.resolved_litres = Some(-1.0);

        let error = point
            .validate()
            .expect_err("negative resolved litres should fail");

        assert_eq!(
            error.to_string(),
            "Calibration point resolved litres must be a finite non-negative value."
        );
    }

    #[test]
    fn negative_level_is_rejected() {
        let mut point = base_point();

        point.level_cm = -1.0;

        let error = point.validate().expect_err("negative level should fail");

        assert_eq!(
            error.to_string(),
            "Calibration point level must be a finite non-negative value."
        );
    }

    #[test]
    fn non_finite_cumulative_change_is_rejected() {
        let mut point = base_point();

        point.cumulative_change_litres = f64::NAN;

        let error = point
            .validate()
            .expect_err("non-finite cumulative fuel change should fail");

        assert_eq!(
            error.to_string(),
            "Calibration point cumulative fuel change must be finite."
        );
    }
}
