#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone)]
pub struct AlertDecision {
    pub should_alert: bool,
    pub severity: AlertSeverity,
    pub reason: String,
}

pub fn evaluate_alert_rule(
    event_type: &str,
    confidence: &str,
    correlation_status: &str,
) -> AlertDecision {
    match (event_type, confidence, correlation_status) {
        ("THEFT", "High", "Consistent") => AlertDecision {
            should_alert: true,
            severity: AlertSeverity::Critical,
            reason: "High-confidence theft with operationally consistent correlation.".to_string(),
        },

        ("REFILL", "Medium", "Conflicting") => AlertDecision {
            should_alert: true,
            severity: AlertSeverity::Warning,
            reason: "Fuel refill pattern conflicts with operational movement state.".to_string(),
        },

        ("LEAK", "High", _) => AlertDecision {
            should_alert: true,
            severity: AlertSeverity::Critical,
            reason: "Persistent high-confidence fuel leak pattern detected.".to_string(),
        },

        _ => AlertDecision {
            should_alert: false,
            severity: AlertSeverity::Info,
            reason: "Event does not currently meet alert escalation thresholds.".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_confidence_consistent_theft_triggers_critical_alert() {
        let decision = evaluate_alert_rule("THEFT", "High", "Consistent");

        assert_eq!(decision.should_alert, true);
        assert_eq!(decision.severity, AlertSeverity::Critical);
    }

    #[test]
    fn conflicting_refill_triggers_warning_alert() {
        let decision = evaluate_alert_rule("REFILL", "Medium", "Conflicting");

        assert_eq!(decision.should_alert, true);
        assert_eq!(decision.severity, AlertSeverity::Warning);
    }

    #[test]
    fn low_confidence_event_does_not_trigger_alert() {
        let decision = evaluate_alert_rule("THEFT", "Low", "Unknown");

        assert_eq!(decision.should_alert, false);
        assert_eq!(decision.severity, AlertSeverity::Info);
    }
}
