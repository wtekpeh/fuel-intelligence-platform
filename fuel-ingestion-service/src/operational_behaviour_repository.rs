use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::operational_behaviour::{
    BehaviourProfile, BehaviourSample, BehaviourType, LearningStatus,
    OperationalBehaviourLearningSession,
};

use crate::domain::telemetry::motion_buffer::MotionEvidence;
/// Creates a new operational behaviour learning session.
///
/// A newly created session begins in the `NOT_STARTED` state. The application
/// service will explicitly move it into `COLLECTING` when learning begins.
///
#[derive(Debug)]
pub enum BehaviourSampleRecordOutcome {
    Recorded(OperationalBehaviourLearningSession),
    SkippedBeforeSessionStart,
    SkippedSessionFull,
}

pub async fn create_learning_session(
    db_pool: &PgPool,
    device_id: Uuid,
    sensor_id: Uuid,
    behaviour_type: BehaviourType,
    requested_sample_count: i32,
) -> Result<OperationalBehaviourLearningSession> {
    let session_id = Uuid::new_v4();

    let row = sqlx::query!(
        r#"
        INSERT INTO operational_behaviour_learning_sessions (
            id,
            device_id,
            sensor_id,
            behaviour_type,
            status,
            requested_sample_count
        )
        VALUES (
            $1,
            $2,
            $3,
            $4,
            'NOT_STARTED',
            $5
        )
        RETURNING
            id,
            device_id,
            sensor_id,
            behaviour_type,
            status,
            requested_sample_count,
            collected_sample_count,
            started_at,
            completed_at,
            failure_reason,
            created_at,
            updated_at
        "#,
        session_id,
        device_id,
        sensor_id,
        behaviour_type.as_str(),
        requested_sample_count,
    )
    .fetch_one(db_pool)
    .await?;

    map_learning_session(
        row.id,
        row.device_id,
        row.sensor_id,
        row.behaviour_type,
        row.status,
        row.requested_sample_count,
        row.collected_sample_count,
        row.started_at,
        row.completed_at,
        row.failure_reason,
        row.created_at,
        row.updated_at,
    )
}

/// Returns one learning session by its identifier.
pub async fn get_learning_session(
    db_pool: &PgPool,
    learning_session_id: Uuid,
) -> Result<Option<OperationalBehaviourLearningSession>> {
    let row = sqlx::query!(
        r#"
        SELECT
            id,
            device_id,
            sensor_id,
            behaviour_type,
            status,
            requested_sample_count,
            collected_sample_count,
            started_at,
            completed_at,
            failure_reason,
            created_at,
            updated_at
        FROM operational_behaviour_learning_sessions
        WHERE id = $1
        "#,
        learning_session_id,
    )
    .fetch_optional(db_pool)
    .await?;

    row.map(|row| {
        map_learning_session(
            row.id,
            row.device_id,
            row.sensor_id,
            row.behaviour_type,
            row.status,
            row.requested_sample_count,
            row.collected_sample_count,
            row.started_at,
            row.completed_at,
            row.failure_reason,
            row.created_at,
            row.updated_at,
        )
    })
    .transpose()
}

/// Returns the currently collecting learning session for a sensor.
///
/// The query returns at most one row because the service should not allow two
/// simultaneous learning sessions for the same sensor.
pub async fn get_active_learning_session(
    db_pool: &PgPool,
    sensor_id: Uuid,
) -> Result<Option<OperationalBehaviourLearningSession>> {
    let row = sqlx::query!(
        r#"
        SELECT
            id,
            device_id,
            sensor_id,
            behaviour_type,
            status,
            requested_sample_count,
            collected_sample_count,
            started_at,
            completed_at,
            failure_reason,
            created_at,
            updated_at
        FROM operational_behaviour_learning_sessions
        WHERE sensor_id = $1
          AND status = 'COLLECTING'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        sensor_id,
    )
    .fetch_optional(db_pool)
    .await?;

    row.map(|row| {
        map_learning_session(
            row.id,
            row.device_id,
            row.sensor_id,
            row.behaviour_type,
            row.status,
            row.requested_sample_count,
            row.collected_sample_count,
            row.started_at,
            row.completed_at,
            row.failure_reason,
            row.created_at,
            row.updated_at,
        )
    })
    .transpose()
}

/// Changes a learning session from `NOT_STARTED` to `COLLECTING`.
///
/// A session that is already collecting, completed, or failed is not modified.
pub async fn start_learning_session(
    db_pool: &PgPool,
    learning_session_id: Uuid,
) -> Result<Option<OperationalBehaviourLearningSession>> {
    let row = sqlx::query!(
        r#"
        UPDATE operational_behaviour_learning_sessions
        SET
            status = 'COLLECTING',
            started_at = NOW(),
            failure_reason = NULL,
            updated_at = NOW()
        WHERE id = $1
          AND status = 'NOT_STARTED'
        RETURNING
            id,
            device_id,
            sensor_id,
            behaviour_type,
            status,
            requested_sample_count,
            collected_sample_count,
            started_at,
            completed_at,
            failure_reason,
            created_at,
            updated_at
        "#,
        learning_session_id,
    )
    .fetch_optional(db_pool)
    .await?;

    row.map(|row| {
        map_learning_session(
            row.id,
            row.device_id,
            row.sensor_id,
            row.behaviour_type,
            row.status,
            row.requested_sample_count,
            row.collected_sample_count,
            row.started_at,
            row.completed_at,
            row.failure_reason,
            row.created_at,
            row.updated_at,
        )
    })
    .transpose()
}

/// Loads all behaviour samples collected for one learning session.
///
/// Samples are returned in their original collection order so that
/// diagnostics, replay, and future learning algorithms can inspect
/// the sequence consistently.
pub async fn list_behaviour_samples(
    db_pool: &PgPool,
    learning_session_id: Uuid,
) -> Result<Vec<BehaviourSample>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            recorded_at,
            vibration_score,
            average_gravity_deviation_g,
            average_rotation_magnitude_dps,
            motion_ratio,
            average_confidence,
            sustained_motion,
            motion_sample_count,
            gps_speed_kmh,
            sample_index
        FROM operational_behaviour_samples
        WHERE learning_session_id = $1
        ORDER BY sample_index ASC
        "#,
        learning_session_id,
    )
    .fetch_all(db_pool)
    .await?;

    let samples = rows
        .into_iter()
        .map(|row| {
            let sample_count = usize::try_from(row.motion_sample_count).map_err(|_| {
                anyhow!(
                    "Invalid motion sample count {} stored for learning session {}.",
                    row.motion_sample_count,
                    learning_session_id
                )
            })?;

            Ok(BehaviourSample {
                recorded_at: row.recorded_at,

                motion_evidence: MotionEvidence {
                    average_vibration_score: row.vibration_score,

                    average_gravity_deviation_g: row.average_gravity_deviation_g,
                    average_rotation_magnitude_dps: row.average_rotation_magnitude_dps,

                    motion_ratio: row.motion_ratio,
                    average_confidence: row.average_confidence,
                    sustained_motion: row.sustained_motion,
                    sample_count,
                },

                gps_speed_kmh: row.gps_speed_kmh,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(samples)
}

/// Creates or replaces the learned profile for one device, sensor,
/// and operational behaviour.
///
/// The database has a unique constraint on:
///
/// device_id + sensor_id + behaviour_type
///
/// Therefore, retraining a behaviour updates the existing profile
/// rather than creating multiple active profiles.
pub async fn save_behaviour_profile(db_pool: &PgPool, profile: &BehaviourProfile) -> Result<Uuid> {
    let sample_count = i32::try_from(profile.statistics.sample_count).map_err(|_| {
        anyhow!(
            "Behaviour profile sample count {} exceeds the PostgreSQL INTEGER range.",
            profile.statistics.sample_count
        )
    })?;

    let profile_id = sqlx::query_scalar!(
        r#"
        INSERT INTO operational_behaviour_profiles (
            id,
            device_id,
            sensor_id,
            behaviour_type,
            learning_session_id,
            sample_count,
            average_vibration_score,
            minimum_vibration_score,
            maximum_vibration_score,
            vibration_variance,
            vibration_standard_deviation,
            average_gravity_deviation_g,
            minimum_gravity_deviation_g,
            maximum_gravity_deviation_g,
            gravity_deviation_variance,
            gravity_deviation_standard_deviation,

            average_rotation_magnitude_dps,
            minimum_rotation_magnitude_dps,
            maximum_rotation_magnitude_dps,
            rotation_magnitude_variance,
            rotation_magnitude_standard_deviation,
            average_motion_ratio,
            minimum_motion_ratio,
            maximum_motion_ratio,
            average_confidence,
            sustained_motion_ratio,
            average_gps_speed_kmh,
            learned_at
        )
        VALUES (
            $1,
            $2,
            $3,
            $4,
            $5,
            $6,
            $7,
            $8,
            $9,
            $10,
            $11,
            $12,
            $13,
            $14,
            $15,
            $16,
            $17,
            $18,
            $19,
            $20,
            $21,
            $22,
            $23,
            $24,
            $25,
            $26,
            $27,
            $28
        )
        ON CONFLICT (
            device_id,
            sensor_id,
            behaviour_type
        )
        DO UPDATE
        SET
            learning_session_id = EXCLUDED.learning_session_id,
            sample_count = EXCLUDED.sample_count,
            average_vibration_score = EXCLUDED.average_vibration_score,
            minimum_vibration_score = EXCLUDED.minimum_vibration_score,
            maximum_vibration_score = EXCLUDED.maximum_vibration_score,
            vibration_variance = EXCLUDED.vibration_variance,
            vibration_standard_deviation =
                EXCLUDED.vibration_standard_deviation,
            average_gravity_deviation_g =
                EXCLUDED.average_gravity_deviation_g,

            minimum_gravity_deviation_g =
                EXCLUDED.minimum_gravity_deviation_g,

            maximum_gravity_deviation_g =
                EXCLUDED.maximum_gravity_deviation_g,

            gravity_deviation_variance =
                EXCLUDED.gravity_deviation_variance,

            gravity_deviation_standard_deviation =
                EXCLUDED.gravity_deviation_standard_deviation,

            average_rotation_magnitude_dps =
                EXCLUDED.average_rotation_magnitude_dps,

            minimum_rotation_magnitude_dps =
                EXCLUDED.minimum_rotation_magnitude_dps,

            maximum_rotation_magnitude_dps =
                EXCLUDED.maximum_rotation_magnitude_dps,

            rotation_magnitude_variance =
                EXCLUDED.rotation_magnitude_variance,

            rotation_magnitude_standard_deviation =
                EXCLUDED.rotation_magnitude_standard_deviation,
            average_motion_ratio = EXCLUDED.average_motion_ratio,
            minimum_motion_ratio = EXCLUDED.minimum_motion_ratio,
            maximum_motion_ratio = EXCLUDED.maximum_motion_ratio,
            average_confidence = EXCLUDED.average_confidence,
            sustained_motion_ratio = EXCLUDED.sustained_motion_ratio,
            average_gps_speed_kmh = EXCLUDED.average_gps_speed_kmh,
            learned_at = EXCLUDED.learned_at,
            updated_at = NOW()
        RETURNING id
        "#,
        profile.id,
        profile.device_id,
        profile.sensor_id,
        profile.behaviour_type.as_str(),
        profile.learning_session_id,
        sample_count,
        profile.statistics.average_vibration_score,
        profile.statistics.minimum_vibration_score,
        profile.statistics.maximum_vibration_score,
        profile.statistics.vibration_variance,
        profile.statistics.vibration_standard_deviation,
        profile.statistics.average_gravity_deviation_g,
        profile.statistics.minimum_gravity_deviation_g,
        profile.statistics.maximum_gravity_deviation_g,
        profile.statistics.gravity_deviation_variance,
        profile.statistics.gravity_deviation_standard_deviation,
        profile.statistics.average_rotation_magnitude_dps,
        profile.statistics.minimum_rotation_magnitude_dps,
        profile.statistics.maximum_rotation_magnitude_dps,
        profile.statistics.rotation_magnitude_variance,
        profile.statistics.rotation_magnitude_standard_deviation,
        profile.statistics.average_motion_ratio,
        profile.statistics.minimum_motion_ratio,
        profile.statistics.maximum_motion_ratio,
        profile.statistics.average_confidence,
        profile.statistics.sustained_motion_ratio,
        profile.statistics.average_gps_speed_kmh,
        profile.learned_at,
    )
    .fetch_one(db_pool)
    .await?;

    Ok(profile_id)
}

/// Loads all learned operational behaviour profiles for one device
/// and vibration sensor.
///
/// The adaptive classifier uses the returned PARKED, IDLE, and MOVING
/// profiles to compare current physical motion evidence against the
/// device's learned behaviour.
pub async fn list_behaviour_profiles(
    db_pool: &PgPool,
    device_id: Uuid,
    sensor_id: Uuid,
) -> Result<Vec<BehaviourProfile>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            id,
            device_id,
            sensor_id,
            behaviour_type,
            learning_session_id,
            sample_count,

            average_vibration_score,
            minimum_vibration_score,
            maximum_vibration_score,
            vibration_variance,
            vibration_standard_deviation,

            average_gravity_deviation_g,
            minimum_gravity_deviation_g,
            maximum_gravity_deviation_g,
            gravity_deviation_variance,
            gravity_deviation_standard_deviation,

            average_rotation_magnitude_dps,
            minimum_rotation_magnitude_dps,
            maximum_rotation_magnitude_dps,
            rotation_magnitude_variance,
            rotation_magnitude_standard_deviation,

            average_motion_ratio,
            minimum_motion_ratio,
            maximum_motion_ratio,
            average_confidence,
            sustained_motion_ratio,
            average_gps_speed_kmh,
            learned_at
        FROM operational_behaviour_profiles
        WHERE device_id = $1
          AND sensor_id = $2
        ORDER BY behaviour_type ASC
        "#,
        device_id,
        sensor_id,
    )
    .fetch_all(db_pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let behaviour_type = BehaviourType::from_str(&row.behaviour_type).ok_or_else(|| {
                anyhow!(
                    "Unsupported operational behaviour type stored in profile {}: {}",
                    row.id,
                    row.behaviour_type
                )
            })?;

            let sample_count = usize::try_from(row.sample_count).map_err(|_| {
                anyhow!(
                    "Invalid sample count {} stored for behaviour profile {}.",
                    row.sample_count,
                    row.id
                )
            })?;

            Ok(BehaviourProfile {
                id: row.id,
                device_id: row.device_id,
                sensor_id: row.sensor_id,
                behaviour_type,
                learning_session_id: row.learning_session_id,

                statistics: crate::domain::operational_behaviour::BehaviourProfileStatistics {
                    sample_count,

                    average_vibration_score: row.average_vibration_score,
                    minimum_vibration_score: row.minimum_vibration_score,
                    maximum_vibration_score: row.maximum_vibration_score,
                    vibration_variance: row.vibration_variance,
                    vibration_standard_deviation: row.vibration_standard_deviation,

                    average_gravity_deviation_g: row.average_gravity_deviation_g,
                    minimum_gravity_deviation_g: row.minimum_gravity_deviation_g,
                    maximum_gravity_deviation_g: row.maximum_gravity_deviation_g,
                    gravity_deviation_variance: row.gravity_deviation_variance,
                    gravity_deviation_standard_deviation: row.gravity_deviation_standard_deviation,

                    average_rotation_magnitude_dps: row.average_rotation_magnitude_dps,
                    minimum_rotation_magnitude_dps: row.minimum_rotation_magnitude_dps,
                    maximum_rotation_magnitude_dps: row.maximum_rotation_magnitude_dps,
                    rotation_magnitude_variance: row.rotation_magnitude_variance,
                    rotation_magnitude_standard_deviation: row
                        .rotation_magnitude_standard_deviation,

                    average_motion_ratio: row.average_motion_ratio,
                    minimum_motion_ratio: row.minimum_motion_ratio,
                    maximum_motion_ratio: row.maximum_motion_ratio,

                    average_confidence: row.average_confidence,

                    sustained_motion_ratio: row.sustained_motion_ratio,

                    average_gps_speed_kmh: row.average_gps_speed_kmh,
                },

                learned_at: row.learned_at,
            })
        })
        .collect()
}

/// Persists one behaviour sample and increments the learning-session
/// sample count as one atomic database transaction.
///
/// The session row is locked while the operation is running so that
/// concurrent telemetry batches cannot assign the same sample index
/// or increment the counter beyond the requested sample count.
pub async fn record_behaviour_sample(
    db_pool: &PgPool,
    learning_session_id: Uuid,
    sample: &BehaviourSample,
) -> Result<BehaviourSampleRecordOutcome> {
    let mut transaction = db_pool.begin().await?;

    let session = sqlx::query!(
        r#"
        SELECT
            id,
            device_id,
            sensor_id,
            behaviour_type,
            status,
            requested_sample_count,
            collected_sample_count,
            started_at,
            completed_at,
            failure_reason,
            created_at,
            updated_at
        FROM operational_behaviour_learning_sessions
        WHERE id = $1
        FOR UPDATE
        "#,
        learning_session_id,
    )
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or_else(|| {
        anyhow!(
            "Operational behaviour learning session {} was not found.",
            learning_session_id
        )
    })?;

    if session.status != LearningStatus::Collecting.as_str() {
        return Err(anyhow!(
            "Cannot record a behaviour sample for session {} because its status is {}.",
            learning_session_id,
            session.status
        ));
    }

    if let Some(started_at) = session.started_at
        && sample.recorded_at < started_at
    {
        transaction.rollback().await?;

        return Ok(BehaviourSampleRecordOutcome::SkippedBeforeSessionStart);
    }
    if session.collected_sample_count >= session.requested_sample_count {
        transaction.rollback().await?;

        return Ok(BehaviourSampleRecordOutcome::SkippedSessionFull);
    }

    let sample_index = session.collected_sample_count + 1;

    sqlx::query!(
        r#"
    INSERT INTO operational_behaviour_samples (
        learning_session_id,
        recorded_at,
        vibration_score,
        average_gravity_deviation_g,
        average_rotation_magnitude_dps,
        motion_ratio,
        average_confidence,
        sustained_motion,
        motion_sample_count,
        gps_speed_kmh,
        sample_index
    )
    VALUES (
        $1,
        $2,
        $3,
        $4,
        $5,
        $6,
        $7,
        $8,
        $9,
        $10,
        $11
    )
    "#,
        learning_session_id,
        sample.recorded_at,
        sample.motion_evidence.average_vibration_score,
        sample.motion_evidence.average_gravity_deviation_g,
        sample.motion_evidence.average_rotation_magnitude_dps,
        sample.motion_evidence.motion_ratio,
        sample.motion_evidence.average_confidence,
        sample.motion_evidence.sustained_motion,
        sample.motion_evidence.sample_count as i32,
        sample.gps_speed_kmh,
        sample_index,
    )
    .execute(&mut *transaction)
    .await?;

    let updated_session = sqlx::query!(
        r#"
        UPDATE operational_behaviour_learning_sessions
        SET
            collected_sample_count = $2,
            updated_at = NOW()
        WHERE id = $1
        RETURNING
            id,
            device_id,
            sensor_id,
            behaviour_type,
            status,
            requested_sample_count,
            collected_sample_count,
            started_at,
            completed_at,
            failure_reason,
            created_at,
            updated_at
        "#,
        learning_session_id,
        sample_index,
    )
    .fetch_one(&mut *transaction)
    .await?;

    transaction.commit().await?;

    let session = map_learning_session(
        updated_session.id,
        updated_session.device_id,
        updated_session.sensor_id,
        updated_session.behaviour_type,
        updated_session.status,
        updated_session.requested_sample_count,
        updated_session.collected_sample_count,
        updated_session.started_at,
        updated_session.completed_at,
        updated_session.failure_reason,
        updated_session.created_at,
        updated_session.updated_at,
    )?;

    Ok(BehaviourSampleRecordOutcome::Recorded(session))
}

/// Marks a collecting learning session as completed.
pub async fn complete_learning_session(
    db_pool: &PgPool,
    learning_session_id: Uuid,
) -> Result<Option<OperationalBehaviourLearningSession>> {
    let row = sqlx::query!(
        r#"
        UPDATE operational_behaviour_learning_sessions
        SET
            status = 'COMPLETED',
            completed_at = NOW(),
            failure_reason = NULL,
            updated_at = NOW()
        WHERE id = $1
          AND status = 'COLLECTING'
        RETURNING
            id,
            device_id,
            sensor_id,
            behaviour_type,
            status,
            requested_sample_count,
            collected_sample_count,
            started_at,
            completed_at,
            failure_reason,
            created_at,
            updated_at
        "#,
        learning_session_id,
    )
    .fetch_optional(db_pool)
    .await?;

    row.map(|row| {
        map_learning_session(
            row.id,
            row.device_id,
            row.sensor_id,
            row.behaviour_type,
            row.status,
            row.requested_sample_count,
            row.collected_sample_count,
            row.started_at,
            row.completed_at,
            row.failure_reason,
            row.created_at,
            row.updated_at,
        )
    })
    .transpose()
}

/// Marks a learning session as failed and stores the reason.
pub async fn fail_learning_session(
    db_pool: &PgPool,
    learning_session_id: Uuid,
    failure_reason: &str,
) -> Result<Option<OperationalBehaviourLearningSession>> {
    let row = sqlx::query!(
        r#"
        UPDATE operational_behaviour_learning_sessions
        SET
            status = 'FAILED',
            failure_reason = $2,
            completed_at = NOW(),
            updated_at = NOW()
        WHERE id = $1
          AND status IN ('NOT_STARTED', 'COLLECTING')
        RETURNING
            id,
            device_id,
            sensor_id,
            behaviour_type,
            status,
            requested_sample_count,
            collected_sample_count,
            started_at,
            completed_at,
            failure_reason,
            created_at,
            updated_at
        "#,
        learning_session_id,
        failure_reason,
    )
    .fetch_optional(db_pool)
    .await?;

    row.map(|row| {
        map_learning_session(
            row.id,
            row.device_id,
            row.sensor_id,
            row.behaviour_type,
            row.status,
            row.requested_sample_count,
            row.collected_sample_count,
            row.started_at,
            row.completed_at,
            row.failure_reason,
            row.created_at,
            row.updated_at,
        )
    })
    .transpose()
}

#[allow(clippy::too_many_arguments)]
fn map_learning_session(
    id: Uuid,
    device_id: Uuid,
    sensor_id: Uuid,
    behaviour_type: String,
    status: String,
    requested_sample_count: i32,
    collected_sample_count: i32,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    failure_reason: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
) -> Result<OperationalBehaviourLearningSession> {
    let behaviour_type = BehaviourType::from_str(&behaviour_type).ok_or_else(|| {
        anyhow!(
            "Unsupported operational behaviour type stored in database: {}",
            behaviour_type
        )
    })?;

    let status = LearningStatus::from_str(&status).ok_or_else(|| {
        anyhow!(
            "Unsupported operational behaviour learning status stored in database: {}",
            status
        )
    })?;

    Ok(OperationalBehaviourLearningSession {
        id,
        device_id,
        sensor_id,
        behaviour_type,
        status,
        requested_sample_count,
        collected_sample_count,
        started_at,
        completed_at,
        failure_reason,
        created_at,
        updated_at,
    })
}
