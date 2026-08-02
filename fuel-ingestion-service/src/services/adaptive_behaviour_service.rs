use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{
        operational_behaviour::{BehaviourProfile, BehaviourType},
        telemetry::motion_buffer::MotionEvidence,
    },
    operational_behaviour_repository,
    services::{
        adaptive_behaviour_classifier::classify_from_learned_profiles,
        device_state::{
            DeviceOperationalState, classify_device_state_from_motion,
            has_meaningful_gps_displacement,
        },
    },
};

const REQUIRED_PROFILE_SAMPLE_COUNT: usize = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviourClassificationSource {
    Gps,
    AdaptiveProfiles,
    RuleBasedFallback,
}

impl BehaviourClassificationSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gps => "GPS",
            Self::AdaptiveProfiles => "ADAPTIVE_PROFILES",
            Self::RuleBasedFallback => "RULE_BASED_FALLBACK",
        }
    }
}

#[derive(Debug)]
pub struct BehaviourClassificationDecision {
    pub state: DeviceOperationalState,
    pub source: BehaviourClassificationSource,
    pub matched_profile_id: Option<Uuid>,
    pub adaptive_distance: Option<f64>,
}

#[derive(Clone)]
pub struct AdaptiveBehaviourService {
    db_pool: PgPool,
}

impl AdaptiveBehaviourService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Classifies the current operational behaviour while preserving
    /// GPS-confirmed travel as authoritative.
    ///
    /// Adaptive profile matching is used only when valid PARKED, IDLE,
    /// and MOVING profiles all exist for the device and sensor.
    /// Otherwise, the existing rolling rule-based classifier is used.
    #[allow(clippy::too_many_arguments)]
    pub async fn classify(
        &self,
        device_id: Uuid,
        sensor_id: Uuid,
        device_status: Option<&str>,
        motion_evidence: Option<&MotionEvidence>,
        previous_latitude: Option<f64>,
        previous_longitude: Option<f64>,
        current_latitude: Option<f64>,
        current_longitude: Option<f64>,
    ) -> Result<BehaviourClassificationDecision> {
        if matches!(device_status, Some("OFFLINE")) {
            return Ok(BehaviourClassificationDecision {
                state: DeviceOperationalState::Offline,
                source: BehaviourClassificationSource::RuleBasedFallback,
                matched_profile_id: None,
                adaptive_distance: None,
            });
        }

        if has_meaningful_gps_displacement(
            previous_latitude,
            previous_longitude,
            current_latitude,
            current_longitude,
        ) {
            return Ok(BehaviourClassificationDecision {
                state: DeviceOperationalState::Moving,
                source: BehaviourClassificationSource::Gps,
                matched_profile_id: None,
                adaptive_distance: None,
            });
        }

        let Some(motion_evidence) = motion_evidence else {
            return Ok(BehaviourClassificationDecision {
                state: DeviceOperationalState::Unknown,
                source: BehaviourClassificationSource::RuleBasedFallback,
                matched_profile_id: None,
                adaptive_distance: None,
            });
        };

        let profiles = operational_behaviour_repository::list_behaviour_profiles(
            &self.db_pool,
            device_id,
            sensor_id,
        )
        .await?;

        if profiles_are_adaptive_ready(&profiles)
            && let Some(classification) = classify_from_learned_profiles(motion_evidence, &profiles)
        {
            return Ok(BehaviourClassificationDecision {
                state: classification.classified_state,
                source: BehaviourClassificationSource::AdaptiveProfiles,
                matched_profile_id: Some(classification.matched_profile_id),
                adaptive_distance: Some(classification.distance),
            });
        }

        let state = classify_device_state_from_motion(
            device_status,
            Some(motion_evidence),
            previous_latitude,
            previous_longitude,
            current_latitude,
            current_longitude,
        );

        Ok(BehaviourClassificationDecision {
            state,
            source: BehaviourClassificationSource::RuleBasedFallback,
            matched_profile_id: None,
            adaptive_distance: None,
        })
    }
}

fn profiles_are_adaptive_ready(profiles: &[BehaviourProfile]) -> bool {
    let parked = profiles
        .iter()
        .find(|profile| profile.behaviour_type == BehaviourType::Parked);

    let idle = profiles
        .iter()
        .find(|profile| profile.behaviour_type == BehaviourType::Idle);

    let moving = profiles
        .iter()
        .find(|profile| profile.behaviour_type == BehaviourType::Moving);

    [parked, idle, moving]
        .into_iter()
        .all(|profile| profile.is_some_and(profile_is_valid))
}

fn profile_is_valid(profile: &BehaviourProfile) -> bool {
    let statistics = &profile.statistics;

    statistics.sample_count >= REQUIRED_PROFILE_SAMPLE_COUNT
        && statistics.average_gravity_deviation_g.is_finite()
        && statistics.gravity_deviation_standard_deviation.is_finite()
        && statistics.average_rotation_magnitude_dps.is_finite()
        && statistics.rotation_magnitude_standard_deviation.is_finite()
        && statistics.average_gravity_deviation_g >= 0.0
        && statistics.average_rotation_magnitude_dps >= 0.0
        && (statistics.average_gravity_deviation_g > 0.0
            || statistics.average_rotation_magnitude_dps > 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{TimeZone, Utc};

    use crate::domain::operational_behaviour::{BehaviourProfileStatistics, BehaviourType};

    fn profile(behaviour_type: BehaviourType, gravity: f64, rotation: f64) -> BehaviourProfile {
        BehaviourProfile {
            id: Uuid::new_v4(),
            device_id: Uuid::new_v4(),
            sensor_id: Uuid::new_v4(),
            behaviour_type,
            learning_session_id: Uuid::new_v4(),

            statistics: BehaviourProfileStatistics {
                sample_count: 30,

                average_vibration_score: 0.0,
                minimum_vibration_score: 0.0,
                maximum_vibration_score: 0.0,
                vibration_variance: 0.0,
                vibration_standard_deviation: 0.0,

                average_gravity_deviation_g: gravity,
                minimum_gravity_deviation_g: gravity,
                maximum_gravity_deviation_g: gravity,
                gravity_deviation_variance: 0.0,
                gravity_deviation_standard_deviation: 0.01,

                average_rotation_magnitude_dps: rotation,
                minimum_rotation_magnitude_dps: rotation,
                maximum_rotation_magnitude_dps: rotation,
                rotation_magnitude_variance: 0.0,
                rotation_magnitude_standard_deviation: 0.1,

                average_motion_ratio: 0.0,
                minimum_motion_ratio: 0.0,
                maximum_motion_ratio: 0.0,
                average_confidence: 0.0,
                sustained_motion_ratio: 0.0,
                average_gps_speed_kmh: Some(0.0),
            },

            learned_at: Utc
                .with_ymd_and_hms(2026, 1, 1, 12, 0, 0)
                .single()
                .expect("test timestamp should be valid"),
        }
    }

    #[test]
    fn complete_physical_profiles_are_adaptive_ready() {
        let profiles = vec![
            profile(BehaviourType::Parked, 0.069, 2.65),
            profile(BehaviourType::Idle, 0.074, 2.64),
            profile(BehaviourType::Moving, 0.085, 4.77),
        ];

        assert!(profiles_are_adaptive_ready(&profiles));
    }

    #[test]
    fn missing_moving_profile_is_not_adaptive_ready() {
        let profiles = vec![
            profile(BehaviourType::Parked, 0.069, 2.65),
            profile(BehaviourType::Idle, 0.074, 2.64),
        ];

        assert!(!profiles_are_adaptive_ready(&profiles));
    }

    #[test]
    fn legacy_zero_profile_is_not_adaptive_ready() {
        let profiles = vec![
            profile(BehaviourType::Parked, 0.069, 2.65),
            profile(BehaviourType::Idle, 0.0, 0.0),
            profile(BehaviourType::Moving, 0.085, 4.77),
        ];

        assert!(!profiles_are_adaptive_ready(&profiles));
    }

    #[test]
    fn insufficient_sample_count_is_not_adaptive_ready() {
        let mut moving = profile(BehaviourType::Moving, 0.085, 4.77);

        moving.statistics.sample_count = 10;

        let profiles = vec![
            profile(BehaviourType::Parked, 0.069, 2.65),
            profile(BehaviourType::Idle, 0.074, 2.64),
            moving,
        ];

        assert!(!profiles_are_adaptive_ready(&profiles));
    }
}
