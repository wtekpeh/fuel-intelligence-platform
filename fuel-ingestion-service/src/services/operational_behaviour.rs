use crate::services::device_state::DeviceOperationalState;

/// Represents an objective transition between two device operational states.
///
/// This layer does not yet assign higher-level business meaning such as
/// "Journey Started" or "Unexpected Movement". It only records the actual
/// change that occurred between consecutive operational states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationalStateTransition {
    ParkedToMoving,
    MovingToIdle,
    IdleToMoving,
    MovingToParked,
    IdleToParked,
    ParkedToIdle,

    DeviceWentOffline,
    DeviceRecovered,

    NoTransition,
}

impl OperationalStateTransition {
    pub fn as_str(&self) -> &'static str {
        match self {
            OperationalStateTransition::ParkedToMoving => "PARKED_TO_MOVING",
            OperationalStateTransition::MovingToIdle => "MOVING_TO_IDLE",
            OperationalStateTransition::IdleToMoving => "IDLE_TO_MOVING",
            OperationalStateTransition::MovingToParked => "MOVING_TO_PARKED",
            OperationalStateTransition::IdleToParked => "IDLE_TO_PARKED",
            OperationalStateTransition::ParkedToIdle => "PARKED_TO_IDLE",
            OperationalStateTransition::DeviceWentOffline => "DEVICE_WENT_OFFLINE",
            OperationalStateTransition::DeviceRecovered => "DEVICE_RECOVERED",
            OperationalStateTransition::NoTransition => "NO_TRANSITION",
        }
    }

    pub fn occurred(&self) -> bool {
        !matches!(self, OperationalStateTransition::NoTransition)
    }
}

/// Compares the previous and current device states and determines whether an
/// objective operational transition occurred.
///
/// The first observed state does not produce a transition because there is no
/// previous state available for comparison.
pub fn determine_operational_transition(
    previous: Option<&DeviceOperationalState>,
    current: &DeviceOperationalState,
) -> OperationalStateTransition {
    let Some(previous) = previous else {
        return OperationalStateTransition::NoTransition;
    };

    match (previous, current) {
        (DeviceOperationalState::Parked, DeviceOperationalState::Moving) => {
            OperationalStateTransition::ParkedToMoving
        }

        (DeviceOperationalState::Moving, DeviceOperationalState::Idle) => {
            OperationalStateTransition::MovingToIdle
        }

        (DeviceOperationalState::Idle, DeviceOperationalState::Moving) => {
            OperationalStateTransition::IdleToMoving
        }

        (DeviceOperationalState::Moving, DeviceOperationalState::Parked) => {
            OperationalStateTransition::MovingToParked
        }

        (DeviceOperationalState::Idle, DeviceOperationalState::Parked) => {
            OperationalStateTransition::IdleToParked
        }

        (DeviceOperationalState::Parked, DeviceOperationalState::Idle) => {
            OperationalStateTransition::ParkedToIdle
        }

        (_, DeviceOperationalState::Offline)
            if !matches!(previous, DeviceOperationalState::Offline) =>
        {
            OperationalStateTransition::DeviceWentOffline
        }

        (DeviceOperationalState::Offline, current)
            if !matches!(current, DeviceOperationalState::Offline) =>
        {
            OperationalStateTransition::DeviceRecovered
        }

        _ => OperationalStateTransition::NoTransition,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_observed_state_does_not_create_transition() {
        let transition = determine_operational_transition(None, &DeviceOperationalState::Parked);

        assert_eq!(transition, OperationalStateTransition::NoTransition);
        assert!(!transition.occurred());
    }

    #[test]
    fn parked_to_moving_is_detected() {
        let transition = determine_operational_transition(
            Some(&DeviceOperationalState::Parked),
            &DeviceOperationalState::Moving,
        );

        assert_eq!(transition, OperationalStateTransition::ParkedToMoving);
        assert!(transition.occurred());
        assert_eq!(transition.as_str(), "PARKED_TO_MOVING");
    }

    #[test]
    fn moving_to_idle_is_detected() {
        let transition = determine_operational_transition(
            Some(&DeviceOperationalState::Moving),
            &DeviceOperationalState::Idle,
        );

        assert_eq!(transition, OperationalStateTransition::MovingToIdle);
    }

    #[test]
    fn idle_to_moving_is_detected() {
        let transition = determine_operational_transition(
            Some(&DeviceOperationalState::Idle),
            &DeviceOperationalState::Moving,
        );

        assert_eq!(transition, OperationalStateTransition::IdleToMoving);
    }

    #[test]
    fn moving_to_parked_is_detected() {
        let transition = determine_operational_transition(
            Some(&DeviceOperationalState::Moving),
            &DeviceOperationalState::Parked,
        );

        assert_eq!(transition, OperationalStateTransition::MovingToParked);
    }

    #[test]
    fn online_state_to_offline_is_detected() {
        let transition = determine_operational_transition(
            Some(&DeviceOperationalState::Moving),
            &DeviceOperationalState::Offline,
        );

        assert_eq!(transition, OperationalStateTransition::DeviceWentOffline);
    }

    #[test]
    fn offline_to_active_state_is_recovery() {
        let transition = determine_operational_transition(
            Some(&DeviceOperationalState::Offline),
            &DeviceOperationalState::Parked,
        );

        assert_eq!(transition, OperationalStateTransition::DeviceRecovered);
    }

    #[test]
    fn unchanged_state_does_not_create_transition() {
        let transition = determine_operational_transition(
            Some(&DeviceOperationalState::Moving),
            &DeviceOperationalState::Moving,
        );

        assert_eq!(transition, OperationalStateTransition::NoTransition);
    }
}
