//! Policy evaluation engine.

use chrono::Utc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::offline::OfflinePolicy;
use super::rules::{AccessDecision, AccessRule, EvaluationContext};
use crate::config::VerifierConfig;

/// Policy engine for access decisions.
///
/// Evaluates rules in priority order, returning the first match.
/// Default action is DENY (fail-closed per spec).
pub struct PolicyEngine {
    /// Access rules sorted by priority.
    rules: RwLock<Vec<AccessRule>>,

    /// Offline policy handler.
    offline: OfflinePolicy,

    /// Property ID for this verifier.
    property_id: String,

    /// Whether we're currently offline.
    is_offline: RwLock<bool>,
}

impl PolicyEngine {
    /// Create a new policy engine.
    pub fn new(config: &VerifierConfig) -> Self {
        Self {
            rules: RwLock::new(Vec::new()),
            offline: OfflinePolicy::new(&config.offline),
            property_id: config.property_id.clone(),
            is_offline: RwLock::new(false),
        }
    }

    /// Evaluate access for a credential presentation.
    pub async fn evaluate(
        &self,
        holder_did: Option<&str>,
        issuer_did: Option<&str>,
        credential_type: Option<&str>,
        requested_floor: Option<&str>,
        zone_id: Option<&str>,
    ) -> AccessDecision {
        let context = EvaluationContext {
            current_time: Utc::now(),
            property_id: self.property_id.clone(),
            holder_did: holder_did.map(String::from),
            issuer_did: issuer_did.map(String::from),
            credential_type: credential_type.map(String::from),
            requested_floor: requested_floor.map(String::from),
            zone_id: zone_id.map(String::from),
        };

        // Check offline mode
        let is_offline = *self.is_offline.read().await;
        if is_offline {
            return self.offline.evaluate(&context);
        }

        // Evaluate rules in priority order
        let rules = self.rules.read().await;
        let mut sorted_rules: Vec<_> = rules.iter().collect();
        sorted_rules.sort_by_key(|r| r.priority);

        for rule in sorted_rules {
            if let Some(decision) = rule.evaluate(&context) {
                debug!(
                    "Rule '{}' matched, decision: {:?}",
                    rule.name, decision
                );
                return decision;
            }
        }

        // Default: DENY (fail-closed)
        warn!("No rules matched, defaulting to DENY");
        AccessDecision::Deny
    }

    /// Load rules from configuration.
    pub async fn load_rules(&self, rules: Vec<AccessRule>) {
        let mut current = self.rules.write().await;
        *current = rules;
        info!("Loaded {} access rules", current.len());
    }

    /// Add a single rule.
    pub async fn add_rule(&self, rule: AccessRule) {
        let mut rules = self.rules.write().await;
        rules.push(rule);
    }

    /// Remove a rule by ID.
    pub async fn remove_rule(&self, rule_id: &str) {
        let mut rules = self.rules.write().await;
        rules.retain(|r| r.id != rule_id);
    }

    /// Set offline mode.
    pub async fn set_offline(&self, offline: bool) {
        let mut is_offline = self.is_offline.write().await;
        *is_offline = offline;
        if offline {
            warn!("Entering offline mode");
        } else {
            info!("Exiting offline mode");
        }
    }

    /// Check if in offline mode.
    pub async fn is_offline(&self) -> bool {
        *self.is_offline.read().await
    }

    /// Get rule count.
    pub async fn rule_count(&self) -> usize {
        self.rules.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{HardwareConfig, OfflineConfig, VerifierConfig, BleConfig, NfcConfig};
    use std::path::PathBuf;
    use std::time::Duration;

    fn test_config() -> VerifierConfig {
        VerifierConfig {
            site_id: "VRF_01HXK".to_string(),
            property_id: "PRY_01HXK".to_string(),
            tenant_id: "TNT_01HXK".to_string(),
            api_base_url: "https://api.test".to_string(),
            mqtt_broker_url: "mqtt://localhost".to_string(),
            mqtt_client_id: None,
            audit_db_path: PathBuf::from("/tmp/audit.db"),
            status_list_ttl: Duration::from_secs(900),
            stale_threshold: Duration::from_secs(14400),
            trust_registry_ttl: Duration::from_secs(14400),
            ble: BleConfig::default(),
            nfc: NfcConfig::default(),
            hardware: HardwareConfig::default(),
            offline: OfflineConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_default_deny() {
        let engine = PolicyEngine::new(&test_config());

        let decision = engine.evaluate(
            Some("did:key:z6Mk..."),
            Some("did:web:sahi.my"),
            Some("ResidentBadge"),
            None,
            None,
        ).await;

        assert_eq!(decision, AccessDecision::Deny);
    }

    #[tokio::test]
    async fn test_rule_match() {
        let engine = PolicyEngine::new(&test_config());

        // Add an allow rule
        engine.add_rule(AccessRule {
            id: "RULE_01".to_string(),
            name: "Allow all residents".to_string(),
            priority: 1,
            conditions: vec![
                super::super::rules::RuleCondition::CredentialType {
                    allowed: vec!["ResidentBadge".to_string()],
                },
            ],
            action: AccessDecision::Allow,
            enabled: true,
        }).await;

        let decision = engine.evaluate(
            Some("did:key:z6Mk..."),
            Some("did:web:sahi.my"),
            Some("ResidentBadge"),
            None,
            None,
        ).await;

        assert_eq!(decision, AccessDecision::Allow);
    }

    #[tokio::test]
    async fn test_priority_order() {
        let engine = PolicyEngine::new(&test_config());

        // Lower priority (runs first) denies
        engine.add_rule(AccessRule {
            id: "RULE_01".to_string(),
            name: "Deny contractors".to_string(),
            priority: 1,
            conditions: vec![
                super::super::rules::RuleCondition::CredentialType {
                    allowed: vec!["ContractorBadge".to_string()],
                },
            ],
            action: AccessDecision::Deny,
            enabled: true,
        }).await;

        // Higher priority (runs second) would allow
        engine.add_rule(AccessRule {
            id: "RULE_02".to_string(),
            name: "Allow all".to_string(),
            priority: 10,
            conditions: vec![],
            action: AccessDecision::Allow,
            enabled: true,
        }).await;

        let decision = engine.evaluate(
            None,
            None,
            Some("ContractorBadge"),
            None,
            None,
        ).await;

        // First matching rule wins
        assert_eq!(decision, AccessDecision::Deny);
    }
}
