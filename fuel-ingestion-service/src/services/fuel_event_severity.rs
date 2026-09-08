#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FuelEventSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl FuelEventSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            FuelEventSeverity::Low => "low",
            FuelEventSeverity::Medium => "medium",
            FuelEventSeverity::High => "high",
            FuelEventSeverity::Critical => "critical",
        }
    }
}

pub fn calculate_fuel_event_severity(
    fuel_difference_litres: f64,
    tank_capacity_litres: f64,
) -> FuelEventSeverity {
    if tank_capacity_litres <= 0.0 {
        return FuelEventSeverity::Low;
    }

    let fraction_of_capacity = fuel_difference_litres.abs() / tank_capacity_litres;

    if fraction_of_capacity >= 0.50 {
        FuelEventSeverity::Critical
    } else if fraction_of_capacity >= 0.25 {
        FuelEventSeverity::High
    } else if fraction_of_capacity >= 0.10 {
        FuelEventSeverity::Medium
    } else {
        FuelEventSeverity::Low
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_event_is_low_severity() {
        let severity = calculate_fuel_event_severity(10.0, 200.0);

        assert_eq!(severity, FuelEventSeverity::Low);
    }

    #[test]
    fn ten_percent_event_is_medium_severity() {
        let severity = calculate_fuel_event_severity(20.0, 200.0);

        assert_eq!(severity, FuelEventSeverity::Medium);
    }

    #[test]
    fn twenty_five_percent_event_is_high_severity() {
        let severity = calculate_fuel_event_severity(50.0, 200.0);

        assert_eq!(severity, FuelEventSeverity::High);
    }

    #[test]
    fn fifty_percent_event_is_critical_severity() {
        let severity = calculate_fuel_event_severity(100.0, 200.0);

        assert_eq!(severity, FuelEventSeverity::Critical);
    }

    #[test]
    fn severity_scales_with_small_tank_capacity() {
        let severity = calculate_fuel_event_severity(0.30, 1.50);

        assert_eq!(severity, FuelEventSeverity::Medium);
    }

    #[test]
    fn invalid_tank_capacity_returns_low_severity() {
        let severity = calculate_fuel_event_severity(10.0, 0.0);

        assert_eq!(severity, FuelEventSeverity::Low);
    }
}
