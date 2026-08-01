use crate::{
    domain::{operational_behaviour::BehaviourProfile, telemetry::motion_buffer::MotionEvidence},
    services::device_state::DeviceOperationalState,
};

/// Lowest standard deviation allowed when normalising a metric.
///
/// Learned profiles can sometimes have extremely small or zero variance.
/// Applying this floor prevents division by zero and prevents one nearly
/// constant metric from dominating the complete similarity score.
const MINIMUM_STANDARD_DEVIATION: f64 = 0.000_001;

/// Result produced when live motion evidence is compared against one
/// or more learned operational behaviour profiles.
#[derive(Debug)]
pub struct AdaptiveBehaviourClassification {
    pub classified_state: DeviceOperationalState,

    /// Normalised distance between the live evidence and the selected
    /// learned profile. Lower values indicate a closer match.
    pub distance: f64,

    pub matched_profile_id: uuid::Uuid,
}

/// Compares live physical motion evidence against learned behaviour
/// profiles and returns the closest recognised operational behaviour.
///
/// This classifier contains no database access and does not confirm state
/// transitions. It only answers:
///
/// "Which learned behaviour profile most closely resembles the current
/// rolling motion evidence?"
pub fn classify_from_learned_profiles(
    motion_evidence: &MotionEvidence,
    profiles: &[BehaviourProfile],
) -> Option<AdaptiveBehaviourClassification> {
    profiles
        .iter()
        .filter_map(|profile| {
            let classified_state = behaviour_profile_state(profile)?;

            let distance = calculate_profile_distance(motion_evidence, profile);

            Some(AdaptiveBehaviourClassification {
                classified_state,
                distance,
                matched_profile_id: profile.id,
            })
        })
        .min_by(|left, right| left.distance.total_cmp(&right.distance))
}

/// Calculates a normalised Euclidean distance between current motion
/// evidence and one learned behaviour profile.
///
/// The first adaptive classifier deliberately uses the two pre-deadband
/// physical metrics that have now been validated with real hardware:
///
/// - gravity deviation
/// - gyroscope-vector magnitude
///
/// Vibration score and motion ratio remain available to the fallback
/// classifier while PARKED, IDLE, and MOVING physical profiles are
/// collected and validated.
fn calculate_profile_distance(motion_evidence: &MotionEvidence, profile: &BehaviourProfile) -> f64 {
    let statistics = &profile.statistics;

    let gravity_standard_deviation = statistics
        .gravity_deviation_standard_deviation
        .max(MINIMUM_STANDARD_DEVIATION);

    let rotation_standard_deviation = statistics
        .rotation_magnitude_standard_deviation
        .max(MINIMUM_STANDARD_DEVIATION);

    let gravity_distance = (motion_evidence.average_gravity_deviation_g
        - statistics.average_gravity_deviation_g)
        / gravity_standard_deviation;

    let rotation_distance = (motion_evidence.average_rotation_magnitude_dps
        - statistics.average_rotation_magnitude_dps)
        / rotation_standard_deviation;

    (gravity_distance.powi(2) + rotation_distance.powi(2)).sqrt()
}

fn behaviour_profile_state(profile: &BehaviourProfile) -> Option<DeviceOperationalState> {
    match profile.behaviour_type.as_str() {
        "PARKED" => Some(DeviceOperationalState::Parked),
        "IDLE" => Some(DeviceOperationalState::Idle),
        "MOVING" => Some(DeviceOperationalState::Moving),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use super::*;

    use crate::domain::operational_behaviour::{BehaviourProfileStatistics, BehaviourType};

    fn profile(
        behaviour_type: BehaviourType,
        gravity_mean: f64,
        rotation_mean: f64,
    ) -> BehaviourProfile {
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

                average_gravity_deviation_g: gravity_mean,
                minimum_gravity_deviation_g: gravity_mean - 0.01,
                maximum_gravity_deviation_g: gravity_mean + 0.01,
                gravity_deviation_variance: 0.0001,
                gravity_deviation_standard_deviation: 0.01,

                average_rotation_magnitude_dps: rotation_mean,
                minimum_rotation_magnitude_dps: rotation_mean - 0.2,
                maximum_rotation_magnitude_dps: rotation_mean + 0.2,
                rotation_magnitude_variance: 0.04,
                rotation_magnitude_standard_deviation: 0.2,

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

    fn live_evidence(gravity: f64, rotation: f64) -> MotionEvidence {
        MotionEvidence {
            average_vibration_score: 0.0,
            average_gravity_deviation_g: gravity,
            average_rotation_magnitude_dps: rotation,
            motion_ratio: 0.0,
            average_confidence: 0.0,
            sustained_motion: false,
            sample_count: 5,
        }
    }

    #[test]
    fn returns_none_when_no_profiles_exist() {
        let evidence = live_evidence(0.07, 2.6);

        let result = classify_from_learned_profiles(&evidence, &[]);

        assert!(result.is_none());
    }

    #[test]
    fn selects_closest_parked_profile() {
        let parked = profile(BehaviourType::Parked, 0.07, 2.6);

        let idle = profile(BehaviourType::Idle, 0.12, 4.0);

        let moving = profile(BehaviourType::Moving, 0.40, 12.0);

        let evidence = live_evidence(0.069, 2.65);

        let result = classify_from_learned_profiles(&evidence, &[parked.clone(), idle, moving])
            .expect("profiles should produce a classification");

        assert_eq!(result.classified_state, DeviceOperationalState::Parked);

        assert_eq!(result.matched_profile_id, parked.id);
    }

    #[test]
    fn selects_closest_idle_profile() {
        let parked = profile(BehaviourType::Parked, 0.07, 2.6);

        let idle = profile(BehaviourType::Idle, 0.12, 4.0);

        let moving = profile(BehaviourType::Moving, 0.40, 12.0);

        let evidence = live_evidence(0.118, 4.1);

        let result = classify_from_learned_profiles(&evidence, &[parked, idle.clone(), moving])
            .expect("profiles should produce a classification");

        assert_eq!(result.classified_state, DeviceOperationalState::Idle);

        assert_eq!(result.matched_profile_id, idle.id);
    }

    #[test]
    fn selects_closest_moving_profile() {
        let parked = profile(BehaviourType::Parked, 0.07, 2.6);

        let idle = profile(BehaviourType::Idle, 0.12, 4.0);

        let moving = profile(BehaviourType::Moving, 0.40, 12.0);

        let evidence = live_evidence(0.39, 11.8);

        let result = classify_from_learned_profiles(&evidence, &[parked, idle, moving.clone()])
            .expect("profiles should produce a classification");

        assert_eq!(result.classified_state, DeviceOperationalState::Moving);

        assert_eq!(result.matched_profile_id, moving.id);
    }

    #[test]
    fn zero_standard_deviation_does_not_divide_by_zero() {
        let mut parked = profile(BehaviourType::Parked, 0.07, 2.6);

        parked.statistics.gravity_deviation_standard_deviation = 0.0;

        parked.statistics.rotation_magnitude_standard_deviation = 0.0;

        let evidence = live_evidence(0.07, 2.6);

        let result = classify_from_learned_profiles(&evidence, &[parked])
            .expect("profile should produce a classification");

        assert!(result.distance.is_finite());
        assert_eq!(result.distance, 0.0);
    }
}
