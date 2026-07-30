use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::operational_behaviour::{
    BehaviourType, LearningStatus, OperationalBehaviourLearningSession,
};

/// Creates a new operational behaviour learning session.
///
/// A newly created session begins in the `NOT_STARTED` state. The application
/// service will explicitly move it into `COLLECTING` when learning begins.
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
