use anyhow::{Result, anyhow};

use crate::domain::telemetry::models::CalibratedFuelTelemetry;

use super::FuelCalibration;

/// Converts KUM liquid-level measurements into calibrated fuel
/// quantities using a verified lookup table.
///
/// Behaviour:
///
/// - Measurements inside the verified level range are interpolated.
/// - Measurements above the final full-tank point are clamped to the
///   declared tank capacity.
/// - Measurements below the first verified point return `None`,
///   because the lower tank range has not yet been calibrated.
/// - Extrapolation below the verified range is deliberately avoided.
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MeasurementClassification {
    BelowVerifiedRange,
    WithinVerifiedRange,
    AboveVerifiedRange,
}
///
pub struct FuelCalibrationEngine;

impl FuelCalibrationEngine {
    pub fn apply(
        measured_level_cm: f64,
        calibration: &FuelCalibration,
    ) -> Result<Option<CalibratedFuelTelemetry>> {
        /*
         * The engine normally receives a typed calibration produced by
         * CalibrationFactory, which has already validated the lookup table.
         *
         * This finite-value check still protects the mathematical boundary
         * against an invalid live sensor measurement.
         */
        if !measured_level_cm.is_finite() {
            return Err(anyhow!("Measured fuel level must be a finite value."));
        }

        match Self::classify_measurement(measured_level_cm, calibration)? {
            MeasurementClassification::BelowVerifiedRange => {
                return Ok(None);
            }

            MeasurementClassification::AboveVerifiedRange => {
                let final_point = calibration
                    .points
                    .last()
                    .expect("classification already verified calibration points");

                return Ok(Some(Self::build_result(
                    final_point.litres,
                    calibration.tank_capacity_litres,
                )));
            }

            MeasurementClassification::WithinVerifiedRange => {}
        }

        /*
         * Locate the two verified calibration points that bound the
         * measured liquid level.
         *
         * Lookup-table validation guarantees that both level and litre
         * quantities are strictly increasing.
         */
        for neighbouring_points in calibration.points.windows(2) {
            let lower_point = &neighbouring_points[0];
            let upper_point = &neighbouring_points[1];

            let measurement_is_inside_segment = measured_level_cm >= lower_point.level_cm
                && measured_level_cm <= upper_point.level_cm;

            if measurement_is_inside_segment {
                let interpolated_litres =
                    Self::interpolate_litres(measured_level_cm, lower_point, upper_point)?;

                return Ok(Some(Self::build_result(
                    interpolated_litres,
                    calibration.tank_capacity_litres,
                )));
            }
        }

        Err(anyhow!(
            "Measured fuel level could not be matched to a calibration interval."
        ))
    }

    fn classify_measurement(
        measured_level_cm: f64,
        calibration: &FuelCalibration,
    ) -> Result<MeasurementClassification> {
        let first_point = calibration
            .points
            .first()
            .ok_or_else(|| anyhow!("Fuel calibration contains no points."))?;

        let final_point = calibration
            .points
            .last()
            .ok_or_else(|| anyhow!("Fuel calibration contains no points."))?;

        if measured_level_cm < first_point.level_cm {
            return Ok(MeasurementClassification::BelowVerifiedRange);
        }

        if measured_level_cm >= final_point.level_cm {
            return Ok(MeasurementClassification::AboveVerifiedRange);
        }

        Ok(MeasurementClassification::WithinVerifiedRange)
    }

    fn interpolate_litres(
        measured_level_cm: f64,
        lower_point: &super::FuelCalibrationPoint,
        upper_point: &super::FuelCalibrationPoint,
    ) -> Result<f64> {
        let level_range_cm = upper_point.level_cm - lower_point.level_cm;

        if level_range_cm <= 0.0 {
            return Err(anyhow!(
                "Fuel calibration contains an invalid level interval."
            ));
        }

        let position_inside_segment = (measured_level_cm - lower_point.level_cm) / level_range_cm;

        let litre_range = upper_point.litres - lower_point.litres;

        Ok(lower_point.litres + position_inside_segment * litre_range)
    }

    fn build_result(litres: f64, tank_capacity_litres: f64) -> CalibratedFuelTelemetry {
        let bounded_litres = litres.clamp(0.0, tank_capacity_litres);

        let percentage = bounded_litres / tank_capacity_litres * 100.0;

        CalibratedFuelTelemetry {
            litres: bounded_litres,
            percentage,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::calibration::{FuelCalibration, FuelCalibrationPoint};

    fn sample_calibration() -> FuelCalibration {
        FuelCalibration {
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
        }
    }

    #[test]
    fn exact_lookup_point_returns_expected_result() {
        let calibration = sample_calibration();

        let calibrated = FuelCalibrationEngine::apply(30.0, &calibration)
            .unwrap()
            .unwrap();

        assert_eq!(calibrated.litres, 60.0);

        assert_eq!(calibrated.percentage, 30.0);
    }

    #[test]
    fn midpoint_between_two_points_interpolates_correctly() {
        let calibration = sample_calibration();

        let calibrated = FuelCalibrationEngine::apply(20.0, &calibration)
            .unwrap()
            .unwrap();

        assert_eq!(calibrated.litres, 40.0);
        assert_eq!(calibrated.percentage, 20.0);
    }

    #[test]
    fn below_verified_range_returns_none() {
        let calibration = sample_calibration();

        let calibrated = FuelCalibrationEngine::apply(5.0, &calibration).unwrap();

        assert!(calibrated.is_none());
    }

    #[test]
    fn above_final_point_clamps_to_tank_capacity() {
        let calibration = sample_calibration();

        let calibrated = FuelCalibrationEngine::apply(120.0, &calibration)
            .unwrap()
            .unwrap();

        assert_eq!(calibrated.litres, 200.0);
        assert_eq!(calibrated.percentage, 100.0);
    }

    #[test]
    fn irregular_lookup_table_interpolates_correctly() {
        let calibration = FuelCalibration {
            tank_capacity_litres: 250.0,

            points: vec![
                FuelCalibrationPoint {
                    level_cm: 5.0,
                    litres: 12.0,
                },
                FuelCalibrationPoint {
                    level_cm: 18.0,
                    litres: 45.0,
                },
                FuelCalibrationPoint {
                    level_cm: 37.0,
                    litres: 91.0,
                },
                FuelCalibrationPoint {
                    level_cm: 62.0,
                    litres: 170.0,
                },
                FuelCalibrationPoint {
                    level_cm: 94.0,
                    litres: 250.0,
                },
            ],
        };

        let calibrated = FuelCalibrationEngine::apply(49.5, &calibration)
            .unwrap()
            .unwrap();

        let expected_litres = 130.5;
        let expected_percentage = 52.2;
        let tolerance = 0.0001;

        assert!((calibrated.litres - expected_litres).abs() < tolerance);

        assert!((calibrated.percentage - expected_percentage).abs() < tolerance);
    }

    #[test]
    fn non_finite_measured_level_is_rejected() {
        let calibration = sample_calibration();

        let result = FuelCalibrationEngine::apply(f64::NAN, &calibration);

        assert!(result.is_err());
    }
}
