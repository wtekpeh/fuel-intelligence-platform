use crate::{
    domain::{operational_behaviour::BehaviourProfile, telemetry::motion_buffer::MotionEvidence},
    services::device_state::DeviceOperationalState,
};

/// Lowest learned scale allowed when normalising a physical metric.
///
/// Some behaviour profiles can be extremely stable, particularly PARKED.
/// A small floor protects the calculation when every learned profile has
/// zero or near-zero variance for a metric.
const MINIMUM_NORMALISATION_SCALE: f64 = 0.000_001;

#[derive(Debug, Clone, Copy)]
struct AdaptiveNormalisationScales {
    vibration: f64,
    gravity_deviation: f64,
    rotation_magnitude: f64,
}

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
    let normalisation_scales = calculate_normalisation_scales(profiles);

    profiles
        .iter()
        .filter_map(|profile| {
            let classified_state = behaviour_profile_state(profile)?;

            let distance =
                calculate_profile_distance(motion_evidence, profile, &normalisation_scales);

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
/// The adaptive classifier compares three learned physical metrics:
///
/// - vibration score
/// - gravity deviation
/// - gyroscope-vector magnitude
///
/// Each metric uses a shared normalisation scale derived from the complete
/// set of learned behaviour profiles. This prevents a high-variance profile
/// from gaining an artificial advantage simply because its own standard
/// deviation is larger.
///
/// Motion ratio and sustained-motion evidence remain available to the
/// rule-based fallback classifier.
fn calculate_profile_distance(
    motion_evidence: &MotionEvidence,
    profile: &BehaviourProfile,
    normalisation_scales: &AdaptiveNormalisationScales,
) -> f64 {
    let statistics = &profile.statistics;

    let vibration_distance = (motion_evidence.average_vibration_score
        - statistics.average_vibration_score)
        / normalisation_scales.vibration;

    let gravity_distance = (motion_evidence.average_gravity_deviation_g
        - statistics.average_gravity_deviation_g)
        / normalisation_scales.gravity_deviation;

    let rotation_distance = (motion_evidence.average_rotation_magnitude_dps
        - statistics.average_rotation_magnitude_dps)
        / normalisation_scales.rotation_magnitude;

    (vibration_distance.powi(2) + gravity_distance.powi(2) + rotation_distance.powi(2)).sqrt()
}

fn calculate_normalisation_scales(profiles: &[BehaviourProfile]) -> AdaptiveNormalisationScales {
    let vibration = profiles
        .iter()
        .map(|profile| profile.statistics.vibration_standard_deviation)
        .fold(0.0_f64, f64::max)
        .max(MINIMUM_NORMALISATION_SCALE);

    let gravity_deviation = profiles
        .iter()
        .map(|profile| profile.statistics.gravity_deviation_standard_deviation)
        .fold(0.0_f64, f64::max)
        .max(MINIMUM_NORMALISATION_SCALE);

    let rotation_magnitude = profiles
        .iter()
        .map(|profile| profile.statistics.rotation_magnitude_standard_deviation)
        .fold(0.0_f64, f64::max)
        .max(MINIMUM_NORMALISATION_SCALE);

    AdaptiveNormalisationScales {
        vibration,
        gravity_deviation,
        rotation_magnitude,
    }
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

    #[test]
    fn broad_moving_variance_does_not_attract_stationary_evidence() {
        let mut parked = profile(BehaviourType::Parked, 0.06895, 2.6457);
        parked.statistics.average_vibration_score = 0.0;
        parked.statistics.vibration_standard_deviation = 0.0;
        parked.statistics.gravity_deviation_standard_deviation = 0.000365;
        parked.statistics.rotation_magnitude_standard_deviation = 0.00814;

        let mut idle = profile(BehaviourType::Idle, 0.07370, 2.6374);
        idle.statistics.average_vibration_score = 0.0100;
        idle.statistics.vibration_standard_deviation = 0.00834;
        idle.statistics.gravity_deviation_standard_deviation = 0.00272;
        idle.statistics.rotation_magnitude_standard_deviation = 0.0711;

        let mut moving = profile(BehaviourType::Moving, 0.08499, 4.7682);
        moving.statistics.average_vibration_score = 0.4900;
        moving.statistics.vibration_standard_deviation = 0.2943;
        moving.statistics.gravity_deviation_standard_deviation = 0.01430;
        moving.statistics.rotation_magnitude_standard_deviation = 1.6995;

        // This represents quiet stationary evidence that has drifted somewhat
        // away from the extremely narrow PARKED profile.
        //
        // Under per-profile normalisation, MOVING could become artificially
        // attractive because its learned variance is much wider.
        let evidence = MotionEvidence {
            average_vibration_score: 0.005,
            average_gravity_deviation_g: 0.072,
            average_rotation_magnitude_dps: 2.66,
            motion_ratio: 0.0,
            average_confidence: 0.0,
            sustained_motion: false,
            sample_count: 5,
        };

        let result = classify_from_learned_profiles(&evidence, &[parked, idle.clone(), moving])
            .expect("profiles should produce a classification");

        assert_eq!(result.classified_state, DeviceOperationalState::Idle);

        assert_eq!(result.matched_profile_id, idle.id);
    }
}
