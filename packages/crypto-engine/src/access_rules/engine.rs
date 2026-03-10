//! Rules engine for access decisions.

use std::sync::RwLock;

use super::conditions::evaluate_condition;
use super::types::{AccessDecision, AccessRule, EvaluationContext, RuleSet};

/// Rules engine for evaluating access decisions.
///
/// Evaluates rules in priority order and returns the first matching
/// decision. If no rules match, returns a default deny.
pub struct RulesEngine {
    /// Rule sets by tenant ID.
    rule_sets: RwLock<std::collections::HashMap<String, RuleSet>>,
}

impl RulesEngine {
    /// Create a new rules engine.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rule_sets: RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Add a rule to the engine.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    pub fn add_rule(&self, rule: AccessRule) {
        let mut rule_sets = self.rule_sets.write().unwrap();
        let rule_set = rule_sets.entry(rule.tenant_id.clone()).or_default();
        rule_set.add(rule);
    }

    /// Remove a rule from the engine.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    pub fn remove_rule(&self, tenant_id: &str, rule_id: &str) {
        let mut rule_sets = self.rule_sets.write().unwrap();
        if let Some(rule_set) = rule_sets.get_mut(tenant_id) {
            rule_set.remove(rule_id);
        }
    }

    /// Get all rules for a tenant.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    #[must_use]
    pub fn get_rules(&self, tenant_id: &str) -> Vec<AccessRule> {
        let rule_sets = self.rule_sets.read().unwrap();
        rule_sets
            .get(tenant_id)
            .map(|rs| rs.rules().to_vec())
            .unwrap_or_default()
    }

    /// Evaluate access for a given context.
    ///
    /// Rules are evaluated in priority order (highest first).
    /// The first rule where all conditions match determines the decision.
    /// If no rules match, returns a default deny.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    #[must_use]
    pub fn evaluate(&self, ctx: &EvaluationContext) -> AccessDecision {
        let rule_sets = self.rule_sets.read().unwrap();

        // Get applicable rules
        let rules = match rule_sets.get(&ctx.tenant_id) {
            Some(rs) => rs.applicable_rules(ctx),
            None => return AccessDecision::default_deny(),
        };

        // Evaluate each rule in priority order
        for rule in rules {
            let all_matched = rule
                .conditions
                .iter()
                .all(|c| evaluate_condition(c, ctx).matched);

            if all_matched {
                // This rule matches
                return match rule.action {
                    super::types::AccessAction::Allow => {
                        AccessDecision::allow(&rule.rule_id, &rule.name)
                    }
                    super::types::AccessAction::Deny => {
                        AccessDecision::deny(Some(rule.rule_id.clone()), &rule.name)
                    }
                    super::types::AccessAction::AllowWithLog => {
                        AccessDecision::allow_with_log(&rule.rule_id, &rule.name)
                    }
                };
            }
        }

        // No rules matched - default deny
        AccessDecision::default_deny()
    }

    /// Evaluate access and return detailed match information.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    #[must_use]
    pub fn evaluate_detailed(
        &self,
        ctx: &EvaluationContext,
    ) -> (AccessDecision, Vec<RuleEvaluation>) {
        let rule_sets = self.rule_sets.read().unwrap();
        let mut evaluations = Vec::new();

        // Get applicable rules
        let rules = match rule_sets.get(&ctx.tenant_id) {
            Some(rs) => rs.applicable_rules(ctx),
            None => return (AccessDecision::default_deny(), evaluations),
        };

        let mut final_decision: Option<AccessDecision> = None;

        // Evaluate each rule in priority order
        for rule in rules {
            let condition_results: Vec<_> = rule
                .conditions
                .iter()
                .map(|c| evaluate_condition(c, ctx))
                .collect();

            let all_matched = condition_results.iter().all(|r| r.matched);

            evaluations.push(RuleEvaluation {
                rule_id: rule.rule_id.clone(),
                rule_name: rule.name.clone(),
                priority: rule.priority,
                all_conditions_matched: all_matched,
                condition_results,
            });

            // First matching rule wins
            if all_matched && final_decision.is_none() {
                final_decision = Some(match rule.action {
                    super::types::AccessAction::Allow => {
                        AccessDecision::allow(&rule.rule_id, &rule.name)
                    }
                    super::types::AccessAction::Deny => {
                        AccessDecision::deny(Some(rule.rule_id.clone()), &rule.name)
                    }
                    super::types::AccessAction::AllowWithLog => {
                        AccessDecision::allow_with_log(&rule.rule_id, &rule.name)
                    }
                });
            }
        }

        (
            final_decision.unwrap_or_else(AccessDecision::default_deny),
            evaluations,
        )
    }

    /// Clear all rules for a tenant.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    pub fn clear_tenant(&self, tenant_id: &str) {
        let mut rule_sets = self.rule_sets.write().unwrap();
        rule_sets.remove(tenant_id);
    }

    /// Get total number of rules.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    #[must_use]
    pub fn rule_count(&self) -> usize {
        let rule_sets = self.rule_sets.read().unwrap();
        rule_sets.values().map(|rs| rs.rules().len()).sum()
    }
}

impl Default for RulesEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Detailed evaluation result for a single rule.
#[derive(Debug, Clone)]
pub struct RuleEvaluation {
    /// Rule ID.
    pub rule_id: String,
    /// Rule name.
    pub rule_name: String,
    /// Rule priority.
    pub priority: u32,
    /// Whether all conditions matched.
    pub all_conditions_matched: bool,
    /// Individual condition results.
    pub condition_results: Vec<super::types::ConditionMatch>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::access_rules::types::{AccessAction, Condition};
    use crate::credential_types::CredentialType;

    fn test_context() -> EvaluationContext {
        EvaluationContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            CredentialType::ResidentBadge,
            "owner",
            "L12",
        )
        .with_clearance(2)
        .with_zone("residential")
    }

    fn residents_allow_rule() -> AccessRule {
        AccessRule::new(
            "RULE_RESIDENTS",
            "TNT_01HXK",
            "PRY_01HXK",
            "Allow residents",
            AccessAction::Allow,
            100,
        )
        .with_condition(Condition::CredentialType {
            types: vec![CredentialType::ResidentBadge],
        })
        .with_condition(Condition::Role {
            roles: vec!["owner".to_string(), "tenant".to_string()],
        })
    }

    fn visitors_deny_rule() -> AccessRule {
        AccessRule::new(
            "RULE_VISITORS_DENY",
            "TNT_01HXK",
            "PRY_01HXK",
            "Deny visitors after hours",
            AccessAction::Deny,
            50,
        )
        .with_condition(Condition::CredentialType {
            types: vec![CredentialType::VisitorPass],
        })
    }

    #[test]
    fn engine_no_rules_default_deny() {
        let engine = RulesEngine::new();
        let ctx = test_context();

        let decision = engine.evaluate(&ctx);
        assert!(!decision.is_granted());
        assert!(decision.matched_rule.is_none());
    }

    #[test]
    fn engine_matching_allow_rule() {
        let engine = RulesEngine::new();
        engine.add_rule(residents_allow_rule());

        let ctx = test_context();
        let decision = engine.evaluate(&ctx);

        assert!(decision.is_granted());
        assert_eq!(decision.matched_rule, Some("RULE_RESIDENTS".to_string()));
    }

    #[test]
    fn engine_matching_deny_rule() {
        let engine = RulesEngine::new();
        engine.add_rule(visitors_deny_rule());

        let ctx = EvaluationContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            CredentialType::VisitorPass,
            "visitor",
            "L12",
        );

        let decision = engine.evaluate(&ctx);
        assert!(!decision.is_granted());
        assert_eq!(
            decision.matched_rule,
            Some("RULE_VISITORS_DENY".to_string())
        );
    }

    #[test]
    fn engine_priority_order() {
        let engine = RulesEngine::new();

        // Low priority allow
        engine.add_rule(
            AccessRule::new(
                "LOW_ALLOW",
                "TNT_01HXK",
                "PRY_01HXK",
                "Low priority allow",
                AccessAction::Allow,
                10,
            )
            .with_condition(Condition::CredentialType {
                types: vec![CredentialType::ResidentBadge],
            }),
        );

        // High priority deny
        engine.add_rule(
            AccessRule::new(
                "HIGH_DENY",
                "TNT_01HXK",
                "PRY_01HXK",
                "High priority deny",
                AccessAction::Deny,
                100,
            )
            .with_condition(Condition::CredentialType {
                types: vec![CredentialType::ResidentBadge],
            }),
        );

        let ctx = test_context();
        let decision = engine.evaluate(&ctx);

        // High priority deny wins
        assert!(!decision.is_granted());
        assert_eq!(decision.matched_rule, Some("HIGH_DENY".to_string()));
    }

    #[test]
    fn engine_no_match_returns_deny() {
        let engine = RulesEngine::new();

        // Rule that requires visitor credential
        engine.add_rule(
            AccessRule::new(
                "VISITORS_ONLY",
                "TNT_01HXK",
                "PRY_01HXK",
                "Visitors only",
                AccessAction::Allow,
                100,
            )
            .with_condition(Condition::CredentialType {
                types: vec![CredentialType::VisitorPass],
            }),
        );

        // Context with resident credential
        let ctx = test_context();
        let decision = engine.evaluate(&ctx);

        // No match, default deny
        assert!(!decision.is_granted());
        assert!(decision.matched_rule.is_none());
    }

    #[test]
    fn engine_all_conditions_must_match() {
        let engine = RulesEngine::new();

        // Rule requiring both resident badge AND clearance 3
        engine.add_rule(
            AccessRule::new(
                "HIGH_CLEARANCE",
                "TNT_01HXK",
                "PRY_01HXK",
                "High clearance only",
                AccessAction::Allow,
                100,
            )
            .with_condition(Condition::CredentialType {
                types: vec![CredentialType::ResidentBadge],
            })
            .with_condition(Condition::Clearance { min_level: 3 }),
        );

        // Context with clearance 2
        let ctx = test_context(); // clearance = 2
        let decision = engine.evaluate(&ctx);

        // Should not match (clearance too low)
        assert!(!decision.is_granted());
    }

    #[test]
    fn engine_detailed_evaluation() {
        let engine = RulesEngine::new();
        engine.add_rule(residents_allow_rule());

        let ctx = test_context();
        let (decision, evaluations) = engine.evaluate_detailed(&ctx);

        assert!(decision.is_granted());
        assert!(!evaluations.is_empty());

        let eval = &evaluations[0];
        assert_eq!(eval.rule_id, "RULE_RESIDENTS");
        assert!(eval.all_conditions_matched);
        assert_eq!(eval.condition_results.len(), 2);
    }

    #[test]
    fn engine_remove_rule() {
        let engine = RulesEngine::new();
        engine.add_rule(residents_allow_rule());

        assert_eq!(engine.rule_count(), 1);

        engine.remove_rule("TNT_01HXK", "RULE_RESIDENTS");

        assert_eq!(engine.rule_count(), 0);
    }

    #[test]
    fn engine_clear_tenant() {
        let engine = RulesEngine::new();
        engine.add_rule(residents_allow_rule());
        engine.add_rule(visitors_deny_rule());

        assert_eq!(engine.rule_count(), 2);

        engine.clear_tenant("TNT_01HXK");

        assert_eq!(engine.rule_count(), 0);
    }

    #[test]
    fn engine_get_rules() {
        let engine = RulesEngine::new();
        engine.add_rule(residents_allow_rule());
        engine.add_rule(visitors_deny_rule());

        let rules = engine.get_rules("TNT_01HXK");
        assert_eq!(rules.len(), 2);

        // Sorted by priority
        assert_eq!(rules[0].rule_id, "RULE_RESIDENTS"); // priority 100
        assert_eq!(rules[1].rule_id, "RULE_VISITORS_DENY"); // priority 50
    }

    #[test]
    fn engine_allow_with_log() {
        let engine = RulesEngine::new();

        engine.add_rule(
            AccessRule::new(
                "EMERGENCY_ACCESS",
                "TNT_01HXK",
                "*",
                "Emergency access (logged)",
                AccessAction::AllowWithLog,
                200,
            )
            .with_condition(Condition::CredentialType {
                types: vec![CredentialType::EmergencyAccess],
            }),
        );

        let ctx = EvaluationContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            CredentialType::EmergencyAccess,
            "fire_dept",
            "L12",
        );

        let decision = engine.evaluate(&ctx);
        assert!(decision.is_granted());
        assert!(decision.log_required);
    }
}
