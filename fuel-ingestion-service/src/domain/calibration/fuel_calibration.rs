use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

/// Represents one production fuel-tank calibration.
///
/// ORBI converts ultrasonic distance measurements into litres by
/// interpolating between measured calibration points collected during
/// the guided tank-filling workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelCalibration {
    pub tank_capacity_litres: f64,

    #[serde(default)]
    pub mounting_offset_cm: f64,

    pub points: Vec<FuelCalibrationPoint>,
}

/// Represents one verified relationship between the KUM sensor's
/// measured distance and the known quantity of fuel in the tank.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelCalibrationPoint {
    pub distance_cm: f64,
    pub litres: f64,
}

impl FuelCalibration {
    pub fn validate(&self) -> Result<()> {
        if !self.tank_capacity_litres.is_finite() || self.tank_capacity_litres <= 0.0 {
            return Err(anyhow!(
                "Tank capacity must be a finite value greater than zero."
            ));
        }

        if !self.mounting_offset_cm.is_finite() {
            return Err(anyhow!("Mounting offset must be finite."));
        }

        if self.points.len() < 2 {
            return Err(anyhow!(
                "Fuel calibration requires at least two calibration points."
            ));
        }

        for point in &self.points {
            if !point.distance_cm.is_finite() || point.distance_cm < 0.0 {
                return Err(anyhow!(
                    "Every calibration distance must be finite and non-negative."
                ));
            }

            if !point.litres.is_finite()
                || point.litres < 0.0
                || point.litres > self.tank_capacity_litres
            {
                return Err(anyhow!(
                    "Every calibration quantity must be between zero and the tank capacity."
                ));
            }
        }

        Ok(())
    }
}
