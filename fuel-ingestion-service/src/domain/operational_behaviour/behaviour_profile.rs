use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::operational_behaviour::{BehaviourProfileStatistics, BehaviourType};

/// Canonical learned operational behaviour profile.
///
/// A behaviour profile represents the statistical signature
/// learned for one operational behaviour (PARKED, IDLE,
/// or MOVING) for a specific device and sensor.
///
/// The profile is independent of persistence and can be
/// reused by operational intelligence, analytics,
/// investigations and future AI learning.
#[derive(Debug, Clone, PartialEq)]
pub struct BehaviourProfile {
    /// Unique identifier of the learned profile.
    pub id: Uuid,

    /// Device the profile belongs to.
    pub device_id: Uuid,

    /// Sensor the profile belongs to.
    pub sensor_id: Uuid,

    /// Behaviour represented by this profile.
    pub behaviour_type: BehaviourType,

    /// Learning session that produced this profile.
    pub learning_session_id: Uuid,

    /// Complete statistical representation of the
    /// learned behaviour.
    pub statistics: BehaviourProfileStatistics,

    /// Time at which learning completed.
    pub learned_at: DateTime<Utc>,
}

impl BehaviourProfile {
    /// Returns the expected vibration score for this
    /// operational behaviour.
    pub fn expected_vibration_score(&self) -> f64 {
        self.statistics.average_vibration_score
    }

    /// Returns the expected motion ratio.
    pub fn expected_motion_ratio(&self) -> f64 {
        self.statistics.average_motion_ratio
    }

    /// Returns the vibration standard deviation.
    pub fn vibration_standard_deviation(&self) -> f64 {
        self.statistics.vibration_standard_deviation
    }

    /// Returns true when the learned behaviour is
    /// characterised by predominantly sustained motion.
    pub fn expects_sustained_motion(&self) -> bool {
        self.statistics.has_predominantly_sustained_motion()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn profile() -> BehaviourProfile {
        BehaviourProfile {
            id: Uuid::new_v4(),
            device_id: Uuid::new_v4(),
            sensor_id: Uuid::new_v4(),
            learning_session_id: Uuid::new_v4(),
            behaviour_type: BehaviourType::Idle,
            learned_at: Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).single().unwrap(),
            statistics: BehaviourProfileStatistics {
                sample_count: 50,
                average_vibration_score: 3.2,
                minimum_vibration_score: 2.1,
                maximum_vibration_score: 4.8,
                vibration_variance: 0.64,
                vibration_standard_deviation: 0.8,
                average_motion_ratio: 0.45,
                minimum_motion_ratio: 0.20,
                maximum_motion_ratio: 0.70,
                average_confidence: 0.92,
                sustained_motion_ratio: 0.75,
                average_gps_speed_kmh: Some(8.5),
            },
        }
    }

    #[test]
    fn exposes_expected_vibration_score() {
        let profile = profile();

        assert_eq!(profile.expected_vibration_score(), 3.2);
    }

    #[test]
    fn exposes_expected_motion_ratio() {
        let profile = profile();

        assert_eq!(profile.expected_motion_ratio(), 0.45);
    }

    #[test]
    fn exposes_standard_deviation() {
        let profile = profile();

        assert_eq!(profile.vibration_standard_deviation(), 0.8);
    }

    #[test]
    fn reports_expected_sustained_motion() {
        let profile = profile();

        assert!(profile.expects_sustained_motion());
    }
}
