use crate::services::confidence_scoring::ConfidenceLevel;
use crate::services::fuel_event_correlation::CorrelationStatus;

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
    confidence: &ConfidenceLevel,
    correlation_status: &CorrelationStatus,
) -> AlertDecision {
    match event_type {
        "THEFT" => match confidence {
            ConfidenceLevel::High | ConfidenceLevel::Critical => AlertDecision {
                should_alert: true,
                severity: AlertSeverity::Critical,
                reason: match correlation_status {
                    CorrelationStatus::Consistent => {
                        "High-confidence theft with operationally consistent correlation."
                            .to_string()
                    }
                    CorrelationStatus::Suspicious => {
                        "High-confidence theft detected with suspicious operational context."
                            .to_string()
                    }
                    CorrelationStatus::Conflicting => {
                        "High-confidence theft detected with conflicting operational context."
                            .to_string()
                    }
                    CorrelationStatus::Unknown => {
                        "High-confidence theft detected without sufficient operational context."
                            .to_string()
                    }
                },
            },

            ConfidenceLevel::Low | ConfidenceLevel::Medium => AlertDecision {
                should_alert: false,
                severity: AlertSeverity::Info,
                reason: "Theft event does not currently meet alert escalation thresholds."
                    .to_string(),
            },
        },

        "REFILL" => match confidence {
            ConfidenceLevel::Medium | ConfidenceLevel::High | ConfidenceLevel::Critical => {
                if *correlation_status == CorrelationStatus::Conflicting {
                    AlertDecision {
                        should_alert: true,
                        severity: AlertSeverity::Warning,
                        reason: "Fuel refill pattern conflicts with operational movement state."
                            .to_string(),
                    }
                } else {
                    AlertDecision {
                        should_alert: false,
                        severity: AlertSeverity::Info,
                        reason: "Refill event does not currently require alert escalation."
                            .to_string(),
                    }
                }
            }

            ConfidenceLevel::Low => AlertDecision {
                should_alert: false,
                severity: AlertSeverity::Info,
                reason: "Refill event does not currently meet alert escalation thresholds."
                    .to_string(),
            },
        },

        "LEAK" => match confidence {
            ConfidenceLevel::High | ConfidenceLevel::Critical => AlertDecision {
                should_alert: true,
                severity: AlertSeverity::Critical,
                reason: "Persistent high-confidence fuel leak pattern detected.".to_string(),
            },

            ConfidenceLevel::Low | ConfidenceLevel::Medium => AlertDecision {
                should_alert: false,
                severity: AlertSeverity::Info,
                reason: "Leak event does not currently meet alert escalation thresholds."
                    .to_string(),
            },
        },

        _ => AlertDecision {
            should_alert: false,
            severity: AlertSeverity::Info,
            reason: "Unknown fuel event type.".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_confidence_consistent_theft_triggers_critical_alert() {
        let decision = evaluate_alert_rule(
            "THEFT",
            &ConfidenceLevel::High,
            &CorrelationStatus::Consistent,
        );

        assert_eq!(decision.should_alert, true);
        assert_eq!(decision.severity, AlertSeverity::Critical);
    }

    #[test]
    fn critical_confidence_theft_also_triggers_alert() {
        let decision = evaluate_alert_rule(
            "THEFT",
            &ConfidenceLevel::Critical,
            &CorrelationStatus::Suspicious,
        );

        assert_eq!(decision.should_alert, true);
        assert_eq!(decision.severity, AlertSeverity::Critical);
    }

    #[test]
    fn conflicting_refill_triggers_warning_alert() {
        let decision = evaluate_alert_rule(
            "REFILL",
            &ConfidenceLevel::Medium,
            &CorrelationStatus::Conflicting,
        );

        assert_eq!(decision.should_alert, true);
        assert_eq!(decision.severity, AlertSeverity::Warning);
    }

    #[test]
    fn high_confidence_conflicting_refill_still_triggers_warning() {
        let decision = evaluate_alert_rule(
            "REFILL",
            &ConfidenceLevel::High,
            &CorrelationStatus::Conflicting,
        );

        assert_eq!(decision.should_alert, true);
        assert_eq!(decision.severity, AlertSeverity::Warning);
    }

    #[test]
    fn high_confidence_leak_triggers_critical_alert() {
        let decision =
            evaluate_alert_rule("LEAK", &ConfidenceLevel::High, &CorrelationStatus::Unknown);

        assert_eq!(decision.should_alert, true);
        assert_eq!(decision.severity, AlertSeverity::Critical);
    }

    #[test]
    fn critical_confidence_leak_triggers_critical_alert() {
        let decision = evaluate_alert_rule(
            "LEAK",
            &ConfidenceLevel::Critical,
            &CorrelationStatus::Unknown,
        );

        assert_eq!(decision.should_alert, true);
        assert_eq!(decision.severity, AlertSeverity::Critical);
    }

    #[test]
    fn low_confidence_event_does_not_trigger_alert() {
        let decision =
            evaluate_alert_rule("THEFT", &ConfidenceLevel::Low, &CorrelationStatus::Unknown);

        assert_eq!(decision.should_alert, false);
        assert_eq!(decision.severity, AlertSeverity::Info);
    }
}
