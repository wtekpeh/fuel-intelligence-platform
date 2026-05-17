#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfidenceLevel {
    Low,
    Medium,
    High,
    Critical,
}

pub fn score_fuel_event_confidence(
    event_type: &str,
    device_state: &str,
    outlier_count: usize,
    candidate_count: usize,
    jump_is_suspicious: bool,
    is_delayed_detection: bool,
) -> ConfidenceLevel {
    let mut score = 0;

    match event_type {
        "THEFT" => score += 3,
        "LEAK" => score += 2,
        "REFILL" => score += 1,
        _ => score += 0,
    }

    match device_state {
        "PARKED" | "IDLE" => score += 2,
        "MOVING" => score += 1,
        "OFFLINE" => score += 1,
        _ => score += 0,
    }

    if candidate_count > 0 && outlier_count == candidate_count {
        score += 2;
    } else if outlier_count > 0 {
        score += 1;
    }

    if jump_is_suspicious {
        score += 1;
    }

    if is_delayed_detection {
        score += 1;
    }

    match score {
        0..=2 => ConfidenceLevel::Low,
        3..=5 => ConfidenceLevel::Medium,
        6..=7 => ConfidenceLevel::High,
        _ => ConfidenceLevel::Critical,
    }
}
