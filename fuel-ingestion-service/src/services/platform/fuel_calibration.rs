use anyhow::{Result, anyhow};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::calibration::{FuelCalibration, FuelCalibrationAnchor, FuelCalibrationPoint};
use crate::fuel_calibration_repository;
use crate::models::{
    FuelCalibrationProfileResponse, FuelCalibrationSessionPointResponse,
    FuelCalibrationSessionResponse, LatestFuelSensorObservationResponse,
};
use serde_json::to_value;

use crate::models::CreateSensorCalibrationRequest;
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
     * A guided calibration profile may produce only one published runtime
     * calibration.
     *
     * Publishing again would create another inactive sensor_calibrations
     * record and replace published_calibration_id on the profile, leaving
     * the previously published calibration orphaned from the managed
     * workflow.
     *
     * Further physical calibration work should therefore continue through
     * guided sessions before publication. Once published, the resulting
     * runtime calibration is the single candidate that may later be
     * approved for production.
     */
    if profile.published_calibration_id.is_some() {
        return Err(anyhow!(
            "Fuel calibration profile has already been published."
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
    cumulative_change_litres: f64,
) -> Result<Uuid> {
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
     * Resolve the calibration session back to the physical FUEL sensor
     * that owns it.
     *
     * The client must not decide which sensor observation belongs to a
     * calibration session. That relationship is owned by the backend:
     *
     * session -> profile -> sensor.
     */
    let session =
        fuel_calibration_repository::get_fuel_calibration_session_by_id(db_pool, session_id)
            .await?;

    let Some(session) = session else {
        return Err(anyhow!("Fuel calibration session was not found."));
    };

    let profile = fuel_calibration_repository::get_fuel_calibration_profile_by_id(
        db_pool,
        session.profile_id,
    )
    .await?;

    let Some(profile) = profile else {
        return Err(anyhow!("Fuel calibration profile not found."));
    };

    let sensor_id = profile.sensor_id;

    /*
     * Load the newest physical KUM observation for the sensor that
     * actually owns this calibration session.
     *
     * This observation comes from fuel_sensor_observations and therefore
     * exists independently of runtime distance-to-litres calibration.
     */
    let observation =
        fuel_calibration_repository::get_latest_fuel_sensor_observation(db_pool, sensor_id).await?;

    let Some(observation) = observation else {
        return Err(anyhow!(
            "No physical fuel sensor observation is available for this calibration session."
        ));
    };

    /*
     * Calibration evidence must come from a recent physical observation.
     *
     * The firmware normally reports every 30 seconds while parked, which
     * is the expected state during guided tank calibration. Allowing up to
     * 60 seconds accommodates approximately two normal parked reporting
     * cycles without accepting genuinely stale physical measurements.
     *
     * This is intentionally stricter than the general device-health stale
     * threshold because calibration evidence directly determines the
     * distance-to-litres relationship used in production.
     */
    const MAX_CALIBRATION_OBSERVATION_AGE_SECONDS: i64 = 60;

    let observation_age_seconds = (chrono::Utc::now() - observation.recorded_at).num_seconds();

    if observation_age_seconds < 0
        || observation_age_seconds > MAX_CALIBRATION_OBSERVATION_AGE_SECONDS
    {
        return Err(anyhow!(
            "Latest physical fuel sensor observation is too old for calibration capture."
        ));
    }

    /*
     * The real-time KUM distance is ORBI's canonical physical measurement
     * for guided fuel calibration.
     *
     * The measurement is resolved entirely by the backend from the latest
     * physical observation belonging to the session's FUEL sensor. The
     * client supplies only the known cumulative fuel change.
     */
    let observed_level_cm = observation.realtime_distance_cm;

    /*
     * The backend-derived physical KUM measurement must itself be valid
     * before it can become calibration evidence.
     */
    if !observed_level_cm.is_finite() || observed_level_cm < 0.0 {
        return Err(anyhow!(
            "Latest physical fuel sensor observation contains an invalid real-time distance."
        ));
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
        observed_level_cm,
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

pub async fn abandon_session(db_pool: &PgPool, session_id: Uuid) -> Result<()> {
    fuel_calibration_repository::abandon_fuel_calibration_session(db_pool, session_id).await
}

pub async fn supersede_profile(db_pool: &PgPool, profile_id: Uuid) -> Result<()> {
    /*
     * A profile must not be superseded while it still contains an
     * active or paused calibration session.
     *
     * Abandoned and completed sessions are historical and therefore
     * do not block retirement of the profile.
     */
    if fuel_calibration_repository::get_unfinished_fuel_calibration_session(db_pool, profile_id)
        .await?
        .is_some()
    {
        return Err(anyhow!(
            "Fuel calibration profile cannot be superseded while it has an unfinished session."
        ));
    }

    fuel_calibration_repository::supersede_fuel_calibration_profile(db_pool, profile_id).await
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

pub async fn build_publishable_calibration(
    db_pool: &PgPool,
    profile_id: Uuid,
) -> Result<FuelCalibration> {
    /*
     * Load the guided calibration profile.
     *
     * The profile owns the physical tank capacity and identifies
     * the FUEL sensor that this calibration belongs to.
     */
    let profile =
        fuel_calibration_repository::get_fuel_calibration_profile_by_id(db_pool, profile_id)
            .await?;

    let Some(profile) = profile else {
        return Err(anyhow!("Fuel calibration profile not found."));
    };

    /*
     * Only resolved points belonging to completed sessions are allowed
     * to contribute to a runtime calibration.
     *
     * Active, paused and abandoned work is deliberately excluded by
     * the repository.
     */
    let stored_points =
        fuel_calibration_repository::list_publishable_fuel_calibration_points(db_pool, profile_id)
            .await?;

    if stored_points.len() < 2 {
        return Err(anyhow!(
            "At least two resolved calibration points from completed sessions are required."
        ));
    }

    /*
     * Convert persistence rows into the domain representation.
     *
     * The repository already returns these ordered by resolved litres.
     */
    let points = stored_points
        .into_iter()
        .map(|point| FuelCalibrationPoint {
            level_cm: point.level_cm,
            litres: point.resolved_litres,
        })
        .collect();

    /*
     * Construct the runtime calibration.
     *
     * Domain validation remains the final authority over whether this
     * lookup table is physically valid and publishable.
     */
    let calibration = FuelCalibration {
        tank_capacity_litres: profile.tank_capacity_litres,
        points,
    };

    calibration.validate_lookup_table()?;

    Ok(calibration)
}

pub async fn publish_profile(db_pool: &PgPool, profile_id: Uuid) -> Result<Uuid> {
    /*
     * Build and domain-validate the runtime lookup table from completed,
     * resolved guided-calibration evidence.
     */
    let calibration = build_publishable_calibration(db_pool, profile_id).await?;

    /*
     * Reload the profile so we know which installed FUEL sensor owns
     * this calibration.
     */
    let profile =
        fuel_calibration_repository::get_fuel_calibration_profile_by_id(db_pool, profile_id)
            .await?;

    let Some(profile) = profile else {
        return Err(anyhow!("Fuel calibration profile not found."));
    };

    if profile.status == "superseded" {
        return Err(anyhow!(
            "A superseded fuel calibration profile cannot be published."
        ));
    }

    /*
     * Persist the validated typed calibration using the normal runtime
     * sensor-calibration pathway.
     */
    let request = CreateSensorCalibrationRequest {
        calibration_type: "fuel".to_string(),
        calibration_values: to_value(&calibration)?,
    };

    let calibration_id =
        repository::create_inactive_sensor_calibration(db_pool, profile.sensor_id, &request)
            .await?;

    /*
     * Link the guided calibration profile to the runtime calibration
     * that was created from it.
     */
    fuel_calibration_repository::mark_fuel_calibration_profile_published(
        db_pool,
        profile_id,
        calibration_id,
    )
    .await?;

    Ok(calibration_id)
}

pub async fn activate_profile_for_production(db_pool: &PgPool, profile_id: Uuid) -> Result<()> {
    /*
     * Production activation is deliberately separate from publication.
     *
     * Publishing proves that the accumulated guided-calibration
     * evidence can form a mathematically valid runtime lookup table.
     *
     * Production activation is the explicit operational approval step
     * that makes that published calibration the active FUEL
     * calibration used by live telemetry.
     *
     * The repository owns the atomic database transition and verifies:
     *
     * - the profile exists;
     * - the profile is not superseded;
     * - confidence is not LOW;
     * - a published runtime calibration exists;
     * - that calibration belongs to the same sensor;
     * - that calibration is a FUEL calibration;
     * - any previously active FUEL calibration is deactivated;
     * - the published calibration becomes active;
     * - the guided profile becomes PRODUCTION.
     *
     * All of those changes happen inside one transaction so runtime
     * calibration activation and profile lifecycle state cannot drift
     * apart.
     */
    fuel_calibration_repository::mark_fuel_calibration_profile_production(db_pool, profile_id).await
}

pub async fn get_latest_sensor_observation(
    db_pool: &PgPool,
    sensor_id: Uuid,
) -> Result<Option<LatestFuelSensorObservationResponse>> {
    /*
     * Retrieve the newest physical KUM observation independently of
     * whether this sensor currently has an active runtime calibration.
     *
     * This is important during initial installation because guided
     * calibration necessarily begins before a distance-to-litres
     * calibration exists.
     */
    let observation =
        fuel_calibration_repository::get_latest_fuel_sensor_observation(db_pool, sensor_id).await?;

    let Some(observation) = observation else {
        return Ok(None);
    };

    /*
     * Keep persistence representation inside the repository layer and
     * expose an explicit Platform API response model.
     *
     * realtime_distance_cm is the measurement that guided calibration
     * will use when capturing the current physical KUM position.
     */
    Ok(Some(LatestFuelSensorObservationResponse {
        sensor_id: observation.sensor_id,
        device_id: observation.device_id,

        recorded_at: observation.recorded_at,

        realtime_distance_cm: observation.realtime_distance_cm,
        smooth_distance_cm: observation.smooth_distance_cm,
        raw_distance_cm: observation.raw_distance_cm,

        temperature_c: observation.temperature_c,

        status_1: observation.status_1,
        status_2: observation.status_2,
        raw_data_validity: observation.raw_data_validity,

        latitude: observation.latitude,
        longitude: observation.longitude,
    }))
}

pub async fn get_profile(
    db_pool: &PgPool,
    sensor_id: Uuid,
) -> Result<Option<FuelCalibrationProfileResponse>> {
    /*
     * Load the current non-superseded guided calibration profile
     * belonging to this installed fuel sensor.
     */
    let profile =
        fuel_calibration_repository::get_current_fuel_calibration_profile(db_pool, sensor_id)
            .await?;

    let Some(profile) = profile else {
        return Ok(None);
    };

    /*
     * Load the complete guided-session history for this profile.
     *
     * This includes:
     *
     * - active sessions;
     * - paused sessions;
     * - completed sessions.
     *
     * That allows Platform Management to completely reconstruct
     * calibration progress after a browser restart, backend restart,
     * installer pause, or later return to the vehicle.
     */
    let stored_sessions =
        fuel_calibration_repository::list_fuel_calibration_sessions(db_pool, profile.id).await?;

    let mut sessions = Vec::with_capacity(stored_sessions.len());

    for stored_session in stored_sessions {
        /*
         * Load every physical KUM observation captured during this
         * particular guided calibration session.
         */
        let stored_points = fuel_calibration_repository::list_fuel_calibration_session_points(
            db_pool,
            stored_session.id,
        )
        .await?;

        /*
         * Convert repository rows into API response models.
         */
        let points = stored_points
            .into_iter()
            .map(|point| FuelCalibrationSessionPointResponse {
                id: point.id,
                level_cm: point.level_cm,
                cumulative_change_litres: point.cumulative_change_litres,
                resolved_litres: point.resolved_litres,
                captured_at: point.captured_at,
            })
            .collect();

        sessions.push(FuelCalibrationSessionResponse {
            id: stored_session.id,
            status: stored_session.status,

            started_at: stored_session.started_at,
            completed_at: stored_session.completed_at,

            starting_litres: stored_session.starting_litres,
            ending_litres: stored_session.ending_litres,

            anchor_cumulative_change_litres: stored_session.anchor_cumulative_change_litres,

            anchor_absolute_litres: stored_session.anchor_absolute_litres,

            anchor_established_at: stored_session.anchor_established_at,

            points,
        });
    }

    /*
     * Return the complete management representation.
     *
     * This is workflow state, not the published runtime calibration
     * stored in sensor_calibrations.
     */
    Ok(Some(FuelCalibrationProfileResponse {
        id: profile.id,
        sensor_id: profile.sensor_id,

        tank_capacity_litres: profile.tank_capacity_litres,

        status: profile.status,
        confidence: profile.confidence,

        verified_from_litres: profile.verified_from_litres,
        verified_to_litres: profile.verified_to_litres,
        coverage_percentage: profile.coverage_percentage,

        published_calibration_id: profile.published_calibration_id,

        sessions,

        created_at: profile.created_at,
        updated_at: profile.updated_at,
    }))
}

pub async fn get_device_calibration_mode(db_pool: &PgPool, device_code: &str) -> Result<bool> {
    /*
     * Resolve the provisioned device and its installed sensor capabilities
     * using the same authoritative telemetry context used during ingestion.
     */
    let context = repository::find_registered_telemetry_context(db_pool, device_code).await?;

    let Some(context) = context else {
        return Err(anyhow!(
            "Unknown device '{}'. Device must be provisioned before runtime state can be requested.",
            device_code
        ));
    };

    /*
     * Devices without the Fuel Intelligence capability can never enter
     * fuel-calibration mode.
     */
    let Some(fuel_sensor_id) = context.fuel_sensor_id else {
        return Ok(false);
    };

    /*
     * Calibration mode is derived from the calibration-session lifecycle.
     *
     * Only an ACTIVE guided calibration session enables the faster physical
     * fuel-observation mode. Paused, completed and abandoned sessions all
     * resolve to false.
     */
    fuel_calibration_repository::is_fuel_calibration_active(db_pool, fuel_sensor_id).await
}
