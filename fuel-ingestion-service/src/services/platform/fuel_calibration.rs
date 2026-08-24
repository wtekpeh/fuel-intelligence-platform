use anyhow::{Result, anyhow};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::calibration::FuelCalibrationAnchor;
use crate::fuel_calibration_repository;
use crate::repository;

pub async fn create_profile(
    db_pool: &PgPool,
    sensor_id: Uuid,
    tank_capacity_litres: f64,
) -> Result<Uuid> {
    /*
     * Guided fuel calibration belongs only to a provisioned FUEL sensor.
     *
     * Looking up the sensor type also proves that the sensor exists.
     */
    let sensor_type = repository::get_sensor_type(db_pool, sensor_id).await?;

    let Some(sensor_type) = sensor_type else {
        return Err(anyhow!("Sensor not found."));
    };

    if sensor_type != "FUEL" {
        return Err(anyhow!(
            "Guided fuel calibration can only be created for a FUEL sensor."
        ));
    }

    /*
     * Tank capacity is a physical installation property and must
     * always be a valid positive finite quantity.
     */
    if !tank_capacity_litres.is_finite() || tank_capacity_litres <= 0.0 {
        return Err(anyhow!("Tank capacity must be a finite positive value."));
    }

    /*
     * A sensor may have only one current, non-superseded guided
     * calibration profile.
     */
    if fuel_calibration_repository::get_current_fuel_calibration_profile(db_pool, sensor_id)
        .await?
        .is_some()
    {
        return Err(anyhow!(
            "A current fuel calibration profile already exists for this sensor."
        ));
    }

    fuel_calibration_repository::create_fuel_calibration_profile(
        db_pool,
        sensor_id,
        tank_capacity_litres,
    )
    .await
}

pub async fn start_session(
    db_pool: &PgPool,
    profile_id: Uuid,
    starting_litres: Option<f64>,
) -> Result<Uuid> {
    /*
     * The guided calibration profile must exist and must still be the
     * current profile for its fuel sensor.
     */
    let profile =
        fuel_calibration_repository::get_fuel_calibration_profile_by_id(db_pool, profile_id)
            .await?;

    let Some(profile) = profile else {
        return Err(anyhow!("Fuel calibration profile not found."));
    };

    /*
     * Superseded profiles are historical records and must never receive
     * new guided calibration sessions.
     */
    if profile.status == "superseded" {
        return Err(anyhow!(
            "A guided calibration session cannot be started for a superseded profile."
        ));
    }

    /*
     * Only one unfinished session may exist for a profile.
     *
     * An unfinished session may be either:
     *
     * - active;
     * - paused.
     *
     * A paused session must be resumed rather than creating another one.
     */
    if fuel_calibration_repository::get_unfinished_fuel_calibration_session(db_pool, profile_id)
        .await?
        .is_some()
    {
        return Err(anyhow!(
            "This fuel calibration profile already has an unfinished session."
        ));
    }

    /*
     * The installer is allowed to begin without knowing the absolute
     * amount of fuel currently in the tank.
     *
     * Therefore:
     *
     *     None
     *
     * is a valid starting condition.
     *
     * If the quantity is known, however, it must be physically valid.
     */
    if let Some(starting_litres) = starting_litres {
        if !starting_litres.is_finite() || starting_litres < 0.0 {
            return Err(anyhow!(
                "Starting fuel quantity must be a finite non-negative value."
            ));
        }

        if starting_litres > profile.tank_capacity_litres {
            return Err(anyhow!(
                "Starting fuel quantity must not exceed the tank capacity."
            ));
        }
    }

    fuel_calibration_repository::start_fuel_calibration_session(
        db_pool,
        profile_id,
        starting_litres,
    )
    .await
}

pub async fn capture_point(
    db_pool: &PgPool,
    session_id: Uuid,
    level_cm: f64,
    cumulative_change_litres: f64,
) -> Result<Uuid> {
    /*
     * The physical KUM measurement must always be a finite
     * non-negative distance.
     */
    if !level_cm.is_finite() || level_cm < 0.0 {
        return Err(anyhow!(
            "Fuel calibration level must be a finite non-negative value."
        ));
    }

    /*
     * Cumulative fuel change is signed:
     *
     *  0.0  = session starting position
     * +20.0 = twenty litres added
     * -20.0 = twenty litres removed
     *
     * It may therefore be positive, zero, or negative, but it must
     * always be finite.
     */
    if !cumulative_change_litres.is_finite() {
        return Err(anyhow!("Cumulative fuel change must be finite."));
    }

    /*
     * The repository owns the persistence details and will:
     *
     * - reject missing sessions;
     * - reject sessions that are not active;
     * - keep resolved_litres NULL before anchoring;
     * - immediately resolve litres after an anchor exists;
     * - reject quantities outside the declared tank capacity.
     */
    fuel_calibration_repository::capture_fuel_calibration_point(
        db_pool,
        session_id,
        level_cm,
        cumulative_change_litres,
    )
    .await
}

pub async fn pause_session(db_pool: &PgPool, session_id: Uuid) -> Result<()> {
    fuel_calibration_repository::pause_fuel_calibration_session(db_pool, session_id).await
}

pub async fn resume_session(db_pool: &PgPool, session_id: Uuid) -> Result<()> {
    fuel_calibration_repository::resume_fuel_calibration_session(db_pool, session_id).await
}

pub async fn apply_anchor(
    db_pool: &PgPool,
    session_id: Uuid,
    cumulative_change_litres: f64,
    absolute_litres: f64,
) -> Result<()> {
    let anchor = FuelCalibrationAnchor {
        cumulative_change_litres,
        absolute_litres,
        established_at: chrono::Utc::now(),
    };

    fuel_calibration_repository::apply_fuel_calibration_anchor(db_pool, session_id, &anchor).await
}

pub async fn complete_session(db_pool: &PgPool, session_id: Uuid) -> Result<()> {
    fuel_calibration_repository::complete_fuel_calibration_session(db_pool, session_id).await
}
