use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

/// Represents one production fuel-tank calibration.
///
/// ORBI converts measured liquid-level height into litres by
/// interpolating between verified calibration points collected
/// during the guided tank-filling workflow.
///
/// The KUM sensor is mounted beneath the tank and reports liquid-level
/// height. Therefore, both `level_cm` and `litres` are expected to
/// increase together.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelCalibration {
    /// Declared usable capacity of the tank.
    pub tank_capacity_litres: f64,

    /// Verified lookup points captured during guided calibration.
    pub points: Vec<FuelCalibrationPoint>,
}

/// Represents one verified relationship between the KUM liquid-level
/// height and the known quantity of fuel inside the tank.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelCalibrationPoint {
    /// Liquid-level height reported by the KUM sensor.
    pub level_cm: f64,

    /// Known quantity of fuel present when the level was captured.
    pub litres: f64,
}

impl FuelCalibration {
    pub fn validate_lookup_table(&self) -> Result<()> {
        const LITRE_TOLERANCE: f64 = 0.001;

        /*
         * The declared tank capacity must represent a usable,
         * positive physical quantity.
         */
        if !self.tank_capacity_litres.is_finite() || self.tank_capacity_litres <= 0.0 {
            return Err(anyhow!(
                "Tank capacity must be a finite value greater than zero."
            ));
        }

        /*
         * Interpolation requires at least two distinct calibration points.
         */
        if self.points.len() < 2 {
            return Err(anyhow!(
                "Fuel calibration requires at least two calibration points."
            ));
        }

        /*
         * Validate every individual captured point before checking the
         * relationship between neighbouring points.
         */
        for point in &self.points {
            if !point.level_cm.is_finite() || point.level_cm < 0.0 {
                return Err(anyhow!(
                    "Every calibration level must be finite and non-negative."
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

        /*
         * The KUM is mounted beneath the tank and reports liquid-level height.
         *
         * Therefore, as fuel is added:
         *
         * level_cm must increase
         * litres must increase
         *
         * Strict comparison also rejects duplicate levels and duplicate
         * litre quantities.
         */
        for neighbouring_points in self.points.windows(2) {
            let current = &neighbouring_points[0];
            let next = &neighbouring_points[1];

            if next.level_cm <= current.level_cm {
                return Err(anyhow!("Calibration levels must be strictly increasing."));
            }

            if next.litres <= current.litres {
                return Err(anyhow!(
                    "Calibration quantities must be strictly increasing."
                ));
            }
        }

        /*
         * The guided production workflow finishes when the tank is full.
         *
         * Therefore, the final calibration point must match the declared
         * usable tank capacity.
         */
        let final_point = self
            .points
            .last()
            .expect("at least two calibration points were already verified");

        if (final_point.litres - self.tank_capacity_litres).abs() > LITRE_TOLERANCE {
            return Err(anyhow!(
                "The final calibration point must equal the declared tank capacity."
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_lookup_table_is_accepted() {
        let calibration = FuelCalibration {
            tank_capacity_litres: 200.0,
            points: vec![
                FuelCalibrationPoint {
                    level_cm: 10.0,
                    litres: 20.0,
                },
                FuelCalibrationPoint {
                    level_cm: 30.0,
                    litres: 60.0,
                },
                FuelCalibrationPoint {
                    level_cm: 50.0,
                    litres: 100.0,
                },
                FuelCalibrationPoint {
                    level_cm: 100.0,
                    litres: 200.0,
                },
            ],
        };

        let result = calibration.validate_lookup_table();

        assert!(result.is_ok());
    }

    #[test]
    fn duplicate_levels_are_rejected() {
        let calibration = FuelCalibration {
            tank_capacity_litres: 200.0,
            points: vec![
                FuelCalibrationPoint {
                    level_cm: 10.0,
                    litres: 20.0,
                },
                FuelCalibrationPoint {
                    level_cm: 10.0,
                    litres: 60.0,
                },
                FuelCalibrationPoint {
                    level_cm: 100.0,
                    litres: 200.0,
                },
            ],
        };

        let result = calibration.validate_lookup_table();

        assert!(result.is_err());
    }

    #[test]
    fn duplicate_litre_quantities_are_rejected() {
        let calibration = FuelCalibration {
            tank_capacity_litres: 200.0,
            points: vec![
                FuelCalibrationPoint {
                    level_cm: 10.0,
                    litres: 20.0,
                },
                FuelCalibrationPoint {
                    level_cm: 30.0,
                    litres: 20.0,
                },
                FuelCalibrationPoint {
                    level_cm: 100.0,
                    litres: 200.0,
                },
            ],
        };

        let result = calibration.validate_lookup_table();

        assert!(result.is_err());
    }

    #[test]
    fn decreasing_levels_are_rejected() {
        let calibration = FuelCalibration {
            tank_capacity_litres: 200.0,
            points: vec![
                FuelCalibrationPoint {
                    level_cm: 10.0,
                    litres: 20.0,
                },
                FuelCalibrationPoint {
                    level_cm: 8.0,
                    litres: 60.0,
                },
                FuelCalibrationPoint {
                    level_cm: 100.0,
                    litres: 200.0,
                },
            ],
        };

        let result = calibration.validate_lookup_table();

        assert!(result.is_err());
    }

    #[test]
    fn decreasing_litre_quantities_are_rejected() {
        let calibration = FuelCalibration {
            tank_capacity_litres: 200.0,
            points: vec![
                FuelCalibrationPoint {
                    level_cm: 10.0,
                    litres: 20.0,
                },
                FuelCalibrationPoint {
                    level_cm: 30.0,
                    litres: 15.0,
                },
                FuelCalibrationPoint {
                    level_cm: 100.0,
                    litres: 200.0,
                },
            ],
        };

        let result = calibration.validate_lookup_table();

        assert!(result.is_err());
    }

    #[test]
    fn final_point_below_tank_capacity_is_rejected() {
        let calibration = FuelCalibration {
            tank_capacity_litres: 200.0,
            points: vec![
                FuelCalibrationPoint {
                    level_cm: 10.0,
                    litres: 20.0,
                },
                FuelCalibrationPoint {
                    level_cm: 30.0,
                    litres: 60.0,
                },
                FuelCalibrationPoint {
                    level_cm: 100.0,
                    litres: 180.0,
                },
            ],
        };

        let result = calibration.validate_lookup_table();

        assert!(result.is_err());
    }

    #[test]
    fn zero_tank_capacity_is_rejected() {
        let calibration = FuelCalibration {
            tank_capacity_litres: 0.0,
            points: vec![
                FuelCalibrationPoint {
                    level_cm: 10.0,
                    litres: 0.0,
                },
                FuelCalibrationPoint {
                    level_cm: 100.0,
                    litres: 0.0,
                },
            ],
        };

        let result = calibration.validate_lookup_table();

        assert!(result.is_err());
    }

    #[test]
    fn fewer_than_two_points_are_rejected() {
        let calibration = FuelCalibration {
            tank_capacity_litres: 200.0,
            points: vec![FuelCalibrationPoint {
                level_cm: 100.0,
                litres: 200.0,
            }],
        };

        let result = calibration.validate_lookup_table();

        assert!(result.is_err());
    }
}
