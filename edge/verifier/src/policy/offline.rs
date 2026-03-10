//! Offline mode policy handling.
//!
//! When connectivity is lost, the verifier operates in degraded mode
//! with cached data and allow-list restrictions.

use tracing::warn;

use super::rules::{AccessDecision, EvaluationContext};
use crate::config::OfflineConfig;

/// Offline policy handler.
#[derive(Debug, Clone)]
pub struct OfflinePolicy {
    /// Allow cached credentials.
    allow_cached: bool,

    /// Pre-approved holder DIDs.
    allow_list: Vec<String>,
}

impl OfflinePolicy {
    /// Create from configuration.
    pub fn new(config: &OfflineConfig) -> Self {
        Self {
            allow_cached: config.allow_cached,
            allow_list: config.allow_list.clone(),
        }
    }

    /// Evaluate access in offline mode.
    ///
    /// In offline mode:
    /// - Only allow-listed DIDs are permitted
    /// - Or deny all if allow-list is empty
    pub fn evaluate(&self, context: &EvaluationContext) -> AccessDecision {
        // If allow-list is empty, default deny in offline mode
        if self.allow_list.is_empty() {
            warn!("Offline mode: No allow-list configured, denying access");
            return AccessDecision::Deny;
        }

        // Check if holder is on allow-list
        let Some(holder_did) = &context.holder_did else {
            warn!("Offline mode: No holder DID, denying access");
            return AccessDecision::Deny;
        };

        if self.allow_list.contains(holder_did) {
            warn!("Offline mode: Holder on allow-list, granting access");
            AccessDecision::AllowWithLog
        } else {
            warn!(
                "Offline mode: Holder not on allow-list, denying access (DID: {})",
                holder_did
            );
            AccessDecision::Deny
        }
    }

    /// Check if a DID is on the allow-list.
    pub fn is_allowed(&self, holder_did: &str) -> bool {
        self.allow_list.contains(&holder_did.to_string())
    }

    /// Add a DID to the allow-list.
    pub fn add_to_allow_list(&mut self, holder_did: &str) {
        if !self.allow_list.contains(&holder_did.to_string()) {
            self.allow_list.push(holder_did.to_string());
        }
    }

    /// Remove a DID from the allow-list.
    pub fn remove_from_allow_list(&mut self, holder_did: &str) {
        self.allow_list.retain(|d| d != holder_did);
    }

    /// Get the allow-list.
    pub fn allow_list(&self) -> &[String] {
        &self.allow_list
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn test_context(holder_did: Option<&str>) -> EvaluationContext {
        EvaluationContext {
            current_time: Utc::now(),
            property_id: "PRY_01HXK".to_string(),
            holder_did: holder_did.map(String::from),
            issuer_did: None,
            credential_type: None,
            requested_floor: None,
            zone_id: None,
        }
    }

    #[test]
    fn test_empty_allow_list_denies() {
        let policy = OfflinePolicy::new(&OfflineConfig {
            allow_cached: true,
            max_offline_duration: std::time::Duration::from_secs(86400),
            allow_list: vec![],
        });

        let context = test_context(Some("did:key:z6Mk..."));
        assert_eq!(policy.evaluate(&context), AccessDecision::Deny);
    }

    #[test]
    fn test_allowed_holder() {
        let policy = OfflinePolicy::new(&OfflineConfig {
            allow_cached: true,
            max_offline_duration: std::time::Duration::from_secs(86400),
            allow_list: vec!["did:key:z6Mk...".to_string()],
        });

        let context = test_context(Some("did:key:z6Mk..."));
        assert_eq!(policy.evaluate(&context), AccessDecision::AllowWithLog);
    }

    #[test]
    fn test_not_on_allow_list() {
        let policy = OfflinePolicy::new(&OfflineConfig {
            allow_cached: true,
            max_offline_duration: std::time::Duration::from_secs(86400),
            allow_list: vec!["did:key:allowed".to_string()],
        });

        let context = test_context(Some("did:key:not-allowed"));
        assert_eq!(policy.evaluate(&context), AccessDecision::Deny);
    }

    #[test]
    fn test_no_holder_did() {
        let policy = OfflinePolicy::new(&OfflineConfig {
            allow_cached: true,
            max_offline_duration: std::time::Duration::from_secs(86400),
            allow_list: vec!["did:key:z6Mk...".to_string()],
        });

        let context = test_context(None);
        assert_eq!(policy.evaluate(&context), AccessDecision::Deny);
    }
}
