//! Access rules types and data structures.

use chrono::{DateTime, NaiveTime, Utc, Weekday};
use serde::{Deserialize, Serialize};

use crate::credential_types::CredentialType;

/// Action to take when a rule matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessAction {
    /// Allow access.
    Allow,
    /// Deny access.
    Deny,
    /// Allow access but log the event for review.
    AllowWithLog,
}

impl AccessAction {
    /// Whether this action grants access.
    #[must_use]
    pub fn grants_access(self) -> bool {
        matches!(self, Self::Allow | Self::AllowWithLog)
    }

    /// Whether this action requires logging.
    #[must_use]
    pub fn requires_log(self) -> bool {
        matches!(self, Self::AllowWithLog | Self::Deny)
    }
}

/// Access decision result.
#[derive(Debug, Clone)]
pub struct AccessDecision {
    /// The action to take.
    pub action: AccessAction,

    /// The rule that matched (if any).
    pub matched_rule: Option<String>,

    /// Reason for the decision.
    pub reason: String,

    /// Whether the decision should be logged.
    pub log_required: bool,

    /// Timestamp of the decision.
    pub timestamp: DateTime<Utc>,
}

impl AccessDecision {
    /// Create an allow decision.
    #[must_use]
    pub fn allow(rule_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            action: AccessAction::Allow,
            matched_rule: Some(rule_id.into()),
            reason: reason.into(),
            log_required: false,
            timestamp: Utc::now(),
        }
    }

    /// Create a deny decision.
    #[must_use]
    pub fn deny(rule_id: Option<String>, reason: impl Into<String>) -> Self {
        Self {
            action: AccessAction::Deny,
            matched_rule: rule_id,
            reason: reason.into(),
            log_required: true,
            timestamp: Utc::now(),
        }
    }

    /// Create an allow-with-log decision.
    #[must_use]
    pub fn allow_with_log(rule_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            action: AccessAction::AllowWithLog,
            matched_rule: Some(rule_id.into()),
            reason: reason.into(),
            log_required: true,
            timestamp: Utc::now(),
        }
    }

    /// Create a default deny (no matching rules).
    #[must_use]
    pub fn default_deny() -> Self {
        Self {
            action: AccessAction::Deny,
            matched_rule: None,
            reason: "No matching access rule".to_string(),
            log_required: true,
            timestamp: Utc::now(),
        }
    }

    /// Whether access is granted.
    #[must_use]
    pub fn is_granted(&self) -> bool {
        self.action.grants_access()
    }
}

/// Context for evaluating access rules.
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// Tenant ID.
    pub tenant_id: String,

    /// Property ID.
    pub property_id: String,

    /// Credential type being presented.
    pub credential_type: CredentialType,

    /// Role from credential.
    pub role: String,

    /// Clearance level (0-3).
    pub clearance: u8,

    /// Floors the credential grants access to.
    pub granted_floors: Vec<String>,

    /// Floor being accessed.
    pub target_floor: String,

    /// Zone being accessed (lobby, parking, gym, etc.).
    pub target_zone: Option<String>,

    /// Current time for time-based rules.
    pub current_time: DateTime<Utc>,

    /// Additional context metadata.
    pub metadata: std::collections::HashMap<String, String>,
}

impl EvaluationContext {
    /// Create a new evaluation context.
    pub fn new(
        tenant_id: impl Into<String>,
        property_id: impl Into<String>,
        credential_type: CredentialType,
        role: impl Into<String>,
        target_floor: impl Into<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            property_id: property_id.into(),
            credential_type,
            role: role.into(),
            clearance: 0,
            granted_floors: Vec::new(),
            target_floor: target_floor.into(),
            target_zone: None,
            current_time: Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set clearance level.
    #[must_use]
    pub fn with_clearance(mut self, level: u8) -> Self {
        self.clearance = level;
        self
    }

    /// Set granted floors.
    #[must_use]
    pub fn with_floors(mut self, floors: Vec<String>) -> Self {
        self.granted_floors = floors;
        self
    }

    /// Set target zone.
    #[must_use]
    pub fn with_zone(mut self, zone: impl Into<String>) -> Self {
        self.target_zone = Some(zone.into());
        self
    }

    /// Set evaluation time (for testing).
    #[must_use]
    pub fn at_time(mut self, time: DateTime<Utc>) -> Self {
        self.current_time = time;
        self
    }
}

/// Access rule definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRule {
    /// Unique rule ID.
    pub rule_id: String,

    /// Tenant ID.
    pub tenant_id: String,

    /// Property ID (or "*" for all properties).
    pub property_id: String,

    /// Human-readable rule name.
    pub name: String,

    /// Conditions that must all match.
    pub conditions: Vec<Condition>,

    /// Action to take when rule matches.
    pub action: AccessAction,

    /// Priority (higher wins on conflict).
    pub priority: u32,

    /// Whether the rule is enabled.
    pub enabled: bool,
}

impl AccessRule {
    /// Create a new access rule.
    pub fn new(
        rule_id: impl Into<String>,
        tenant_id: impl Into<String>,
        property_id: impl Into<String>,
        name: impl Into<String>,
        action: AccessAction,
        priority: u32,
    ) -> Self {
        Self {
            rule_id: rule_id.into(),
            tenant_id: tenant_id.into(),
            property_id: property_id.into(),
            name: name.into(),
            conditions: Vec::new(),
            action,
            priority,
            enabled: true,
        }
    }

    /// Add a condition to the rule.
    #[must_use]
    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Check if this rule applies to the given context.
    #[must_use]
    pub fn applies_to(&self, ctx: &EvaluationContext) -> bool {
        if !self.enabled {
            return false;
        }

        // Check tenant
        if self.tenant_id != ctx.tenant_id {
            return false;
        }

        // Check property (wildcard matches all)
        if self.property_id != "*" && self.property_id != ctx.property_id {
            return false;
        }

        true
    }
}

/// Rule condition types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Condition {
    /// Time window condition.
    TimeWindow {
        /// Start time (HH:MM).
        start: String,
        /// End time (HH:MM).
        end: String,
        /// Days of week (0=Sunday, 6=Saturday).
        days: Vec<u8>,
    },
    /// Floor access condition.
    FloorAccess {
        /// Allowed floors.
        floors: Vec<String>,
    },
    /// Role condition.
    Role {
        /// Allowed roles.
        roles: Vec<String>,
    },
    /// Clearance level condition.
    Clearance {
        /// Minimum required clearance.
        min_level: u8,
    },
    /// Credential type condition.
    CredentialType {
        /// Allowed credential types.
        types: Vec<CredentialType>,
    },
    /// Zone condition.
    Zone {
        /// Allowed zones.
        zones: Vec<String>,
    },
}

/// Result of evaluating a single condition.
#[derive(Debug, Clone)]
pub struct ConditionMatch {
    /// The condition that was evaluated.
    pub condition_type: String,

    /// Whether the condition matched.
    pub matched: bool,

    /// Reason for match/no-match.
    pub reason: String,
}

impl ConditionMatch {
    /// Create a matched result.
    #[must_use]
    pub fn matched(condition_type: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            condition_type: condition_type.into(),
            matched: true,
            reason: reason.into(),
        }
    }

    /// Create an unmatched result.
    #[must_use]
    pub fn unmatched(condition_type: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            condition_type: condition_type.into(),
            matched: false,
            reason: reason.into(),
        }
    }
}

/// A set of access rules for a tenant/property.
#[derive(Debug, Clone, Default)]
pub struct RuleSet {
    /// Rules sorted by priority (highest first).
    rules: Vec<AccessRule>,
}

impl RuleSet {
    /// Create a new empty rule set.
    #[must_use]
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a rule to the set.
    pub fn add(&mut self, rule: AccessRule) {
        self.rules.push(rule);
        // Sort by priority descending
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Remove a rule by ID.
    pub fn remove(&mut self, rule_id: &str) {
        self.rules.retain(|r| r.rule_id != rule_id);
    }

    /// Get all rules.
    #[must_use]
    pub fn rules(&self) -> &[AccessRule] {
        &self.rules
    }

    /// Get rules applicable to a context.
    #[must_use]
    pub fn applicable_rules(&self, ctx: &EvaluationContext) -> Vec<&AccessRule> {
        self.rules.iter().filter(|r| r.applies_to(ctx)).collect()
    }
}

/// Parse time from HH:MM string.
pub fn parse_time(s: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(s, "%H:%M").ok()
}

/// Get weekday number (0=Sunday, 6=Saturday).
#[must_use]
pub fn weekday_number(weekday: Weekday) -> u8 {
    match weekday {
        Weekday::Sun => 0,
        Weekday::Mon => 1,
        Weekday::Tue => 2,
        Weekday::Wed => 3,
        Weekday::Thu => 4,
        Weekday::Fri => 5,
        Weekday::Sat => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn access_action_grants_access() {
        assert!(AccessAction::Allow.grants_access());
        assert!(AccessAction::AllowWithLog.grants_access());
        assert!(!AccessAction::Deny.grants_access());
    }

    #[test]
    fn access_action_requires_log() {
        assert!(!AccessAction::Allow.requires_log());
        assert!(AccessAction::AllowWithLog.requires_log());
        assert!(AccessAction::Deny.requires_log());
    }

    #[test]
    fn access_decision_allow() {
        let decision = AccessDecision::allow("RULE_01", "Floor access granted");
        assert!(decision.is_granted());
        assert!(!decision.log_required);
        assert_eq!(decision.matched_rule, Some("RULE_01".to_string()));
    }

    #[test]
    fn access_decision_deny() {
        let decision = AccessDecision::deny(Some("RULE_02".to_string()), "Access denied");
        assert!(!decision.is_granted());
        assert!(decision.log_required);
    }

    #[test]
    fn access_decision_default_deny() {
        let decision = AccessDecision::default_deny();
        assert!(!decision.is_granted());
        assert!(decision.matched_rule.is_none());
        assert!(decision.reason.contains("No matching"));
    }

    #[test]
    fn evaluation_context_builder() {
        let ctx = EvaluationContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            CredentialType::ResidentBadge,
            "owner",
            "L12",
        )
        .with_clearance(2)
        .with_floors(vec!["L12".to_string(), "L13".to_string()])
        .with_zone("residential");

        assert_eq!(ctx.clearance, 2);
        assert_eq!(ctx.granted_floors.len(), 2);
        assert_eq!(ctx.target_zone, Some("residential".to_string()));
    }

    #[test]
    fn access_rule_applies_to() {
        let rule = AccessRule::new(
            "RULE_01",
            "TNT_01HXK",
            "PRY_01HXK",
            "Allow residents",
            AccessAction::Allow,
            100,
        );

        let ctx = EvaluationContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            CredentialType::ResidentBadge,
            "owner",
            "L12",
        );

        assert!(rule.applies_to(&ctx));

        // Different tenant
        let ctx2 = EvaluationContext::new(
            "TNT_02ABC",
            "PRY_01HXK",
            CredentialType::ResidentBadge,
            "owner",
            "L12",
        );
        assert!(!rule.applies_to(&ctx2));
    }

    #[test]
    fn access_rule_wildcard_property() {
        let rule = AccessRule::new(
            "RULE_01",
            "TNT_01HXK",
            "*",
            "Allow all properties",
            AccessAction::Allow,
            100,
        );

        let ctx1 = EvaluationContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            CredentialType::ResidentBadge,
            "owner",
            "L12",
        );

        let ctx2 = EvaluationContext::new(
            "TNT_01HXK",
            "PRY_99ZZZ",
            CredentialType::ResidentBadge,
            "owner",
            "L12",
        );

        assert!(rule.applies_to(&ctx1));
        assert!(rule.applies_to(&ctx2));
    }

    #[test]
    fn rule_set_sorting() {
        let mut rules = RuleSet::new();

        rules.add(AccessRule::new(
            "LOW",
            "TNT_01",
            "*",
            "Low priority",
            AccessAction::Allow,
            10,
        ));
        rules.add(AccessRule::new(
            "HIGH",
            "TNT_01",
            "*",
            "High priority",
            AccessAction::Deny,
            100,
        ));
        rules.add(AccessRule::new(
            "MED",
            "TNT_01",
            "*",
            "Medium priority",
            AccessAction::AllowWithLog,
            50,
        ));

        let rule_ids: Vec<_> = rules.rules().iter().map(|r| r.rule_id.as_str()).collect();
        assert_eq!(rule_ids, vec!["HIGH", "MED", "LOW"]);
    }

    #[test]
    fn parse_time_valid() {
        let time = parse_time("09:30").unwrap();
        assert_eq!(time.hour(), 9);
        assert_eq!(time.minute(), 30);
    }

    #[test]
    fn parse_time_invalid() {
        assert!(parse_time("invalid").is_none());
        assert!(parse_time("25:00").is_none());
    }

    #[test]
    fn weekday_number_mapping() {
        assert_eq!(weekday_number(Weekday::Sun), 0);
        assert_eq!(weekday_number(Weekday::Mon), 1);
        assert_eq!(weekday_number(Weekday::Sat), 6);
    }
}
