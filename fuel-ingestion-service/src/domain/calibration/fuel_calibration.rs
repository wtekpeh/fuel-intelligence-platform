use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

/// Represents one production fuel-tank calibration.
///
/// ORBI converts a KUM ultrasonic measurement into litres by
/// interpolating between physically verified calibration points.
///
/// Physical bench testing of the current KUM installation confirmed
/// that the reported measurement increases as liquid quantity increases.
///
/// Therefore:
///
/// - lower fuel quantity -> lower measured level;
/// - higher fuel quantity -> higher measured level.
///
/// A valid production lookup table is consequently ordered by
/// increasing litres and strictly increasing `level_cm`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelCalibration {
    /// Declared usable capacity of the tank.
    pub tank_capacity_litres: f64,

    /// Verified lookup points captured during guided calibration.
    pub points: Vec<FuelCalibrationPoint>,
}

/// Represents one verified relationship between a KUM measurement and
/// the known quantity of fuel inside the tank.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelCalibrationPoint {
    /// KUM measurement associated with this known fuel quantity.
    ///
    /// The current backend field remains named `level_cm`.
    pub level_cm: f64,

    /// Known quantity of fuel present when the measurement was captured.
    pub litres: f64,
}

impl FuelCalibration {
    pub fn validate_lookup_table(&self) -> Result<()> {
        const LITRE_TOLERANCE: f64 = 0.001;

        /*
         * Tank capacity must represent a physically meaningful,
         * positive finite quantity.
         */
        if !self.tank_capacity_litres.is_finite() || self.tank_capacity_litres <= 0.0 {
            return Err(anyhow!(
                "Tank capacity must be a finite value greater than zero."
            ));
        }

        /*
         * At least two verified points are required for interpolation.
         */
        if self.points.len() < 2 {
            return Err(anyhow!(
                "Fuel calibration requires at least two calibration points."
            ));
        }

        /*
         * Validate every individual calibration observation.
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
         * Physical KUM relationship
         * -------------------------
         *
         * Bench testing confirmed:
         *
         *     litres increase
         *     measured level increases
         *
         * Therefore lookup points ordered by litres must satisfy:
         *
         *     next.litres   > current.litres
         *     next.level_cm > current.level_cm
         *
         * Strict comparisons also reject duplicate levels and duplicate
         * fuel quantities.
         */
        for neighbouring_points in self.points.windows(2) {
            let current = &neighbouring_points[0];
            let next = &neighbouring_points[1];

            if next.level_cm <= current.level_cm {
                return Err(anyhow!(
                    "Calibration levels must be strictly increasing as fuel quantity increases."
                ));
            }

            if next.litres <= current.litres {
                return Err(anyhow!(
                    "Calibration quantities must be strictly increasing."
                ));
            }
        }

        /*
         * A publishable production lookup table must include the
         * declared usable tank capacity.
         *
         * Since points are ordered by increasing litres, the final
         * point represents the highest calibrated quantity.
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
    fn valid_increasing_level_lookup_table_is_accepted() {
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

        assert!(calibration.validate_lookup_table().is_ok());
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

        assert!(calibration.validate_lookup_table().is_err());
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

        assert!(calibration.validate_lookup_table().is_err());
    }

    #[test]
    fn decreasing_levels_as_litres_increase_are_rejected() {
        let calibration = FuelCalibration {
            tank_capacity_litres: 200.0,
            points: vec![
                FuelCalibrationPoint {
                    level_cm: 30.0,
                    litres: 20.0,
                },
                FuelCalibrationPoint {
                    level_cm: 20.0,
                    litres: 60.0,
                },
                FuelCalibrationPoint {
                    level_cm: 10.0,
                    litres: 200.0,
                },
            ],
        };

        assert!(calibration.validate_lookup_table().is_err());
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

        assert!(calibration.validate_lookup_table().is_err());
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

        assert!(calibration.validate_lookup_table().is_err());
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

        assert!(calibration.validate_lookup_table().is_err());
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

        assert!(calibration.validate_lookup_table().is_err());
    }

    #[test]
    fn physical_bench_style_increasing_curve_is_accepted() {
        /*
         * Representative values following the direction physically
         * observed during the water-container experiment:
         *
         * less liquid  -> smaller KUM measurement
         * more liquid  -> larger KUM measurement.
         *
         * These values protect the direction of the relationship;
         * they are not intended to become production calibration data.
         */
        let calibration = FuelCalibration {
            tank_capacity_litres: 1.5,
            points: vec![
                FuelCalibrationPoint {
                    level_cm: 19.80,
                    litres: 1.00,
                },
                FuelCalibrationPoint {
                    level_cm: 24.75,
                    litres: 1.25,
                },
                FuelCalibrationPoint {
                    level_cm: 30.00,
                    litres: 1.50,
                },
            ],
        };

        assert!(calibration.validate_lookup_table().is_ok());
    }
}
