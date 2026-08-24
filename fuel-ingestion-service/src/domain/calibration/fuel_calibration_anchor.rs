use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Describes an absolute fuel-quantity reference used to resolve
/// previously captured relative calibration observations.
///
/// Examples:
///
/// - Empty tank: 0 litres.
/// - Full tank: declared tank capacity.
/// - Independently measured quantity: e.g. 120 litres confirmed
///   by a calibrated dispenser or another trusted measurement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelCalibrationAnchor {
    /// Cumulative fuel change, relative to the session start,
    /// at the moment this absolute quantity was established.
    pub cumulative_change_litres: f64,

    /// Trusted absolute fuel quantity at this anchor.
    pub absolute_litres: f64,

    /// When this anchor was established.
    pub established_at: DateTime<Utc>,
}

impl FuelCalibrationAnchor {
    pub fn validate(&self, tank_capacity_litres: f64) -> Result<()> {
        if !tank_capacity_litres.is_finite() || tank_capacity_litres <= 0.0 {
            return Err(anyhow!("Tank capacity must be a finite positive value."));
        }

        if !self.cumulative_change_litres.is_finite() {
            return Err(anyhow!(
                "Calibration anchor cumulative fuel change must be finite."
            ));
        }

        if !self.absolute_litres.is_finite()
            || self.absolute_litres < 0.0
            || self.absolute_litres > tank_capacity_litres
        {
            return Err(anyhow!(
                "Calibration anchor absolute litres must be between zero and the tank capacity."
            ));
        }

        Ok(())
    }

    /// Resolves the unknown session-start quantity.
    ///
    /// Example:
    ///
    /// anchor absolute quantity = 200 L
    /// cumulative fuel added     = +60 L
    ///
    /// session start             = 140 L
    pub fn resolve_starting_litres(&self) -> f64 {
        self.absolute_litres - self.cumulative_change_litres
    }
}
