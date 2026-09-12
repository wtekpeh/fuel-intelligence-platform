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

fn is_stationary_state(state: &DeviceOperationalState) -> bool {
    matches!(
        state,
        DeviceOperationalState::Idle | DeviceOperationalState::Parked
    )
}

fn candidate_supports_same_transition(
    current_confirmed_state: &DeviceOperationalState,
    existing_candidate_state: &str,
    classified_state: &DeviceOperationalState,
) -> bool {
    // Normal behaviour:
    // the new classification exactly matches the existing candidate.
    if existing_candidate_state == classified_state.as_str() {
        return true;
    }

    // Special case when leaving MOVING:
    //
    // IDLE and PARKED are both evidence that the asset has stopped
    // travelling. A quiet engine can legitimately cause the classifier
    // to alternate between these two states while the vehicle remains
    // physically stationary.
    //
    // We therefore allow both states to contribute to the same
    // MOVING -> stationary transition confirmation.
    matches!(current_confirmed_state, DeviceOperationalState::Moving)
        && is_stationary_state(classified_state)
        && matches!(existing_candidate_state, "IDLE" | "PARKED")
}

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

    let matching_candidate = existing_candidate.as_ref().filter(|candidate| {
        candidate_supports_same_transition(
            &current_confirmed_state,
            &candidate.candidate_state,
            &classified_state,
        )
    });

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::device_state::DeviceOperationalState;

    #[test]
    fn stationary_states_share_transition_evidence_when_leaving_moving() {
        assert!(candidate_supports_same_transition(
            &DeviceOperationalState::Moving,
            DeviceOperationalState::Idle.as_str(),
            &DeviceOperationalState::Parked,
        ));

        assert!(candidate_supports_same_transition(
            &DeviceOperationalState::Moving,
            DeviceOperationalState::Parked.as_str(),
            &DeviceOperationalState::Idle,
        ));
    }

    #[test]
    fn exact_candidate_state_still_matches_normally() {
        assert!(candidate_supports_same_transition(
            &DeviceOperationalState::Moving,
            DeviceOperationalState::Idle.as_str(),
            &DeviceOperationalState::Idle,
        ));
    }

    #[test]
    fn idle_and_parked_do_not_share_transition_evidence_when_already_stationary() {
        assert!(!candidate_supports_same_transition(
            &DeviceOperationalState::Parked,
            DeviceOperationalState::Parked.as_str(),
            &DeviceOperationalState::Idle,
        ));

        assert!(!candidate_supports_same_transition(
            &DeviceOperationalState::Idle,
            DeviceOperationalState::Idle.as_str(),
            &DeviceOperationalState::Parked,
        ));
    }

    #[test]
    fn moving_candidate_does_not_match_stationary_transition() {
        assert!(!candidate_supports_same_transition(
            &DeviceOperationalState::Moving,
            DeviceOperationalState::Moving.as_str(),
            &DeviceOperationalState::Idle,
        ));
    }
}
