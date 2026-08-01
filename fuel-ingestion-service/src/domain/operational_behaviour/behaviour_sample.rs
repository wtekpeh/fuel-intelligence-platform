use chrono::{DateTime, Utc};

use crate::domain::telemetry::motion_buffer::MotionEvidence;

/// One operational behaviour observation collected during
/// a learning session.
///
/// This wraps the existing MotionEvidence produced by the
/// telemetry pipeline rather than duplicating its fields.
#[derive(Debug, Clone)]
pub struct BehaviourSample {
    /// Time at which this behaviour sample was recorded.
    pub recorded_at: DateTime<Utc>,

    /// Canonical motion evidence derived from the rolling
    /// telemetry window.
    pub motion_evidence: MotionEvidence,

    /// Optional GPS speed at the time of collection.
    ///
    /// This allows future validation of learning sessions
    /// (for example detecting that a PARKED session was
    /// contaminated by vehicle movement).
    pub gps_speed_kmh: Option<f64>,
}

impl BehaviourSample {
    /// Returns true when GPS indicates the asset was stationary.
    pub fn is_stationary(&self) -> bool {
        self.gps_speed_kmh.unwrap_or(0.0) < 1.0
    }

    /// Returns true when GPS indicates meaningful movement.
    pub fn is_moving(&self) -> bool {
        self.gps_speed_kmh.unwrap_or(0.0) >= 5.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn evidence() -> MotionEvidence {
        MotionEvidence {
            average_vibration_score: 0.0,
            average_gravity_deviation_g: 0.0,
            average_rotation_magnitude_dps: 0.0,
            motion_ratio: 0.0,
            average_confidence: 0.0,
            sustained_motion: false,
            sample_count: 5,
        }
    }

    #[test]
    fn stationary_speed_is_detected() {
        let sample = BehaviourSample {
            recorded_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single().unwrap(),
            motion_evidence: evidence(),
            gps_speed_kmh: Some(0.3),
        };

        assert!(sample.is_stationary());
        assert!(!sample.is_moving());
    }

    #[test]
    fn moving_speed_is_detected() {
        let sample = BehaviourSample {
            recorded_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single().unwrap(),
            motion_evidence: evidence(),
            gps_speed_kmh: Some(42.0),
        };

        assert!(sample.is_moving());
        assert!(!sample.is_stationary());
    }
}
