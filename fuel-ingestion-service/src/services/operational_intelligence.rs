use crate::services::operational_behaviour::OperationalStateTransition;

/// Represents the first level of operational meaning derived from an
/// objective device-state transition.
///
/// These events describe normal operational behaviour only. They do not yet
/// classify security incidents such as unauthorized movement or fuel theft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationalIntelligenceEvent {
    JourneyStarted,
    VehicleIdling,
    JourneyResumed,
    JourneyEnded,
    VehicleParkedAfterIdling,
    VehicleBecameIdleWhileParked,
    DeviceWentOffline,
    DeviceRecovered,
    NoOperationalEvent,
}

impl OperationalIntelligenceEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            OperationalIntelligenceEvent::JourneyStarted => "JOURNEY_STARTED",
            OperationalIntelligenceEvent::VehicleIdling => "VEHICLE_IDLING",
            OperationalIntelligenceEvent::JourneyResumed => "JOURNEY_RESUMED",
            OperationalIntelligenceEvent::JourneyEnded => "JOURNEY_ENDED",
            OperationalIntelligenceEvent::VehicleParkedAfterIdling => "VEHICLE_PARKED_AFTER_IDLING",
            OperationalIntelligenceEvent::VehicleBecameIdleWhileParked => {
                "VEHICLE_BECAME_IDLE_WHILE_PARKED"
            }
            OperationalIntelligenceEvent::DeviceWentOffline => "DEVICE_WENT_OFFLINE",
            OperationalIntelligenceEvent::DeviceRecovered => "DEVICE_RECOVERED",
            OperationalIntelligenceEvent::NoOperationalEvent => "NO_OPERATIONAL_EVENT",
        }
    }

    pub fn occurred(&self) -> bool {
        !matches!(self, OperationalIntelligenceEvent::NoOperationalEvent)
    }
}

/// Converts an objective state transition into its first-level operational
/// interpretation.
///
/// This function deliberately avoids contextual conclusions. For example,
/// `ParkedToMoving` becomes `JourneyStarted`, but not yet
/// `UnauthorizedMovement`.
pub fn interpret_operational_transition(
    transition: &OperationalStateTransition,
) -> OperationalIntelligenceEvent {
    match transition {
        OperationalStateTransition::ParkedToMoving => OperationalIntelligenceEvent::JourneyStarted,

        OperationalStateTransition::MovingToIdle => OperationalIntelligenceEvent::VehicleIdling,

        OperationalStateTransition::IdleToMoving => OperationalIntelligenceEvent::JourneyResumed,

        OperationalStateTransition::MovingToParked => OperationalIntelligenceEvent::JourneyEnded,

        OperationalStateTransition::IdleToParked => {
            OperationalIntelligenceEvent::VehicleParkedAfterIdling
        }

        OperationalStateTransition::ParkedToIdle => {
            OperationalIntelligenceEvent::VehicleBecameIdleWhileParked
        }

        OperationalStateTransition::DeviceWentOffline => {
            OperationalIntelligenceEvent::DeviceWentOffline
        }

        OperationalStateTransition::DeviceRecovered => {
            OperationalIntelligenceEvent::DeviceRecovered
        }

        OperationalStateTransition::NoTransition => {
            OperationalIntelligenceEvent::NoOperationalEvent
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parked_to_moving_starts_journey() {
        let event = interpret_operational_transition(&OperationalStateTransition::ParkedToMoving);

        assert_eq!(event, OperationalIntelligenceEvent::JourneyStarted);
        assert_eq!(event.as_str(), "JOURNEY_STARTED");
        assert!(event.occurred());
    }

    #[test]
    fn moving_to_idle_creates_idling_event() {
        let event = interpret_operational_transition(&OperationalStateTransition::MovingToIdle);

        assert_eq!(event, OperationalIntelligenceEvent::VehicleIdling);
    }

    #[test]
    fn idle_to_moving_resumes_journey() {
        let event = interpret_operational_transition(&OperationalStateTransition::IdleToMoving);

        assert_eq!(event, OperationalIntelligenceEvent::JourneyResumed);
    }

    #[test]
    fn moving_to_parked_ends_journey() {
        let event = interpret_operational_transition(&OperationalStateTransition::MovingToParked);

        assert_eq!(event, OperationalIntelligenceEvent::JourneyEnded);
    }

    #[test]
    fn offline_transition_creates_device_offline_event() {
        let event =
            interpret_operational_transition(&OperationalStateTransition::DeviceWentOffline);

        assert_eq!(event, OperationalIntelligenceEvent::DeviceWentOffline);
    }

    #[test]
    fn recovery_transition_creates_device_recovered_event() {
        let event = interpret_operational_transition(&OperationalStateTransition::DeviceRecovered);

        assert_eq!(event, OperationalIntelligenceEvent::DeviceRecovered);
    }

    #[test]
    fn no_transition_creates_no_operational_event() {
        let event = interpret_operational_transition(&OperationalStateTransition::NoTransition);

        assert_eq!(event, OperationalIntelligenceEvent::NoOperationalEvent);
        assert!(!event.occurred());
    }
}
