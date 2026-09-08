#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfidenceLevel {
    Low,
    Medium,
    High,
    Critical,
}

pub fn score_fuel_event_confidence(
    device_state: &str,
    current_reading_is_outlier: bool,
    jump_is_suspicious: bool,
    is_delayed_detection: bool,
) -> ConfidenceLevel {
    // Reaching this function means the event-specific detector has already
    // satisfied its minimum detection rule.
    //
    // Confidence therefore starts from the same evidence base for every fuel
    // event type. THEFT, REFILL, and LEAK do not receive different confidence
    // merely because of their event label.
    let mut score = 2;

    match device_state {
        "PARKED" | "IDLE" => score += 2,
        "MOVING" => score += 1,
        "OFFLINE" => score += 1,
        _ => score += 0,
    }

    if current_reading_is_outlier {
        score += 2;
    }

    if jump_is_suspicious {
        score -= 1;
    }

    if is_delayed_detection {
        score -= 1;
    }

    let score = score.max(0);

    match score {
        0..=2 => ConfidenceLevel::Low,
        3..=5 => ConfidenceLevel::Medium,
        6..=7 => ConfidenceLevel::High,
        _ => ConfidenceLevel::Critical,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stationary_event_with_current_outlier_is_high_confidence() {
        let confidence = score_fuel_event_confidence("PARKED", true, false, false);

        assert_eq!(confidence, ConfidenceLevel::High);
    }

    #[test]
    fn moving_event_with_current_outlier_is_medium_confidence() {
        let confidence = score_fuel_event_confidence("MOVING", true, false, false);

        assert_eq!(confidence, ConfidenceLevel::Medium);
    }

    #[test]
    fn suspicious_jump_reduces_confidence() {
        let confidence = score_fuel_event_confidence("PARKED", true, true, false);

        assert_eq!(confidence, ConfidenceLevel::Medium);
    }

    #[test]
    fn delayed_detection_reduces_confidence() {
        let confidence = score_fuel_event_confidence("PARKED", true, false, true);

        assert_eq!(confidence, ConfidenceLevel::Medium);
    }

    #[test]
    fn suspicious_and_delayed_event_is_medium_confidence() {
        let confidence = score_fuel_event_confidence("PARKED", true, true, true);

        assert_eq!(confidence, ConfidenceLevel::Medium);
    }

    #[test]
    fn stationary_event_without_current_outlier_stays_medium_confidence() {
        let confidence = score_fuel_event_confidence("PARKED", false, false, false);

        assert_eq!(confidence, ConfidenceLevel::Medium);
    }

    #[test]
    fn unknown_state_without_current_outlier_is_low_confidence() {
        let confidence = score_fuel_event_confidence("UNKNOWN", false, false, false);

        assert_eq!(confidence, ConfidenceLevel::Low);
    }

    #[test]
    fn penalties_never_create_negative_confidence_score() {
        let confidence = score_fuel_event_confidence("UNKNOWN", false, true, true);

        assert_eq!(confidence, ConfidenceLevel::Low);
    }
}
