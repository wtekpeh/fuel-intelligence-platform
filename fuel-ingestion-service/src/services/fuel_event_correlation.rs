#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorrelationStatus {
    Consistent,
    Suspicious,
    Conflicting,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct FuelEventCorrelationResult {
    pub status: CorrelationStatus,
    pub reason: String,
}

pub fn correlate_fuel_event(
    event_type: &str,
    device_state: &str,
    motion_detected: bool,
) -> FuelEventCorrelationResult {
    match (event_type, device_state, motion_detected) {
        ("THEFT", "PARKED", false) => FuelEventCorrelationResult {
            status: CorrelationStatus::Consistent,
            reason: "Fuel theft pattern aligns with parked stationary vehicle.".to_string(),
        },

        ("THEFT", "IDLE", false) => FuelEventCorrelationResult {
            status: CorrelationStatus::Consistent,
            reason: "Fuel theft pattern aligns with idle stationary vehicle.".to_string(),
        },

        ("THEFT", "MOVING", true) => FuelEventCorrelationResult {
            status: CorrelationStatus::Suspicious,
            reason: "Fuel drop occurred while vehicle was moving.".to_string(),
        },

        ("REFILL", "IDLE", false) | ("REFILL", "PARKED", false) => FuelEventCorrelationResult {
            status: CorrelationStatus::Consistent,
            reason: "Fuel increase aligns with stationary refill pattern.".to_string(),
        },

        ("REFILL", "MOVING", true) => FuelEventCorrelationResult {
            status: CorrelationStatus::Conflicting,
            reason: "Fuel increase occurred while vehicle was moving.".to_string(),
        },

        _ => FuelEventCorrelationResult {
            status: CorrelationStatus::Unknown,
            reason: "Insufficient operational context for correlation.".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theft_while_parked_is_consistent() {
        let result = correlate_fuel_event("THEFT", "PARKED", false);

        assert_eq!(result.status, CorrelationStatus::Consistent);
    }

    #[test]
    fn theft_while_idle_is_consistent() {
        let result = correlate_fuel_event("THEFT", "IDLE", false);

        assert_eq!(result.status, CorrelationStatus::Consistent);
    }

    #[test]
    fn refill_while_moving_is_conflicting() {
        let result = correlate_fuel_event("REFILL", "MOVING", true);

        assert_eq!(result.status, CorrelationStatus::Conflicting);
    }

    #[test]
    fn refill_while_idle_is_consistent() {
        let result = correlate_fuel_event("REFILL", "IDLE", false);

        assert_eq!(result.status, CorrelationStatus::Consistent);
    }
}
