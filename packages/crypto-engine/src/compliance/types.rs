//! Compliance types and data structures.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::credential_types::CredentialType;

/// Context for compliance evaluation.
#[derive(Debug, Clone)]
pub struct ComplianceContext {
    /// Tenant ID.
    pub tenant_id: String,

    /// Property ID.
    pub property_id: String,

    /// User ID (applicant).
    pub user_id: String,

    /// Requested credential type.
    pub credential_type: CredentialType,

    /// Unit ID (for residential credentials).
    pub unit_id: Option<String>,

    /// Additional metadata for checks.
    pub metadata: std::collections::HashMap<String, String>,
}

impl ComplianceContext {
    /// Create a new compliance context.
    pub fn new(
        tenant_id: impl Into<String>,
        property_id: impl Into<String>,
        user_id: impl Into<String>,
        credential_type: CredentialType,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            property_id: property_id.into(),
            user_id: user_id.into(),
            credential_type,
            unit_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set the unit ID.
    #[must_use]
    pub fn with_unit(mut self, unit_id: impl Into<String>) -> Self {
        self.unit_id = Some(unit_id.into());
        self
    }

    /// Add metadata.
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Compliance check status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatus {
    /// Check passed.
    Passed,
    /// Check failed (hard block).
    Failed,
    /// Check passed with warning (soft issue).
    Warning,
    /// Check was skipped (not applicable).
    Skipped,
    /// Check is pending (async verification in progress).
    Pending,
}

impl ComplianceStatus {
    /// Whether this status allows credential issuance.
    #[must_use]
    pub fn allows_issuance(self) -> bool {
        matches!(self, Self::Passed | Self::Warning | Self::Skipped)
    }
}

/// Compliance check error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceError {
    /// Error code (SAHI_XXXX format).
    pub code: String,

    /// Human-readable message.
    pub message: String,

    /// Check that failed.
    pub check_name: String,

    /// Additional details.
    pub details: Option<String>,
}

impl ComplianceError {
    /// Create a new compliance error.
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        check_name: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            check_name: check_name.into(),
            details: None,
        }
    }

    /// Add details.
    #[must_use]
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

impl std::fmt::Display for ComplianceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.code, self.check_name, self.message)
    }
}

impl std::error::Error for ComplianceError {}

/// Trait for compliance checks.
pub trait ComplianceCheck: Send + Sync {
    /// Name of this check (for logging/reporting).
    fn name(&self) -> &'static str;

    /// Execute the compliance check.
    ///
    /// # Errors
    /// Returns `ComplianceError` if the check fails.
    fn check(&self, ctx: &ComplianceContext) -> Result<ComplianceStatus, ComplianceError>;

    /// Whether this check is required for the given credential type.
    fn applies_to(&self, credential_type: CredentialType) -> bool;
}

/// Identity verification status (eKYC via MyDigital ID).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityVerification {
    /// User ID.
    pub user_id: String,

    /// Verification status.
    pub verified: bool,

    /// Verification method used.
    pub method: VerificationMethod,

    /// When verification was completed.
    pub verified_at: Option<DateTime<Utc>>,

    /// Verification provider reference.
    pub provider_ref: Option<String>,

    /// Expiry of verification (some require re-verification).
    pub expires_at: Option<DateTime<Utc>>,
}

impl IdentityVerification {
    /// Check if verification is valid (verified and not expired).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        if !self.verified {
            return false;
        }
        match self.expires_at {
            Some(exp) => exp > Utc::now(),
            None => true,
        }
    }
}

/// Identity verification method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationMethod {
    /// MyDigital ID (Malaysian national digital identity).
    MyDigitalId,
    /// Manual verification by property admin.
    ManualReview,
    /// Document upload + OCR verification.
    DocumentOcr,
    /// In-person verification.
    InPerson,
}

/// Unit ownership/tenancy status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitOwnership {
    /// User ID.
    pub user_id: String,

    /// Property ID.
    pub property_id: String,

    /// Unit ID.
    pub unit_id: String,

    /// Ownership type.
    pub ownership_type: OwnershipType,

    /// Whether ownership is confirmed by property admin.
    pub confirmed: bool,

    /// Confirmation date.
    pub confirmed_at: Option<DateTime<Utc>>,

    /// Confirming admin user ID.
    pub confirmed_by: Option<String>,

    /// Ownership start date.
    pub valid_from: Option<DateTime<Utc>>,

    /// Ownership end date (for tenants).
    pub valid_until: Option<DateTime<Utc>>,
}

impl UnitOwnership {
    /// Check if ownership is currently valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        if !self.confirmed {
            return false;
        }
        let now = Utc::now();
        let after_start = self.valid_from.is_none_or(|v| now >= v);
        let before_end = self.valid_until.is_none_or(|v| now <= v);
        after_start && before_end
    }
}

/// Unit ownership type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnershipType {
    /// Property owner.
    Owner,
    /// Tenant with lease.
    Tenant,
    /// Family member of owner/tenant.
    FamilyMember,
    /// Authorized occupant.
    Occupant,
}

/// Blacklist entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistEntry {
    /// User ID.
    pub user_id: String,

    /// Property ID (None = all properties for tenant).
    pub property_id: Option<String>,

    /// Blacklist reason.
    pub reason: BlacklistReason,

    /// Human-readable description.
    pub description: String,

    /// When the blacklist entry was created.
    pub created_at: DateTime<Utc>,

    /// When the blacklist expires (None = permanent).
    pub expires_at: Option<DateTime<Utc>>,

    /// Created by (admin user ID).
    pub created_by: String,
}

impl BlacklistEntry {
    /// Check if blacklist is currently active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        match self.expires_at {
            Some(exp) => exp > Utc::now(),
            None => true, // Permanent
        }
    }

    /// Check if blacklist applies to a specific property.
    #[must_use]
    pub fn applies_to_property(&self, property_id: &str) -> bool {
        match &self.property_id {
            Some(pid) => pid == property_id,
            None => true, // Tenant-wide
        }
    }
}

/// Blacklist reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlacklistReason {
    /// Security violation (tailgating, credential sharing).
    SecurityViolation,
    /// Non-payment of fees.
    NonPayment,
    /// Misconduct (noise complaints, damage).
    Misconduct,
    /// Legal/court order.
    LegalOrder,
    /// Fraudulent activity.
    Fraud,
    /// Other (see description).
    Other,
}

/// Credential limit configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialLimit {
    /// Credential type.
    pub credential_type: CredentialType,

    /// Maximum active credentials per user.
    pub max_per_user: u32,

    /// Maximum active credentials per unit.
    pub max_per_unit: Option<u32>,

    /// Maximum issuances per day (rate limit).
    pub max_per_day: Option<u32>,
}

impl Default for CredentialLimit {
    fn default() -> Self {
        Self {
            credential_type: CredentialType::ResidentBadge,
            max_per_user: 3,       // 3 devices per user
            max_per_unit: Some(6), // 6 total per unit
            max_per_day: Some(10), // Prevent abuse
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compliance_status_allows_issuance() {
        assert!(ComplianceStatus::Passed.allows_issuance());
        assert!(ComplianceStatus::Warning.allows_issuance());
        assert!(ComplianceStatus::Skipped.allows_issuance());
        assert!(!ComplianceStatus::Failed.allows_issuance());
        assert!(!ComplianceStatus::Pending.allows_issuance());
    }

    #[test]
    fn identity_verification_validity() {
        let mut verification = IdentityVerification {
            user_id: "USR_01HXK".to_string(),
            verified: true,
            method: VerificationMethod::MyDigitalId,
            verified_at: Some(Utc::now()),
            provider_ref: Some("MDI-12345".to_string()),
            expires_at: None,
        };

        assert!(verification.is_valid());

        // Not verified
        verification.verified = false;
        assert!(!verification.is_valid());

        // Verified but expired
        verification.verified = true;
        verification.expires_at = Some(Utc::now() - chrono::Duration::hours(1));
        assert!(!verification.is_valid());

        // Verified and not expired
        verification.expires_at = Some(Utc::now() + chrono::Duration::hours(1));
        assert!(verification.is_valid());
    }

    #[test]
    fn unit_ownership_validity() {
        let mut ownership = UnitOwnership {
            user_id: "USR_01HXK".to_string(),
            property_id: "PRY_01HXK".to_string(),
            unit_id: "12-03".to_string(),
            ownership_type: OwnershipType::Owner,
            confirmed: true,
            confirmed_at: Some(Utc::now()),
            confirmed_by: Some("ADM_01HXK".to_string()),
            valid_from: None,
            valid_until: None,
        };

        assert!(ownership.is_valid());

        // Not confirmed
        ownership.confirmed = false;
        assert!(!ownership.is_valid());

        // Confirmed but expired
        ownership.confirmed = true;
        ownership.valid_until = Some(Utc::now() - chrono::Duration::days(1));
        assert!(!ownership.is_valid());

        // Confirmed and valid
        ownership.valid_from = Some(Utc::now() - chrono::Duration::days(30));
        ownership.valid_until = Some(Utc::now() + chrono::Duration::days(30));
        assert!(ownership.is_valid());
    }

    #[test]
    fn blacklist_entry_active() {
        let mut entry = BlacklistEntry {
            user_id: "USR_01HXK".to_string(),
            property_id: None,
            reason: BlacklistReason::SecurityViolation,
            description: "Tailgating incident".to_string(),
            created_at: Utc::now(),
            expires_at: None,
            created_by: "ADM_01HXK".to_string(),
        };

        // Permanent blacklist
        assert!(entry.is_active());

        // Expired blacklist
        entry.expires_at = Some(Utc::now() - chrono::Duration::days(1));
        assert!(!entry.is_active());

        // Active temporary blacklist
        entry.expires_at = Some(Utc::now() + chrono::Duration::days(30));
        assert!(entry.is_active());
    }

    #[test]
    fn blacklist_property_scope() {
        let tenant_wide = BlacklistEntry {
            user_id: "USR_01HXK".to_string(),
            property_id: None,
            reason: BlacklistReason::Fraud,
            description: "Fraudulent credentials".to_string(),
            created_at: Utc::now(),
            expires_at: None,
            created_by: "ADM_01HXK".to_string(),
        };

        let property_specific = BlacklistEntry {
            user_id: "USR_01HXK".to_string(),
            property_id: Some("PRY_01HXK".to_string()),
            reason: BlacklistReason::Misconduct,
            description: "Property damage".to_string(),
            created_at: Utc::now(),
            expires_at: None,
            created_by: "ADM_01HXK".to_string(),
        };

        // Tenant-wide applies to any property
        assert!(tenant_wide.applies_to_property("PRY_01HXK"));
        assert!(tenant_wide.applies_to_property("PRY_02ABC"));

        // Property-specific only applies to that property
        assert!(property_specific.applies_to_property("PRY_01HXK"));
        assert!(!property_specific.applies_to_property("PRY_02ABC"));
    }

    #[test]
    fn compliance_error_display() {
        let error = ComplianceError::new(
            "SAHI_3001",
            "Identity verification required",
            "identity_verification",
        )
        .with_details("User has not completed eKYC");

        assert!(error.to_string().contains("SAHI_3001"));
        assert!(error.to_string().contains("identity_verification"));
    }

    #[test]
    fn compliance_context_builder() {
        let ctx = ComplianceContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            "USR_01HXK",
            CredentialType::ResidentBadge,
        )
        .with_unit("12-03")
        .with_metadata("floor", "12");

        assert_eq!(ctx.tenant_id, "TNT_01HXK");
        assert_eq!(ctx.unit_id, Some("12-03".to_string()));
        assert_eq!(ctx.metadata.get("floor"), Some(&"12".to_string()));
    }
}
