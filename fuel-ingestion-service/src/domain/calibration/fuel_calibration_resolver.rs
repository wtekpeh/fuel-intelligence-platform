use anyhow::{Result, anyhow};

use super::{FuelCalibrationAnchor, FuelCalibrationSessionPoint};

/// Resolves previously captured relative calibration points into
/// absolute fuel quantities using a trusted calibration anchor.
///
/// Example:
///
/// session points:
///
/// cumulative change = 0 L
/// cumulative change = 20 L
/// cumulative change = 40 L
/// cumulative change = 60 L
///
/// anchor:
///
/// cumulative change = 60 L
/// absolute quantity = 200 L
///
/// resolved starting quantity:
///
/// 200 - 60 = 140 L
///
/// resolved points:
///
/// 0  → 140 L
/// 20 → 160 L
/// 40 → 180 L
/// 60 → 200 L
pub fn resolve_session_points(
    points: &mut [FuelCalibrationSessionPoint],
    anchor: &FuelCalibrationAnchor,
    tank_capacity_litres: f64,
) -> Result<f64> {
    anchor.validate(tank_capacity_litres)?;

    let starting_litres = anchor.resolve_starting_litres();

    if !starting_litres.is_finite()
        || starting_litres < 0.0
        || starting_litres > tank_capacity_litres
    {
        return Err(anyhow!(
            "Calibration anchor resolves to an invalid session starting quantity."
        ));
    }

    for point in points.iter_mut() {
        point.validate()?;

        let resolved_litres = starting_litres + point.cumulative_change_litres;

        if !resolved_litres.is_finite()
            || resolved_litres < 0.0
            || resolved_litres > tank_capacity_litres
        {
            return Err(anyhow!(
                "Calibration point resolves outside the declared tank capacity."
            ));
        }

        point.resolved_litres = Some(resolved_litres);
    }

    Ok(starting_litres)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn point(level_cm: f64, cumulative_change_litres: f64) -> FuelCalibrationSessionPoint {
        FuelCalibrationSessionPoint {
            id: Uuid::new_v4(),
            level_cm,
            cumulative_change_litres,
            resolved_litres: None,
            captured_at: Utc::now(),
        }
    }

    #[test]
    fn full_tank_anchor_resolves_previous_added_fuel_points() {
        /*
         * The installer begins calibration without knowing how much
         * fuel is already in the tank.
         *
         * Fuel is then added in known quantities:
         *
         * Start       = unknown
         * +20 litres
         * +40 litres
         * +60 litres
         *
         * At +60 litres, the tank is confirmed FULL at 200 litres.
         *
         * Therefore:
         *
         * starting quantity = 200 - 60 = 140 litres
         */
        let mut points = vec![
            point(70.0, 0.0),
            point(55.0, 20.0),
            point(40.0, 40.0),
            point(25.0, 60.0),
        ];

        let anchor = FuelCalibrationAnchor {
            cumulative_change_litres: 60.0,
            absolute_litres: 200.0,
            established_at: Utc::now(),
        };

        let starting_litres = resolve_session_points(&mut points, &anchor, 200.0)
            .expect("full-tank anchor should resolve the session");

        assert_eq!(starting_litres, 140.0);

        assert_eq!(points[0].resolved_litres, Some(140.0));
        assert_eq!(points[1].resolved_litres, Some(160.0));
        assert_eq!(points[2].resolved_litres, Some(180.0));
        assert_eq!(points[3].resolved_litres, Some(200.0));
    }

    #[test]
    fn empty_tank_anchor_resolves_previous_removed_fuel_points() {
        /*
         * Again, the initial absolute quantity is unknown.
         *
         * This time fuel is removed:
         *
         * Start       = unknown
         * -20 litres
         * -40 litres
         * -60 litres
         *
         * At -60 litres, the tank is confirmed EMPTY at 0 litres.
         *
         * Therefore:
         *
         * starting quantity = 0 - (-60) = 60 litres
         */
        let mut points = vec![
            point(25.0, 0.0),
            point(40.0, -20.0),
            point(55.0, -40.0),
            point(70.0, -60.0),
        ];

        let anchor = FuelCalibrationAnchor {
            cumulative_change_litres: -60.0,
            absolute_litres: 0.0,
            established_at: Utc::now(),
        };

        let starting_litres = resolve_session_points(&mut points, &anchor, 200.0)
            .expect("empty-tank anchor should resolve the session");

        assert_eq!(starting_litres, 60.0);

        assert_eq!(points[0].resolved_litres, Some(60.0));
        assert_eq!(points[1].resolved_litres, Some(40.0));
        assert_eq!(points[2].resolved_litres, Some(20.0));
        assert_eq!(points[3].resolved_litres, Some(0.0));
    }

    #[test]
    fn independently_measured_anchor_resolves_session() {
        /*
         * An anchor does not have to be EMPTY or FULL.
         *
         * Suppose the installer has added 40 litres since starting
         * calibration and independently establishes that the tank
         * currently contains exactly 150 litres.
         *
         * Starting quantity:
         *
         * 150 - 40 = 110 litres
         */
        let mut points = vec![point(60.0, 0.0), point(50.0, 20.0), point(40.0, 40.0)];

        let anchor = FuelCalibrationAnchor {
            cumulative_change_litres: 40.0,
            absolute_litres: 150.0,
            established_at: Utc::now(),
        };

        let starting_litres = resolve_session_points(&mut points, &anchor, 200.0)
            .expect("measured anchor should resolve the session");

        assert_eq!(starting_litres, 110.0);

        assert_eq!(points[0].resolved_litres, Some(110.0));
        assert_eq!(points[1].resolved_litres, Some(130.0));
        assert_eq!(points[2].resolved_litres, Some(150.0));
    }

    #[test]
    fn anchor_that_resolves_start_above_capacity_is_rejected() {
        let mut points = vec![point(40.0, 0.0), point(50.0, -20.0)];

        /*
         * Current absolute quantity = 190 litres.
         * 20 litres have already been removed.
         *
         * This implies:
         *
         * starting quantity = 190 - (-20) = 210 litres
         *
         * But tank capacity is only 200 litres.
         */
        let anchor = FuelCalibrationAnchor {
            cumulative_change_litres: -20.0,
            absolute_litres: 190.0,
            established_at: Utc::now(),
        };

        let error = resolve_session_points(&mut points, &anchor, 200.0)
            .expect_err("impossible starting quantity should fail");

        assert_eq!(
            error.to_string(),
            "Calibration anchor resolves to an invalid session starting quantity."
        );
    }

    #[test]
    fn point_that_resolves_above_capacity_is_rejected() {
        let mut points = vec![point(60.0, 0.0), point(40.0, 60.0)];

        /*
         * Anchor establishes the starting quantity as 160 litres.
         *
         * The second point would therefore resolve to:
         *
         * 160 + 60 = 220 litres
         *
         * which exceeds the 200 litre tank capacity.
         */
        let anchor = FuelCalibrationAnchor {
            cumulative_change_litres: 0.0,
            absolute_litres: 160.0,
            established_at: Utc::now(),
        };

        let error = resolve_session_points(&mut points, &anchor, 200.0)
            .expect_err("point above capacity should fail");

        assert_eq!(
            error.to_string(),
            "Calibration point resolves outside the declared tank capacity."
        );
    }
}
