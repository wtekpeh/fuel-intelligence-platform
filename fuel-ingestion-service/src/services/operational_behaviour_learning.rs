use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::telemetry::motion_buffer::MotionEvidence;

use crate::{
    domain::operational_behaviour::{
        BehaviourProfile, BehaviourProfileBuildError, BehaviourProfileBuilder, BehaviourSample,
    },
    operational_behaviour_repository,
};

use crate::operational_behaviour_repository::BehaviourSampleRecordOutcome;

/// Coordinates installer-guided Operational Behaviour Learning.
///
/// This service contains no statistical or persistence logic.
/// It orchestrates the domain builder and repository operations.
pub struct OperationalBehaviourLearningService {
    db_pool: PgPool,
}

impl OperationalBehaviourLearningService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Processes one behaviour observation for a vibration sensor.
    ///
    /// When no learning session is active for the sensor, the sample
    /// is ignored and normal telemetry processing continues.
    ///
    /// When the requested number of samples has been collected, this
    /// method builds and persists the learned behaviour profile and
    /// completes the learning session.
    pub async fn process_sample(&self, sensor_id: Uuid, sample: BehaviourSample) -> Result<()> {
        let Some(session) =
            operational_behaviour_repository::get_active_learning_session(&self.db_pool, sensor_id)
                .await?
        else {
            return Ok(());
        };

        let record_outcome = operational_behaviour_repository::record_behaviour_sample(
            &self.db_pool,
            session.id,
            &sample,
        )
        .await?;

        let updated_session = match record_outcome {
            BehaviourSampleRecordOutcome::Recorded(session) => session,

            BehaviourSampleRecordOutcome::SkippedBeforeSessionStart => {
                println!(
                    "[BEHAVIOUR LEARNING SKIPPED] session_id={}, \
            reason=reading_before_session_start, recorded_at={}",
                    session.id, sample.recorded_at,
                );

                return Ok(());
            }

            BehaviourSampleRecordOutcome::SkippedSessionFull => {
                return Ok(());
            }
        };

        if updated_session.collected_sample_count < updated_session.requested_sample_count {
            return Ok(());
        }

        let samples = operational_behaviour_repository::list_behaviour_samples(
            &self.db_pool,
            updated_session.id,
        )
        .await?;

        let statistics =
            BehaviourProfileBuilder::build(&samples).map_err(map_profile_build_error)?;

        let profile = BehaviourProfile {
            id: Uuid::new_v4(),
            device_id: updated_session.device_id,
            sensor_id: updated_session.sensor_id,
            behaviour_type: updated_session.behaviour_type,
            learning_session_id: updated_session.id,
            statistics,
            learned_at: Utc::now(),
        };

        operational_behaviour_repository::save_behaviour_profile(&self.db_pool, &profile).await?;

        let completed_session = operational_behaviour_repository::complete_learning_session(
            &self.db_pool,
            updated_session.id,
        )
        .await?;

        if completed_session.is_none() {
            return Err(anyhow!(
                "Behaviour profile was saved, but learning session {} could not be marked as completed.",
                updated_session.id
            ));
        }

        println!(
            "[BEHAVIOUR LEARNING COMPLETED] device_id={}, sensor_id={}, behaviour={}, samples={}, profile_id={}",
            profile.device_id,
            profile.sensor_id,
            profile.behaviour_type.as_str(),
            profile.statistics.sample_count,
            profile.id,
        );

        Ok(())
    }

    /// Converts canonical motion evidence into a behaviour-learning sample.
    ///
    /// This is the integration boundary between the telemetry domain and
    /// Operational Behaviour Learning.
    ///
    /// If no rolling motion evidence is available yet, learning is skipped.
    /// Normal operational-state processing remains unaffected.
    pub async fn process_motion_evidence(
        &self,
        sensor_id: Uuid,
        recorded_at: DateTime<Utc>,
        motion_evidence: Option<&MotionEvidence>,
        gps_speed_kmh: Option<f64>,
    ) -> Result<()> {
        let Some(motion_evidence) = motion_evidence else {
            return Ok(());
        };

        const REQUIRED_MOTION_WINDOW_SIZE: usize = 5;

        if motion_evidence.sample_count < REQUIRED_MOTION_WINDOW_SIZE {
            return Ok(());
        }

        let sample = BehaviourSample {
            recorded_at,
            motion_evidence: motion_evidence.clone(),
            gps_speed_kmh,
        };

        self.process_sample(sensor_id, sample).await
    }
}

fn map_profile_build_error(error: BehaviourProfileBuildError) -> anyhow::Error {
    match error {
        BehaviourProfileBuildError::InsufficientSamples { required, actual } => anyhow!(
            "Cannot build operational behaviour profile: at least {} samples are required, but only {} were collected.",
            required,
            actual
        ),
    }
}
