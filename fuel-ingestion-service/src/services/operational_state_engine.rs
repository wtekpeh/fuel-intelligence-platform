use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::repository::{
    delete_operational_state_candidate, get_operational_state_candidate,
    upsert_operational_state_candidate,
};
use crate::services::device_state::DeviceOperationalState;

const REQUIRED_CONSECUTIVE_OBSERVATIONS: i32 = 3;

#[derive(Debug)]
pub struct OperationalStateDecision {
    pub confirmed_state: DeviceOperationalState,
    pub transition_confirmed: bool,
}

pub async fn confirm_operational_state(
    db_pool: &PgPool,
    device_id: Uuid,
    current_confirmed_state: Option<DeviceOperationalState>,
    classified_state: DeviceOperationalState,
    observed_at: DateTime<Utc>,
) -> Result<OperationalStateDecision> {
    let Some(current_confirmed_state) = current_confirmed_state else {
        delete_operational_state_candidate(db_pool, device_id).await?;

        return Ok(OperationalStateDecision {
            confirmed_state: classified_state,
            transition_confirmed: true,
        });
    };

    if classified_state == current_confirmed_state {
        delete_operational_state_candidate(db_pool, device_id).await?;

        return Ok(OperationalStateDecision {
            confirmed_state: current_confirmed_state,
            transition_confirmed: false,
        });
    }

    let classified_state_string = classified_state.as_str();

    let existing_candidate = get_operational_state_candidate(db_pool, device_id).await?;

    let matching_candidate = existing_candidate
        .as_ref()
        .filter(|candidate| candidate.candidate_state == classified_state_string);

    let next_observation_count = matching_candidate
        .map(|candidate| candidate.observation_count + 1)
        .unwrap_or(1);

    if next_observation_count >= REQUIRED_CONSECUTIVE_OBSERVATIONS {
        delete_operational_state_candidate(db_pool, device_id).await?;

        return Ok(OperationalStateDecision {
            confirmed_state: classified_state,
            transition_confirmed: true,
        });
    }

    let first_observed_at = matching_candidate
        .map(|candidate| candidate.first_observed_at)
        .unwrap_or(observed_at);

    upsert_operational_state_candidate(
        db_pool,
        device_id,
        classified_state_string,
        next_observation_count,
        first_observed_at,
        observed_at,
    )
    .await?;

    Ok(OperationalStateDecision {
        confirmed_state: current_confirmed_state,
        transition_confirmed: false,
    })
}
