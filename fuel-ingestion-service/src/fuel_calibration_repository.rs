use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::calibration::{
    FuelCalibrationAnchor, FuelCalibrationSessionPoint, resolve_session_points,
};

#[derive(Debug, Clone)]
pub struct FuelCalibrationProfileRow {
    pub id: Uuid,
    pub sensor_id: Uuid,
    pub tank_capacity_litres: f64,
    pub status: String,
    pub confidence: String,
    pub verified_from_litres: f64,
    pub verified_to_litres: f64,
    pub coverage_percentage: f64,
    pub published_calibration_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct FuelCalibrationSessionRow {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub status: String,

    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,

    pub starting_litres: Option<f64>,
    pub ending_litres: Option<f64>,

    pub anchor_cumulative_change_litres: Option<f64>,
    pub anchor_absolute_litres: Option<f64>,
    pub anchor_established_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct FuelCalibrationSessionPointRow {
    pub id: Uuid,
    pub session_id: Uuid,

    pub level_cm: f64,
    pub cumulative_change_litres: f64,
    pub resolved_litres: Option<f64>,

    pub captured_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

pub async fn create_fuel_calibration_profile(
    db_pool: &PgPool,
    sensor_id: Uuid,
    tank_capacity_litres: f64,
) -> Result<Uuid> {
    let profile_id = sqlx::query_scalar!(
        r#"
        INSERT INTO fuel_calibration_profiles (
            sensor_id,
            tank_capacity_litres
        )
        VALUES ($1, $2)
        RETURNING id
        "#,
        sensor_id,
        tank_capacity_litres,
    )
    .fetch_one(db_pool)
    .await?;

    Ok(profile_id)
}

pub async fn get_current_fuel_calibration_profile(
    db_pool: &PgPool,
    sensor_id: Uuid,
) -> Result<Option<FuelCalibrationProfileRow>> {
    let profile = sqlx::query_as!(
        FuelCalibrationProfileRow,
        r#"
        SELECT
            id,
            sensor_id,
            tank_capacity_litres,
            status,
            confidence,
            verified_from_litres,
            verified_to_litres,
            coverage_percentage,
            published_calibration_id,
            created_at,
            updated_at
        FROM fuel_calibration_profiles
        WHERE sensor_id = $1
          AND status <> 'superseded'
        LIMIT 1
        "#,
        sensor_id,
    )
    .fetch_optional(db_pool)
    .await?;

    Ok(profile)
}

pub async fn get_fuel_calibration_profile_by_id(
    db_pool: &PgPool,
    profile_id: Uuid,
) -> Result<Option<FuelCalibrationProfileRow>> {
    let profile = sqlx::query_as!(
        FuelCalibrationProfileRow,
        r#"
        SELECT
            id,
            sensor_id,
            tank_capacity_litres,
            status,
            confidence,
            verified_from_litres,
            verified_to_litres,
            coverage_percentage,
            published_calibration_id,
            created_at,
            updated_at
        FROM fuel_calibration_profiles
        WHERE id = $1
        LIMIT 1
        "#,
        profile_id,
    )
    .fetch_optional(db_pool)
    .await?;

    Ok(profile)
}

pub async fn supersede_fuel_calibration_profile(db_pool: &PgPool, profile_id: Uuid) -> Result<()> {
    let result = sqlx::query!(
        r#"
        UPDATE fuel_calibration_profiles
        SET
            status = 'superseded',
            updated_at = NOW()
        WHERE id = $1
          AND status <> 'superseded'
        "#,
        profile_id,
    )
    .execute(db_pool)
    .await?;

    if result.rows_affected() == 0 {
        let existing_status = sqlx::query_scalar!(
            r#"
            SELECT status
            FROM fuel_calibration_profiles
            WHERE id = $1
            "#,
            profile_id,
        )
        .fetch_optional(db_pool)
        .await?;

        match existing_status {
            None => {
                return Err(anyhow!("Fuel calibration profile was not found."));
            }

            Some(_) => {
                return Err(anyhow!("Fuel calibration profile is already superseded."));
            }
        }
    }

    Ok(())
}

pub async fn start_fuel_calibration_session(
    db_pool: &PgPool,
    profile_id: Uuid,
    starting_litres: Option<f64>,
) -> Result<Uuid> {
    let session_id = sqlx::query_scalar!(
        r#"
        INSERT INTO fuel_calibration_sessions (
            profile_id,
            status,
            starting_litres,
            ending_litres
        )
        VALUES (
            $1,
            'active',
            $2,
            $2
        )
        RETURNING id
        "#,
        profile_id,
        starting_litres,
    )
    .fetch_one(db_pool)
    .await?;

    Ok(session_id)
}

pub async fn get_unfinished_fuel_calibration_session(
    db_pool: &PgPool,
    profile_id: Uuid,
) -> Result<Option<FuelCalibrationSessionRow>> {
    let session = sqlx::query_as!(
        FuelCalibrationSessionRow,
        r#"
        SELECT
            id,
            profile_id,
            status,
            started_at,
            completed_at,
            starting_litres,
            ending_litres,
            anchor_cumulative_change_litres,
            anchor_absolute_litres,
            anchor_established_at,
            created_at,
            updated_at
        FROM fuel_calibration_sessions
        WHERE profile_id = $1
          AND status IN ('active', 'paused')
        LIMIT 1
        "#,
        profile_id,
    )
    .fetch_optional(db_pool)
    .await?;

    Ok(session)
}

pub async fn list_fuel_calibration_sessions(
    db_pool: &PgPool,
    profile_id: Uuid,
) -> Result<Vec<FuelCalibrationSessionRow>> {
    let sessions = sqlx::query_as!(
        FuelCalibrationSessionRow,
        r#"
        SELECT
            id,
            profile_id,
            status,
            started_at,
            completed_at,
            starting_litres,
            ending_litres,
            anchor_cumulative_change_litres,
            anchor_absolute_litres,
            anchor_established_at,
            created_at,
            updated_at
        FROM fuel_calibration_sessions
        WHERE profile_id = $1
        ORDER BY started_at ASC, created_at ASC
        "#,
        profile_id,
    )
    .fetch_all(db_pool)
    .await?;

    Ok(sessions)
}

pub async fn capture_fuel_calibration_point(
    db_pool: &PgPool,
    session_id: Uuid,
    level_cm: f64,
    cumulative_change_litres: f64,
) -> Result<Uuid> {
    let session_context = sqlx::query!(
        r#"
        SELECT
            fcs.status,
            fcs.starting_litres,
            fcp.tank_capacity_litres
        FROM fuel_calibration_sessions fcs
        JOIN fuel_calibration_profiles fcp
            ON fcp.id = fcs.profile_id
        WHERE fcs.id = $1
        "#,
        session_id,
    )
    .fetch_optional(db_pool)
    .await?;

    let Some(session_context) = session_context else {
        return Err(anyhow!("Fuel calibration session was not found."));
    };

    if session_context.status != "active" {
        return Err(anyhow!(
            "Calibration points can only be captured while the session is active."
        ));
    }

    /*
     * Before an absolute anchor exists, starting_litres is None and
     * the new physical observation remains unresolved.
     *
     * After anchoring, starting_litres contains the resolved session
     * starting quantity, so every subsequent point can immediately be
     * converted into absolute litres.
     */
    let resolved_litres = if let Some(starting_litres) = session_context.starting_litres {
        let litres = starting_litres + cumulative_change_litres;

        if !litres.is_finite() || litres < 0.0 || litres > session_context.tank_capacity_litres {
            return Err(anyhow!(
                "Calibration point resolves outside the declared tank capacity."
            ));
        }

        Some(litres)
    } else {
        None
    };

    let point_id = sqlx::query_scalar!(
        r#"
        INSERT INTO fuel_calibration_session_points (
            session_id,
            level_cm,
            cumulative_change_litres,
            resolved_litres
        )
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        session_id,
        level_cm,
        cumulative_change_litres,
        resolved_litres,
    )
    .fetch_one(db_pool)
    .await?;

    /*
     * Once the session has been anchored, the most recently captured
     * point also represents the current resolved ending quantity.
     */
    if let Some(resolved_litres) = resolved_litres {
        sqlx::query!(
            r#"
            UPDATE fuel_calibration_sessions
            SET
                ending_litres = $2,
                updated_at = NOW()
            WHERE id = $1
            "#,
            session_id,
            resolved_litres,
        )
        .execute(db_pool)
        .await?;
    }

    Ok(point_id)
}

pub async fn list_fuel_calibration_session_points(
    db_pool: &PgPool,
    session_id: Uuid,
) -> Result<Vec<FuelCalibrationSessionPointRow>> {
    let points = sqlx::query_as!(
        FuelCalibrationSessionPointRow,
        r#"
        SELECT
            id,
            session_id,
            level_cm,
            cumulative_change_litres,
            resolved_litres,
            captured_at,
            created_at
        FROM fuel_calibration_session_points
        WHERE session_id = $1
        ORDER BY captured_at ASC, created_at ASC
        "#,
        session_id,
    )
    .fetch_all(db_pool)
    .await?;

    Ok(points)
}

pub async fn pause_fuel_calibration_session(db_pool: &PgPool, session_id: Uuid) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE fuel_calibration_sessions
        SET
            status = 'paused',
            updated_at = NOW()
        WHERE id = $1
          AND status = 'active'
        "#,
        session_id,
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn resume_fuel_calibration_session(db_pool: &PgPool, session_id: Uuid) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE fuel_calibration_sessions
        SET
            status = 'active',
            updated_at = NOW()
        WHERE id = $1
          AND status = 'paused'
        "#,
        session_id,
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn abandon_fuel_calibration_session(db_pool: &PgPool, session_id: Uuid) -> Result<()> {
    /*
     * Abandoning a guided calibration session means that the
     * observations collected during that session must no longer
     * contribute to verified calibration evidence.
     *
     * The session is retained for traceability rather than deleted.
     *
     * Only an unfinished session may be abandoned:
     *
     * - active;
     * - paused.
     *
     * Completed calibration evidence is immutable through this
     * operation.
     */
    let result = sqlx::query!(
        r#"
        UPDATE fuel_calibration_sessions
        SET
            status = 'abandoned',
            updated_at = NOW()
        WHERE id = $1
          AND status IN ('active', 'paused')
        "#,
        session_id,
    )
    .execute(db_pool)
    .await?;

    /*
     * An affected-row count of zero has two possible meanings:
     *
     * - the session does not exist;
     * - the session exists but is no longer unfinished.
     *
     * Distinguish those cases so the service/API can return a useful
     * error rather than silently reporting success.
     */
    if result.rows_affected() == 0 {
        let existing_status = sqlx::query_scalar!(
            r#"
            SELECT status
            FROM fuel_calibration_sessions
            WHERE id = $1
            "#,
            session_id,
        )
        .fetch_optional(db_pool)
        .await?;

        match existing_status {
            None => {
                return Err(anyhow!("Fuel calibration session was not found."));
            }

            Some(_) => {
                return Err(anyhow!(
                    "Only an active or paused fuel calibration session can be abandoned."
                ));
            }
        }
    }

    Ok(())
}

pub async fn apply_fuel_calibration_anchor(
    db_pool: &PgPool,
    session_id: Uuid,
    anchor: &FuelCalibrationAnchor,
) -> Result<()> {
    let mut transaction = db_pool.begin().await?;

    /*
     * Lock the calibration session and its parent profile while
     * resolving the captured observations.
     *
     * This prevents another calibration operation from modifying the
     * session while its absolute fuel quantities are being resolved.
     */
    let session_context = sqlx::query!(
        r#"
        SELECT
            fcs.id AS session_id,
            fcs.status AS session_status,
            fcp.tank_capacity_litres
        FROM fuel_calibration_sessions fcs
        JOIN fuel_calibration_profiles fcp
            ON fcp.id = fcs.profile_id
        WHERE fcs.id = $1
        FOR UPDATE OF fcs, fcp
        "#,
        session_id,
    )
    .fetch_optional(&mut *transaction)
    .await?;

    let Some(session_context) = session_context else {
        return Err(anyhow!("Fuel calibration session was not found."));
    };

    /*
     * Anchoring is only meaningful while the guided calibration
     * session is still unfinished.
     */
    if session_context.session_status != "active" && session_context.session_status != "paused" {
        return Err(anyhow!(
            "Only an active or paused fuel calibration session can be anchored."
        ));
    }

    let tank_capacity_litres = session_context.tank_capacity_litres;

    /*
     * Load every physical observation captured for this session.
     *
     * Capture order matters because the final point represents the
     * resolved ending quantity of the session at this stage.
     */
    let stored_points = sqlx::query!(
        r#"
        SELECT
            id,
            level_cm,
            cumulative_change_litres,
            resolved_litres,
            captured_at
        FROM fuel_calibration_session_points
        WHERE session_id = $1
        ORDER BY captured_at ASC, created_at ASC
        FOR UPDATE
        "#,
        session_id,
    )
    .fetch_all(&mut *transaction)
    .await?;

    if stored_points.is_empty() {
        return Err(anyhow!(
            "A fuel calibration anchor cannot be applied before any calibration points have been captured."
        ));
    }

    /*
     * The anchor should correspond to one of the physical observations
     * captured during the session.
     *
     * For example:
     *
     * point 0   → cumulative change 0 L
     * point 1   → cumulative change +20 L
     * point 2   → cumulative change +40 L
     * point 3   → cumulative change +60 L
     *
     * FULL = 200 L established at +60 L
     *
     * The +60 L observation therefore provides both:
     *
     * - a physical KUM distance;
     * - the absolute 200 L anchor.
     */
    const CHANGE_TOLERANCE_LITRES: f64 = 0.000_001;

    let anchor_has_captured_point = stored_points.iter().any(|point| {
        (point.cumulative_change_litres - anchor.cumulative_change_litres).abs()
            <= CHANGE_TOLERANCE_LITRES
    });

    if !anchor_has_captured_point {
        return Err(anyhow!(
            "The calibration anchor must correspond to a captured calibration point."
        ));
    }

    /*
     * Convert repository rows into the pure domain model.
     *
     * Resolution itself stays inside the domain layer and does not
     * access PostgreSQL.
     */
    let mut domain_points: Vec<FuelCalibrationSessionPoint> = stored_points
        .iter()
        .map(|point| FuelCalibrationSessionPoint {
            id: point.id,
            level_cm: point.level_cm,
            cumulative_change_litres: point.cumulative_change_litres,
            resolved_litres: point.resolved_litres,
            captured_at: point.captured_at,
        })
        .collect();

    /*
     * Resolve all captured observations into absolute litres.
     *
     * Example:
     *
     * anchor:
     *     +60 L cumulative change
     *     200 L absolute quantity
     *
     * resolved session start:
     *     200 - 60 = 140 L
     *
     * resolved points:
     *       0 L → 140 L
     *     +20 L → 160 L
     *     +40 L → 180 L
     *     +60 L → 200 L
     */
    let starting_litres = resolve_session_points(&mut domain_points, anchor, tank_capacity_litres)?;

    /*
     * The final captured observation represents the current resolved
     * ending quantity of this guided session.
     */
    let ending_litres = domain_points
        .last()
        .and_then(|point| point.resolved_litres)
        .ok_or_else(|| {
            anyhow!("Fuel calibration session ending quantity could not be resolved.")
        })?;

    /*
     * Persist every resolved point inside this same transaction.
     */
    for point in &domain_points {
        let resolved_litres = point.resolved_litres.ok_or_else(|| {
            anyhow!("Fuel calibration point remained unresolved after anchor application.")
        })?;

        sqlx::query!(
            r#"
            UPDATE fuel_calibration_session_points
            SET resolved_litres = $2
            WHERE id = $1
            "#,
            point.id,
            resolved_litres,
        )
        .execute(&mut *transaction)
        .await?;
    }

    /*
     * Persist the anchor itself together with the absolute quantities
     * it resolved for the session.
     *
     * The session remains active or paused. Applying an anchor does
     * not automatically complete the calibration.
     */
    sqlx::query!(
        r#"
        UPDATE fuel_calibration_sessions
        SET
            starting_litres = $2,
            ending_litres = $3,
            anchor_cumulative_change_litres = $4,
            anchor_absolute_litres = $5,
            anchor_established_at = $6,
            updated_at = NOW()
        WHERE id = $1
        "#,
        session_id,
        starting_litres,
        ending_litres,
        anchor.cumulative_change_litres,
        anchor.absolute_litres,
        anchor.established_at,
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(())
}

pub async fn complete_fuel_calibration_session(db_pool: &PgPool, session_id: Uuid) -> Result<()> {
    let mut transaction = db_pool.begin().await?;

    /*
     * Lock the session and its parent profile.
     *
     * Completion changes both records, so they must remain consistent
     * throughout the operation.
     */
    let session_context = sqlx::query!(
        r#"
        SELECT
            fcs.id AS session_id,
            fcs.profile_id,
            fcs.status AS session_status,
            fcs.starting_litres,
            fcs.ending_litres,
            fcp.tank_capacity_litres,
            fcp.verified_from_litres,
            fcp.verified_to_litres
        FROM fuel_calibration_sessions fcs
        JOIN fuel_calibration_profiles fcp
            ON fcp.id = fcs.profile_id
        WHERE fcs.id = $1
        FOR UPDATE OF fcs, fcp
        "#,
        session_id,
    )
    .fetch_optional(&mut *transaction)
    .await?;

    let Some(session_context) = session_context else {
        return Err(anyhow!("Fuel calibration session was not found."));
    };

    /*
     * Only unfinished sessions may be completed.
     */
    if session_context.session_status != "active" && session_context.session_status != "paused" {
        return Err(anyhow!(
            "Only an active or paused fuel calibration session can be completed."
        ));
    }

    /*
     * A completed session must already have an absolute reference.
     *
     * If these are still NULL, the installer has not yet established
     * enough information to resolve the relative observations.
     */
    let starting_litres = session_context.starting_litres.ok_or_else(|| {
        anyhow!(
            "Fuel calibration session cannot be completed before its starting quantity has been resolved."
        )
    })?;

    let ending_litres = session_context.ending_litres.ok_or_else(|| {
        anyhow!(
            "Fuel calibration session cannot be completed before its ending quantity has been resolved."
        )
    })?;

    /*
     * Load and lock all observations belonging to this session.
     */
    let points = sqlx::query!(
        r#"
        SELECT
            id,
            resolved_litres
        FROM fuel_calibration_session_points
        WHERE session_id = $1
        ORDER BY captured_at ASC, created_at ASC
        FOR UPDATE
        "#,
        session_id,
    )
    .fetch_all(&mut *transaction)
    .await?;

    /*
     * This mirrors the FuelCalibrationSession domain invariant:
     *
     * a completed session requires at least two verified points.
     */
    if points.len() < 2 {
        return Err(anyhow!(
            "A completed calibration session must contain at least two verified points."
        ));
    }

    /*
     * Every observation must have been converted from relative change
     * into an absolute fuel quantity before the session can become
     * completed evidence.
     */
    if points.iter().any(|point| point.resolved_litres.is_none()) {
        return Err(anyhow!(
            "Fuel calibration session cannot be completed while calibration points remain unresolved."
        ));
    }

    /*
     * Determine the actual verified absolute range represented by this
     * session.
     *
     * We calculate this from the resolved observations rather than
     * assuming that calibration always progresses in one direction.
     *
     * This allows both:
     *
     *     adding fuel
     *
     * and:
     *
     *     removing fuel
     *
     * during guided calibration.
     */
    let mut session_verified_from_litres = starting_litres.min(ending_litres);
    let mut session_verified_to_litres = starting_litres.max(ending_litres);

    for point in &points {
        let resolved_litres = point.resolved_litres.ok_or_else(|| {
            anyhow!("Fuel calibration session contains an unresolved calibration point.")
        })?;

        session_verified_from_litres = session_verified_from_litres.min(resolved_litres);

        session_verified_to_litres = session_verified_to_litres.max(resolved_litres);
    }

    /*
     * Protect the declared physical tank range.
     */
    if session_verified_from_litres < 0.0
        || session_verified_to_litres > session_context.tank_capacity_litres
    {
        return Err(anyhow!(
            "Completed calibration session resolves outside the declared tank capacity."
        ));
    }

    /*
     * Merge this session's verified range with the profile's existing
     * verified range.
     *
     * The database currently represents profile coverage as one
     * continuous interval:
     *
     *     verified_from_litres .. verified_to_litres
     *
     * For the first completed session, use that session directly.
     *
     * For subsequent sessions, extend the existing range.
     */
    let profile_has_existing_coverage =
        session_context.verified_to_litres > session_context.verified_from_litres;

    let new_verified_from_litres;
    let new_verified_to_litres;

    if profile_has_existing_coverage {
        /*
         * The current database model stores calibration coverage as one
         * continuous verified interval.
         *
         * Therefore a new completed session must touch or overlap the
         * existing verified interval before those ranges may be merged.
         *
         * Examples:
         *
         * existing: 100 -> 140
         * new:      140 -> 180
         *
         * valid because the ranges touch.
         *
         * existing: 100 -> 140
         * new:      120 -> 160
         *
         * valid because the ranges overlap.
         *
         * existing: 100 -> 140
         * new:      20 -> 40
         *
         * invalid because 40 -> 100 has not been verified.
         */
        const COVERAGE_TOLERANCE_LITRES: f64 = 0.000_001;

        let session_ends_before_existing = session_verified_to_litres
            < session_context.verified_from_litres - COVERAGE_TOLERANCE_LITRES;

        let session_starts_after_existing = session_verified_from_litres
            > session_context.verified_to_litres + COVERAGE_TOLERANCE_LITRES;

        if session_ends_before_existing || session_starts_after_existing {
            return Err(anyhow!(
                "Completed calibration session does not connect to the profile's existing verified fuel range."
            ));
        }

        new_verified_from_litres = session_context
            .verified_from_litres
            .min(session_verified_from_litres);

        new_verified_to_litres = session_context
            .verified_to_litres
            .max(session_verified_to_litres);
    } else {
        /*
         * This is the first completed guided calibration session for the
         * profile, so its verified interval establishes the initial
         * coverage.
         */
        new_verified_from_litres = session_verified_from_litres;
        new_verified_to_litres = session_verified_to_litres;
    }

    let verified_range_litres = new_verified_to_litres - new_verified_from_litres;

    let coverage_percentage = verified_range_litres / session_context.tank_capacity_litres * 100.0;

    /*
     * Mark the guided session as completed.
     */
    sqlx::query!(
        r#"
        UPDATE fuel_calibration_sessions
        SET
            status = 'completed',
            completed_at = NOW(),
            updated_at = NOW()
        WHERE id = $1
        "#,
        session_id,
    )
    .execute(&mut *transaction)
    .await?;

    /*
     * The profile now contains completed calibration evidence.
     *
     * It therefore leaves DRAFT and becomes PROGRESSIVE.
     *
     * We deliberately do NOT mark it VALIDATED here.
     *
     * Completing a guided session means:
     *
     *     verified evidence exists
     *
     * It does not automatically mean:
     *
     *     a publishable lookup table has been constructed and approved.
     */
    sqlx::query!(
        r#"
        UPDATE fuel_calibration_profiles
        SET
            status = CASE
                WHEN status = 'draft' THEN 'progressive'
                ELSE status
            END,
            verified_from_litres = $2,
            verified_to_litres = $3,
            coverage_percentage = $4,
            updated_at = NOW()
        WHERE id = $1
        "#,
        session_context.profile_id,
        new_verified_from_litres,
        new_verified_to_litres,
        coverage_percentage,
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(())
}
