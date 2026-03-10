//! Access rule definitions.

use chrono::{DateTime, Datelike, Timelike, Utc, Weekday};
use serde::{Deserialize, Serialize};

/// Access decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessDecision {
    /// Access granted.
    Allow,
    /// Access denied.
    Deny,
    /// Access granted but logged for audit.
    AllowWithLog,
}

/// Access rule for policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRule {
    /// Rule ID.
    pub id: String,

    /// Rule name.
    pub name: String,

    /// Rule priority (lower = higher priority).
    pub priority: u32,

    /// Conditions that must all be met.
    pub conditions: Vec<RuleCondition>,

    /// Action if conditions match.
    pub action: AccessDecision,

    /// Whether this rule is enabled.
    pub enabled: bool,
}

/// Rule condition types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleCondition {
    /// Time window condition.
    TimeWindow {
        /// Start time (HH:MM format).
        start: String,
        /// End time (HH:MM format).
        end: String,
        /// Days of week (empty = all days).
        days: Vec<Weekday>,
    },

    /// Credential type condition.
    CredentialType {
        /// Allowed credential types.
        allowed: Vec<String>,
    },

    /// Floor access condition.
    FloorAccess {
        /// Allowed floors.
        floors: Vec<String>,
    },

    /// Holder DID condition.
    HolderDid {
        /// Specific DIDs to match.
        dids: Vec<String>,
    },

    /// Issuer DID condition.
    IssuerDid {
        /// Allowed issuer DIDs.
        issuers: Vec<String>,
    },

    /// Property condition.
    Property {
        /// Property ID to match.
        property_id: String,
    },

    /// Zone condition.
    Zone {
        /// Zone IDs to match.
        zones: Vec<String>,
    },
}

impl RuleCondition {
    /// Evaluate the condition.
    pub fn evaluate(&self, context: &EvaluationContext) -> bool {
        match self {
            Self::TimeWindow { start, end, days } => {
                Self::check_time_window(start, end, days, context.current_time)
            }
            Self::CredentialType { allowed } => {
                context.credential_type.as_ref().is_some_and(|ct| allowed.contains(ct))
            }
            Self::FloorAccess { floors } => {
                context.requested_floor.as_ref().is_some_and(|f| floors.contains(f))
            }
            Self::HolderDid { dids } => {
                context.holder_did.as_ref().is_some_and(|did| dids.contains(did))
            }
            Self::IssuerDid { issuers } => {
                context.issuer_did.as_ref().is_some_and(|did| issuers.contains(did))
            }
            Self::Property { property_id } => context.property_id == *property_id,
            Self::Zone { zones } => {
                context.zone_id.as_ref().is_some_and(|z| zones.contains(z))
            }
        }
    }

    fn check_time_window(
        start: &str,
        end: &str,
        days: &[Weekday],
        current_time: DateTime<Utc>,
    ) -> bool {
        // Check day of week if specified
        if !days.is_empty() {
            let current_day = current_time.weekday();
            if !days.contains(&current_day) {
                return false;
            }
        }

        // Parse times
        let current_minutes = current_time.hour() * 60 + current_time.minute();

        let start_minutes = Self::parse_time_to_minutes(start).unwrap_or(0);
        let end_minutes = Self::parse_time_to_minutes(end).unwrap_or(24 * 60);

        // Handle overnight windows (e.g., 22:00 - 06:00)
        if start_minutes > end_minutes {
            current_minutes >= start_minutes || current_minutes < end_minutes
        } else {
            current_minutes >= start_minutes && current_minutes < end_minutes
        }
    }

    fn parse_time_to_minutes(time: &str) -> Option<u32> {
        let parts: Vec<&str> = time.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        let hours: u32 = parts[0].parse().ok()?;
        let minutes: u32 = parts[1].parse().ok()?;
        Some(hours * 60 + minutes)
    }
}

/// Context for rule evaluation.
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// Current time.
    pub current_time: DateTime<Utc>,

    /// Property ID.
    pub property_id: String,

    /// Holder DID.
    pub holder_did: Option<String>,

    /// Issuer DID.
    pub issuer_did: Option<String>,

    /// Credential type.
    pub credential_type: Option<String>,

    /// Requested floor.
    pub requested_floor: Option<String>,

    /// Zone ID.
    pub zone_id: Option<String>,
}

impl AccessRule {
    /// Evaluate this rule against a context.
    pub fn evaluate(&self, context: &EvaluationContext) -> Option<AccessDecision> {
        if !self.enabled {
            return None;
        }

        // All conditions must match
        let all_match = self.conditions.iter().all(|c| c.evaluate(context));

        if all_match {
            Some(self.action)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn create_test_context() -> EvaluationContext {
        EvaluationContext {
            current_time: Utc.with_ymd_and_hms(2026, 3, 10, 14, 30, 0).unwrap(),
            property_id: "PRY_01HXK".to_string(),
            holder_did: Some("did:key:z6Mk...".to_string()),
            issuer_did: Some("did:web:sahi.my".to_string()),
            credential_type: Some("ResidentBadge".to_string()),
            requested_floor: Some("12".to_string()),
            zone_id: Some("LOBBY".to_string()),
        }
    }

    #[test]
    fn test_time_window_condition_within() {
        let condition = RuleCondition::TimeWindow {
            start: "09:00".to_string(),
            end: "18:00".to_string(),
            days: vec![],
        };

        let context = create_test_context();
        assert!(condition.evaluate(&context));
    }

    #[test]
    fn test_time_window_condition_outside() {
        let condition = RuleCondition::TimeWindow {
            start: "09:00".to_string(),
            end: "12:00".to_string(),
            days: vec![],
        };

        let context = create_test_context();
        assert!(!condition.evaluate(&context));
    }

    #[test]
    fn test_credential_type_condition() {
        let condition = RuleCondition::CredentialType {
            allowed: vec!["ResidentBadge".to_string(), "VisitorPass".to_string()],
        };

        let context = create_test_context();
        assert!(condition.evaluate(&context));
    }

    #[test]
    fn test_rule_evaluation() {
        let rule = AccessRule {
            id: "RULE_01".to_string(),
            name: "Business hours access".to_string(),
            priority: 1,
            conditions: vec![
                RuleCondition::TimeWindow {
                    start: "09:00".to_string(),
                    end: "18:00".to_string(),
                    days: vec![],
                },
                RuleCondition::CredentialType {
                    allowed: vec!["ResidentBadge".to_string()],
                },
            ],
            action: AccessDecision::Allow,
            enabled: true,
        };

        let context = create_test_context();
        assert_eq!(rule.evaluate(&context), Some(AccessDecision::Allow));
    }

    #[test]
    fn test_disabled_rule() {
        let rule = AccessRule {
            id: "RULE_01".to_string(),
            name: "Disabled rule".to_string(),
            priority: 1,
            conditions: vec![],
            action: AccessDecision::Allow,
            enabled: false,
        };

        let context = create_test_context();
        assert_eq!(rule.evaluate(&context), None);
    }
}
