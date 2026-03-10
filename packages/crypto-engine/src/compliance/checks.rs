//! Individual compliance check implementations.

use std::sync::RwLock;

use crate::credential_types::CredentialType;

use super::types::{
    BlacklistEntry, ComplianceCheck, ComplianceContext, ComplianceError, ComplianceStatus,
    CredentialLimit, IdentityVerification, UnitOwnership,
};

/// Identity verification check (eKYC via MyDigital ID).
///
/// Verifies that the user has completed identity verification
/// before credential issuance.
pub struct IdentityVerificationCheck {
    /// Lookup function for identity verification status.
    verifications: RwLock<std::collections::HashMap<String, IdentityVerification>>,
}

impl IdentityVerificationCheck {
    /// Create a new identity verification check.
    #[must_use]
    pub fn new() -> Self {
        Self {
            verifications: RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Register a verification status (for testing/mocking).
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    pub fn register(&self, verification: IdentityVerification) {
        let mut verifications = self.verifications.write().unwrap();
        verifications.insert(verification.user_id.clone(), verification);
    }

    /// Get verification status.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    #[must_use]
    pub fn get(&self, user_id: &str) -> Option<IdentityVerification> {
        let verifications = self.verifications.read().unwrap();
        verifications.get(user_id).cloned()
    }
}

impl Default for IdentityVerificationCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl ComplianceCheck for IdentityVerificationCheck {
    fn name(&self) -> &'static str {
        "identity_verification"
    }

    fn check(&self, ctx: &ComplianceContext) -> Result<ComplianceStatus, ComplianceError> {
        let verifications = self.verifications.read().unwrap();

        match verifications.get(&ctx.user_id) {
            Some(verification) => {
                if verification.is_valid() {
                    Ok(ComplianceStatus::Passed)
                } else if verification.verified {
                    // Verified but expired
                    Err(ComplianceError::new(
                        "SAHI_2301",
                        "Identity verification has expired",
                        self.name(),
                    )
                    .with_details("Please complete re-verification"))
                } else {
                    Err(ComplianceError::new(
                        "SAHI_2300",
                        "Identity verification required",
                        self.name(),
                    )
                    .with_details("Complete eKYC via MyDigital ID"))
                }
            }
            None => Err(ComplianceError::new(
                "SAHI_2300",
                "Identity verification required",
                self.name(),
            )
            .with_details("No verification record found")),
        }
    }

    fn applies_to(&self, credential_type: CredentialType) -> bool {
        // Required for all credential types except visitor passes
        // (visitors are verified by host invitation)
        !matches!(credential_type, CredentialType::VisitorPass)
    }
}

/// Unit ownership check.
///
/// Verifies that the user has confirmed ownership/tenancy
/// of the unit before issuing residential credentials.
pub struct UnitOwnershipCheck {
    /// Ownership records.
    ownerships: RwLock<Vec<UnitOwnership>>,
}

impl UnitOwnershipCheck {
    /// Create a new unit ownership check.
    #[must_use]
    pub fn new() -> Self {
        Self {
            ownerships: RwLock::new(Vec::new()),
        }
    }

    /// Register an ownership record.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    pub fn register(&self, ownership: UnitOwnership) {
        let mut ownerships = self.ownerships.write().unwrap();
        ownerships.push(ownership);
    }

    /// Get ownership for a user and unit.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    #[must_use]
    pub fn get(&self, user_id: &str, property_id: &str, unit_id: &str) -> Option<UnitOwnership> {
        let ownerships = self.ownerships.read().unwrap();
        ownerships
            .iter()
            .find(|o| o.user_id == user_id && o.property_id == property_id && o.unit_id == unit_id)
            .cloned()
    }
}

impl Default for UnitOwnershipCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl ComplianceCheck for UnitOwnershipCheck {
    fn name(&self) -> &'static str {
        "unit_ownership"
    }

    fn check(&self, ctx: &ComplianceContext) -> Result<ComplianceStatus, ComplianceError> {
        // Unit ID is required for this check
        let Some(unit_id) = &ctx.unit_id else {
            // No unit specified - skip for non-residential credentials
            return Ok(ComplianceStatus::Skipped);
        };

        let ownerships = self.ownerships.read().unwrap();

        let ownership = ownerships.iter().find(|o| {
            o.user_id == ctx.user_id && o.property_id == ctx.property_id && o.unit_id == *unit_id
        });

        match ownership {
            Some(o) => {
                if o.is_valid() {
                    Ok(ComplianceStatus::Passed)
                } else if !o.confirmed {
                    Err(ComplianceError::new(
                        "SAHI_2302",
                        "Unit ownership pending confirmation",
                        self.name(),
                    )
                    .with_details("Property admin has not confirmed ownership"))
                } else {
                    Err(ComplianceError::new(
                        "SAHI_2303",
                        "Unit ownership has expired",
                        self.name(),
                    )
                    .with_details("Lease or ownership period has ended"))
                }
            }
            None => Err(ComplianceError::new(
                "SAHI_2302",
                "No ownership record found",
                self.name(),
            )
            .with_details("User has no registered ownership for this unit")),
        }
    }

    fn applies_to(&self, credential_type: CredentialType) -> bool {
        // Only for resident badges (owner/tenant credentials)
        matches!(credential_type, CredentialType::ResidentBadge)
    }
}

/// Blacklist check.
///
/// Verifies that the user is not blacklisted from the property
/// or tenant.
pub struct BlacklistCheck {
    /// Blacklist entries.
    entries: RwLock<Vec<BlacklistEntry>>,
}

impl BlacklistCheck {
    /// Create a new blacklist check.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
        }
    }

    /// Add a blacklist entry.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    pub fn add(&self, entry: BlacklistEntry) {
        let mut entries = self.entries.write().unwrap();
        entries.push(entry);
    }

    /// Remove a blacklist entry (by user and property).
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    pub fn remove(&self, user_id: &str, property_id: Option<&str>) {
        let mut entries = self.entries.write().unwrap();
        entries.retain(|e| {
            !(e.user_id == user_id && e.property_id.as_deref() == property_id)
        });
    }

    /// Get active blacklist entries for a user.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    #[must_use]
    pub fn get_active(&self, user_id: &str, property_id: &str) -> Vec<BlacklistEntry> {
        let entries = self.entries.read().unwrap();
        entries
            .iter()
            .filter(|e| {
                e.user_id == user_id && e.is_active() && e.applies_to_property(property_id)
            })
            .cloned()
            .collect()
    }
}

impl Default for BlacklistCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl ComplianceCheck for BlacklistCheck {
    fn name(&self) -> &'static str {
        "blacklist"
    }

    fn check(&self, ctx: &ComplianceContext) -> Result<ComplianceStatus, ComplianceError> {
        let entries = self.entries.read().unwrap();

        let active_entries: Vec<_> = entries
            .iter()
            .filter(|e| {
                e.user_id == ctx.user_id
                    && e.is_active()
                    && e.applies_to_property(&ctx.property_id)
            })
            .collect();

        if active_entries.is_empty() {
            Ok(ComplianceStatus::Passed)
        } else {
            // Use the most severe/recent entry
            let entry = active_entries.first().unwrap();
            Err(ComplianceError::new(
                "SAHI_2304",
                format!("User is blacklisted: {:?}", entry.reason),
                self.name(),
            )
            .with_details(&entry.description))
        }
    }

    fn applies_to(&self, _credential_type: CredentialType) -> bool {
        // Applies to all credential types
        true
    }
}

/// Credential limit check.
///
/// Verifies that the user has not exceeded credential limits
/// (per user, per unit, per day).
pub struct CredentialLimitCheck {
    /// Credential limit configuration.
    limits: RwLock<std::collections::HashMap<CredentialType, CredentialLimit>>,
    /// Active credential counts (user_id -> type -> count).
    active_counts: RwLock<std::collections::HashMap<String, std::collections::HashMap<CredentialType, u32>>>,
    /// Daily issuance counts (user_id:date -> type -> count).
    daily_counts: RwLock<std::collections::HashMap<String, std::collections::HashMap<CredentialType, u32>>>,
}

impl CredentialLimitCheck {
    /// Create a new credential limit check with default limits.
    #[must_use]
    pub fn new() -> Self {
        let mut limits = std::collections::HashMap::new();

        // Default limits per credential type
        limits.insert(
            CredentialType::ResidentBadge,
            CredentialLimit {
                credential_type: CredentialType::ResidentBadge,
                max_per_user: 3,       // 3 devices
                max_per_unit: Some(6), // 6 per unit
                max_per_day: Some(10),
            },
        );

        limits.insert(
            CredentialType::VisitorPass,
            CredentialLimit {
                credential_type: CredentialType::VisitorPass,
                max_per_user: 1,        // 1 active pass
                max_per_unit: Some(20), // 20 visitors per unit per day
                max_per_day: Some(50),
            },
        );

        limits.insert(
            CredentialType::ContractorBadge,
            CredentialLimit {
                credential_type: CredentialType::ContractorBadge,
                max_per_user: 1,
                max_per_unit: None,
                max_per_day: Some(20),
            },
        );

        limits.insert(
            CredentialType::EmergencyAccess,
            CredentialLimit {
                credential_type: CredentialType::EmergencyAccess,
                max_per_user: 1,
                max_per_unit: None,
                max_per_day: Some(5),
            },
        );

        Self {
            limits: RwLock::new(limits),
            active_counts: RwLock::new(std::collections::HashMap::new()),
            daily_counts: RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Set a custom limit for a credential type.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    pub fn set_limit(&self, limit: CredentialLimit) {
        let mut limits = self.limits.write().unwrap();
        limits.insert(limit.credential_type, limit);
    }

    /// Record a credential issuance (for tracking counts).
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    pub fn record_issuance(&self, user_id: &str, credential_type: CredentialType) {
        // Update active count
        {
            let mut active_counts = self.active_counts.write().unwrap();
            let user_counts = active_counts
                .entry(user_id.to_string())
                .or_default();
            *user_counts.entry(credential_type).or_insert(0) += 1;
        }

        // Update daily count
        {
            let date_key = format!("{}:{}", user_id, chrono::Utc::now().format("%Y-%m-%d"));
            let mut daily_counts = self.daily_counts.write().unwrap();
            let day_counts = daily_counts.entry(date_key).or_default();
            *day_counts.entry(credential_type).or_insert(0) += 1;
        }
    }

    /// Record a credential revocation (decrement active count).
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    pub fn record_revocation(&self, user_id: &str, credential_type: CredentialType) {
        let mut active_counts = self.active_counts.write().unwrap();
        if let Some(user_counts) = active_counts.get_mut(user_id) {
            if let Some(count) = user_counts.get_mut(&credential_type) {
                *count = count.saturating_sub(1);
            }
        }
    }

    /// Get current active count for a user and type.
    ///
    /// # Panics
    /// Panics if the lock is poisoned.
    #[must_use]
    pub fn get_active_count(&self, user_id: &str, credential_type: CredentialType) -> u32 {
        let active_counts = self.active_counts.read().unwrap();
        active_counts
            .get(user_id)
            .and_then(|m| m.get(&credential_type))
            .copied()
            .unwrap_or(0)
    }
}

impl Default for CredentialLimitCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl ComplianceCheck for CredentialLimitCheck {
    fn name(&self) -> &'static str {
        "credential_limit"
    }

    fn check(&self, ctx: &ComplianceContext) -> Result<ComplianceStatus, ComplianceError> {
        let limits = self.limits.read().unwrap();
        let active_counts = self.active_counts.read().unwrap();
        let daily_counts = self.daily_counts.read().unwrap();

        let Some(limit) = limits.get(&ctx.credential_type) else {
            return Ok(ComplianceStatus::Skipped); // No limits configured
        };

        // Check per-user limit
        let user_count = active_counts
            .get(&ctx.user_id)
            .and_then(|m| m.get(&ctx.credential_type))
            .copied()
            .unwrap_or(0);

        if user_count >= limit.max_per_user {
            return Err(ComplianceError::new(
                "SAHI_2305",
                format!(
                    "User has reached maximum active credentials ({})",
                    limit.max_per_user
                ),
                self.name(),
            )
            .with_details("Revoke an existing credential before issuing a new one"));
        }

        // Check daily rate limit
        if let Some(max_daily) = limit.max_per_day {
            let date_key = format!("{}:{}", ctx.user_id, chrono::Utc::now().format("%Y-%m-%d"));
            let daily_count = daily_counts
                .get(&date_key)
                .and_then(|m| m.get(&ctx.credential_type))
                .copied()
                .unwrap_or(0);

            if daily_count >= max_daily {
                return Err(ComplianceError::new(
                    "SAHI_2306",
                    format!("Daily issuance limit exceeded ({max_daily})"),
                    self.name(),
                )
                .with_details("Try again tomorrow"));
            }
        }

        Ok(ComplianceStatus::Passed)
    }

    fn applies_to(&self, _credential_type: CredentialType) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance::types::{BlacklistReason, OwnershipType, VerificationMethod};
    use chrono::Utc;

    #[test]
    fn identity_verification_check_passed() {
        let check = IdentityVerificationCheck::new();
        check.register(IdentityVerification {
            user_id: "USR_01HXK".to_string(),
            verified: true,
            method: VerificationMethod::MyDigitalId,
            verified_at: Some(Utc::now()),
            provider_ref: Some("MDI-12345".to_string()),
            expires_at: None,
        });

        let ctx = ComplianceContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            "USR_01HXK",
            CredentialType::ResidentBadge,
        );

        let result = check.check(&ctx);
        assert!(matches!(result, Ok(ComplianceStatus::Passed)));
    }

    #[test]
    fn identity_verification_check_missing() {
        let check = IdentityVerificationCheck::new();
        let ctx = ComplianceContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            "USR_01HXK",
            CredentialType::ResidentBadge,
        );

        let result = check.check(&ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().code.contains("2300"));
    }

    #[test]
    fn identity_verification_not_required_for_visitors() {
        let check = IdentityVerificationCheck::new();
        assert!(!check.applies_to(CredentialType::VisitorPass));
        assert!(check.applies_to(CredentialType::ResidentBadge));
    }

    #[test]
    fn unit_ownership_check_passed() {
        let check = UnitOwnershipCheck::new();
        check.register(UnitOwnership {
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

        let ctx = ComplianceContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            "USR_01HXK",
            CredentialType::ResidentBadge,
        )
        .with_unit("12-03");

        let result = check.check(&ctx);
        assert!(matches!(result, Ok(ComplianceStatus::Passed)));
    }

    #[test]
    fn unit_ownership_check_skipped_without_unit() {
        let check = UnitOwnershipCheck::new();
        let ctx = ComplianceContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            "USR_01HXK",
            CredentialType::ContractorBadge,
        );

        let result = check.check(&ctx);
        assert!(matches!(result, Ok(ComplianceStatus::Skipped)));
    }

    #[test]
    fn blacklist_check_passed() {
        let check = BlacklistCheck::new();
        let ctx = ComplianceContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            "USR_01HXK",
            CredentialType::ResidentBadge,
        );

        let result = check.check(&ctx);
        assert!(matches!(result, Ok(ComplianceStatus::Passed)));
    }

    #[test]
    fn blacklist_check_blocked() {
        let check = BlacklistCheck::new();
        check.add(BlacklistEntry {
            user_id: "USR_01HXK".to_string(),
            property_id: None,
            reason: BlacklistReason::SecurityViolation,
            description: "Tailgating incident".to_string(),
            created_at: Utc::now(),
            expires_at: None,
            created_by: "ADM_01HXK".to_string(),
        });

        let ctx = ComplianceContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            "USR_01HXK",
            CredentialType::ResidentBadge,
        );

        let result = check.check(&ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().code.contains("2304"));
    }

    #[test]
    fn blacklist_check_expired_entry_ignored() {
        let check = BlacklistCheck::new();
        check.add(BlacklistEntry {
            user_id: "USR_01HXK".to_string(),
            property_id: None,
            reason: BlacklistReason::Misconduct,
            description: "Old incident".to_string(),
            created_at: Utc::now() - chrono::Duration::days(365),
            expires_at: Some(Utc::now() - chrono::Duration::days(1)),
            created_by: "ADM_01HXK".to_string(),
        });

        let ctx = ComplianceContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            "USR_01HXK",
            CredentialType::ResidentBadge,
        );

        let result = check.check(&ctx);
        assert!(matches!(result, Ok(ComplianceStatus::Passed)));
    }

    #[test]
    fn credential_limit_check_passed() {
        let check = CredentialLimitCheck::new();
        let ctx = ComplianceContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            "USR_01HXK",
            CredentialType::ResidentBadge,
        );

        let result = check.check(&ctx);
        assert!(matches!(result, Ok(ComplianceStatus::Passed)));
    }

    #[test]
    fn credential_limit_check_exceeded() {
        let check = CredentialLimitCheck::new();

        // Issue 3 credentials (max for ResidentBadge)
        for _ in 0..3 {
            check.record_issuance("USR_01HXK", CredentialType::ResidentBadge);
        }

        let ctx = ComplianceContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            "USR_01HXK",
            CredentialType::ResidentBadge,
        );

        let result = check.check(&ctx);
        assert!(result.is_err());
        assert!(result.unwrap_err().code.contains("2305"));
    }

    #[test]
    fn credential_limit_respects_revocation() {
        let check = CredentialLimitCheck::new();

        // Issue 3 credentials
        for _ in 0..3 {
            check.record_issuance("USR_01HXK", CredentialType::ResidentBadge);
        }

        // Revoke one
        check.record_revocation("USR_01HXK", CredentialType::ResidentBadge);

        assert_eq!(
            check.get_active_count("USR_01HXK", CredentialType::ResidentBadge),
            2
        );

        let ctx = ComplianceContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            "USR_01HXK",
            CredentialType::ResidentBadge,
        );

        // Should pass now
        let result = check.check(&ctx);
        assert!(matches!(result, Ok(ComplianceStatus::Passed)));
    }
}
