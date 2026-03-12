//! Compliance engine combining all checks.

use std::sync::Arc;

use super::checks::{
    BlacklistCheck, CredentialLimitCheck, IdentityVerificationCheck, UnitOwnershipCheck,
};
use super::types::{ComplianceCheck, ComplianceContext, ComplianceError, ComplianceStatus};

/// Result of compliance evaluation.
#[derive(Debug, Clone)]
pub struct ComplianceResult {
    /// Overall status.
    pub status: ComplianceStatus,

    /// Individual check results.
    pub checks: Vec<CheckResult>,

    /// Errors from failed checks.
    pub errors: Vec<ComplianceError>,
}

impl ComplianceResult {
    /// Whether all checks passed (allowing credential issuance).
    #[must_use]
    pub fn is_compliant(&self) -> bool {
        self.status.allows_issuance()
    }

    /// Get the first error, if any.
    #[must_use]
    pub fn first_error(&self) -> Option<&ComplianceError> {
        self.errors.first()
    }
}

/// Individual check result.
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// Check name.
    pub name: String,
    /// Check status.
    pub status: ComplianceStatus,
    /// Error if failed.
    pub error: Option<ComplianceError>,
}

/// Compliance engine that evaluates all checks.
pub struct ComplianceEngine {
    /// Registered checks.
    checks: Vec<Arc<dyn ComplianceCheck>>,
}

impl ComplianceEngine {
    /// Create a new compliance engine with default checks.
    #[must_use]
    pub fn new() -> Self {
        Self { checks: Vec::new() }
    }

    /// Create a compliance engine with all standard checks.
    #[must_use]
    pub fn with_standard_checks() -> Self {
        let mut engine = Self::new();
        engine.add_check(Arc::new(IdentityVerificationCheck::new()));
        engine.add_check(Arc::new(UnitOwnershipCheck::new()));
        engine.add_check(Arc::new(BlacklistCheck::new()));
        engine.add_check(Arc::new(CredentialLimitCheck::new()));
        engine
    }

    /// Add a compliance check.
    pub fn add_check(&mut self, check: Arc<dyn ComplianceCheck>) {
        self.checks.push(check);
    }

    /// Get a check by name.
    #[must_use]
    pub fn get_check(&self, name: &str) -> Option<&Arc<dyn ComplianceCheck>> {
        self.checks.iter().find(|c| c.name() == name)
    }

    /// Evaluate all applicable checks.
    #[must_use]
    pub fn evaluate(&self, ctx: &ComplianceContext) -> ComplianceResult {
        let mut check_results = Vec::new();
        let mut errors = Vec::new();
        let mut overall_status = ComplianceStatus::Passed;

        for check in &self.checks {
            // Skip checks that don't apply to this credential type
            if !check.applies_to(ctx.credential_type) {
                check_results.push(CheckResult {
                    name: check.name().to_string(),
                    status: ComplianceStatus::Skipped,
                    error: None,
                });
                continue;
            }

            match check.check(ctx) {
                Ok(status) => {
                    // Update overall status
                    if status == ComplianceStatus::Warning
                        && overall_status == ComplianceStatus::Passed
                    {
                        overall_status = ComplianceStatus::Warning;
                    }

                    check_results.push(CheckResult {
                        name: check.name().to_string(),
                        status,
                        error: None,
                    });
                }
                Err(error) => {
                    // Failed check
                    overall_status = ComplianceStatus::Failed;

                    check_results.push(CheckResult {
                        name: check.name().to_string(),
                        status: ComplianceStatus::Failed,
                        error: Some(error.clone()),
                    });

                    errors.push(error);
                }
            }
        }

        ComplianceResult {
            status: overall_status,
            checks: check_results,
            errors,
        }
    }

    /// Evaluate checks and return Ok if compliant, Err with first error otherwise.
    ///
    /// # Errors
    /// Returns the first `ComplianceError` if any check fails.
    pub fn require_compliance(
        &self,
        ctx: &ComplianceContext,
    ) -> Result<ComplianceResult, ComplianceError> {
        let result = self.evaluate(ctx);

        if result.is_compliant() {
            Ok(result)
        } else {
            Err(result.first_error().cloned().unwrap_or_else(|| {
                ComplianceError::new("SAHI_2300", "Compliance check failed", "unknown")
            }))
        }
    }
}

impl Default for ComplianceEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance::types::{
        BlacklistEntry, BlacklistReason, IdentityVerification, OwnershipType, UnitOwnership,
        VerificationMethod,
    };
    use crate::credential_types::CredentialType;
    use chrono::Utc;

    fn setup_compliant_engine() -> ComplianceEngine {
        let identity_check = Arc::new(IdentityVerificationCheck::new());
        identity_check.register(IdentityVerification {
            user_id: "USR_01HXK".to_string(),
            verified: true,
            method: VerificationMethod::MyDigitalId,
            verified_at: Some(Utc::now()),
            provider_ref: Some("MDI-12345".to_string()),
            expires_at: None,
        });

        let ownership_check = Arc::new(UnitOwnershipCheck::new());
        ownership_check.register(UnitOwnership {
            user_id: "USR_01HXK".to_string(),
            property_id: "PRY_01HXK".to_string(),
            unit_id: "12-03".to_string(),
            ownership_type: OwnershipType::Owner,
            confirmed: true,
            confirmed_at: Some(Utc::now()),
            confirmed_by: Some("ADM_01HXK".to_string()),
            valid_from: None,
            valid_until: None,
        });

        let blacklist_check = Arc::new(BlacklistCheck::new());
        let limit_check = Arc::new(CredentialLimitCheck::new());

        let mut engine = ComplianceEngine::new();
        engine.add_check(identity_check);
        engine.add_check(ownership_check);
        engine.add_check(blacklist_check);
        engine.add_check(limit_check);
        engine
    }

    #[test]
    fn engine_all_checks_pass() {
        let engine = setup_compliant_engine();

        let ctx = ComplianceContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            "USR_01HXK",
            CredentialType::ResidentBadge,
        )
        .with_unit("12-03");

        let result = engine.evaluate(&ctx);
        assert!(result.is_compliant());
        assert!(result.errors.is_empty());
        assert_eq!(result.status, ComplianceStatus::Passed);
    }

    #[test]
    fn engine_identity_check_fails() {
        let engine = ComplianceEngine::with_standard_checks();

        let ctx = ComplianceContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            "USR_UNKNOWN",
            CredentialType::ResidentBadge,
        );

        let result = engine.evaluate(&ctx);
        assert!(!result.is_compliant());
        assert!(!result.errors.is_empty());
        assert_eq!(result.status, ComplianceStatus::Failed);
    }

    #[test]
    fn engine_blacklist_blocks() {
        let identity_check = Arc::new(IdentityVerificationCheck::new());
        identity_check.register(IdentityVerification {
            user_id: "USR_01HXK".to_string(),
            verified: true,
            method: VerificationMethod::MyDigitalId,
            verified_at: Some(Utc::now()),
            provider_ref: None,
            expires_at: None,
        });

        let blacklist_check = Arc::new(BlacklistCheck::new());
        blacklist_check.add(BlacklistEntry {
            user_id: "USR_01HXK".to_string(),
            property_id: None,
            reason: BlacklistReason::SecurityViolation,
            description: "Tailgating".to_string(),
            created_at: Utc::now(),
            expires_at: None,
            created_by: "ADM_01HXK".to_string(),
        });

        let mut engine = ComplianceEngine::new();
        engine.add_check(identity_check);
        engine.add_check(blacklist_check);

        let ctx = ComplianceContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            "USR_01HXK",
            CredentialType::ContractorBadge,
        );

        let result = engine.evaluate(&ctx);
        assert!(!result.is_compliant());
        assert!(result.errors.iter().any(|e| e.code.contains("2304")));
    }

    #[test]
    fn engine_require_compliance_returns_error() {
        let engine = ComplianceEngine::with_standard_checks();

        let ctx = ComplianceContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            "USR_UNKNOWN",
            CredentialType::ResidentBadge,
        );

        let result = engine.require_compliance(&ctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.code.contains("2300")); // Identity verification required
    }

    #[test]
    fn engine_skips_non_applicable_checks() {
        let engine = setup_compliant_engine();

        // Visitor pass doesn't require identity verification
        let ctx = ComplianceContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            "USR_01HXK",
            CredentialType::VisitorPass,
        );

        let result = engine.evaluate(&ctx);
        assert!(result.is_compliant());

        // Identity check should be skipped
        let identity_result = result
            .checks
            .iter()
            .find(|c| c.name == "identity_verification");
        assert!(matches!(
            identity_result,
            Some(CheckResult {
                status: ComplianceStatus::Skipped,
                ..
            })
        ));
    }

    #[test]
    fn engine_get_check_by_name() {
        let engine = ComplianceEngine::with_standard_checks();

        assert!(engine.get_check("identity_verification").is_some());
        assert!(engine.get_check("blacklist").is_some());
        assert!(engine.get_check("nonexistent").is_none());
    }

    #[test]
    fn compliance_result_first_error() {
        let engine = ComplianceEngine::with_standard_checks();

        let ctx = ComplianceContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            "USR_UNKNOWN",
            CredentialType::ResidentBadge,
        );

        let result = engine.evaluate(&ctx);
        let first_error = result.first_error();
        assert!(first_error.is_some());
        assert_eq!(first_error.unwrap().check_name, "identity_verification");
    }
}
