use chrono::{DateTime, Utc};
use uuid::Uuid;

/// The operational behaviour being demonstrated during a learning session.
///
/// These values intentionally mirror the database CHECK constraint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviourType {
    Parked,
    Idle,
    Moving,
}

impl BehaviourType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Parked => "PARKED",
            Self::Idle => "IDLE",
            Self::Moving => "MOVING",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "PARKED" => Some(Self::Parked),
            "IDLE" => Some(Self::Idle),
            "MOVING" => Some(Self::Moving),
            _ => None,
        }
    }
}

/// Current lifecycle state of an operational behaviour learning session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LearningStatus {
    NotStarted,
    Collecting,
    Completed,
    Failed,
}

impl LearningStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotStarted => "NOT_STARTED",
            Self::Collecting => "COLLECTING",
            Self::Completed => "COMPLETED",
            Self::Failed => "FAILED",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "NOT_STARTED" => Some(Self::NotStarted),
            "COLLECTING" => Some(Self::Collecting),
            "COMPLETED" => Some(Self::Completed),
            "FAILED" => Some(Self::Failed),
            _ => None,
        }
    }
}

/// Canonical domain representation of one installer-guided
/// operational behaviour learning exercise.
///
/// Persistence-specific SQL representations should be converted
/// into this model by the repository layer.
#[derive(Debug, Clone)]
pub struct OperationalBehaviourLearningSession {
    pub id: Uuid,
    pub device_id: Uuid,
    pub sensor_id: Uuid,
    pub behaviour_type: BehaviourType,
    pub status: LearningStatus,
    pub requested_sample_count: i32,
    pub collected_sample_count: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OperationalBehaviourLearningSession {
    /// Returns the number of samples still required.
    pub fn remaining_sample_count(&self) -> i32 {
        (self.requested_sample_count - self.collected_sample_count).max(0)
    }

    /// Returns collection progress in the inclusive range 0.0 to 1.0.
    pub fn progress_ratio(&self) -> f64 {
        if self.requested_sample_count <= 0 {
            return 0.0;
        }

        (self.collected_sample_count as f64 / self.requested_sample_count as f64).clamp(0.0, 1.0)
    }

    pub fn is_collecting(&self) -> bool {
        self.status == LearningStatus::Collecting
    }

    pub fn is_complete(&self) -> bool {
        self.status == LearningStatus::Completed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn behaviour_type_maps_to_database_values() {
        assert_eq!(BehaviourType::Parked.as_str(), "PARKED");
        assert_eq!(BehaviourType::Idle.as_str(), "IDLE");
        assert_eq!(BehaviourType::Moving.as_str(), "MOVING");
    }

    #[test]
    fn behaviour_type_parses_database_values() {
        assert_eq!(
            BehaviourType::from_str("PARKED"),
            Some(BehaviourType::Parked)
        );
        assert_eq!(BehaviourType::from_str("IDLE"), Some(BehaviourType::Idle));
        assert_eq!(
            BehaviourType::from_str("MOVING"),
            Some(BehaviourType::Moving)
        );
        assert_eq!(BehaviourType::from_str("UNKNOWN"), None);
    }

    #[test]
    fn learning_status_maps_to_database_values() {
        assert_eq!(LearningStatus::NotStarted.as_str(), "NOT_STARTED");
        assert_eq!(LearningStatus::Collecting.as_str(), "COLLECTING");
        assert_eq!(LearningStatus::Completed.as_str(), "COMPLETED");
        assert_eq!(LearningStatus::Failed.as_str(), "FAILED");
    }
}
