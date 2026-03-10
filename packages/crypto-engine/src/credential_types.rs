//! VaultPass credential type definitions.
//!
//! Per MASTER_PLAN §11.1 Credential Types:
//!
//! | Type              | TTL      | Selective Disclosure    |
//! |-------------------|----------|-------------------------|
//! | ResidentBadge     | 8 hours  | name, unit              |
//! | VisitorPass       | 4 hours  | name                    |
//! | ContractorBadge   | 12 hours | company                 |
//! | EmergencyAccess   | 1 hour   | none (all disclosed)    |

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::status_list::BitstringStatusListEntry;

/// Credential type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum CredentialType {
    /// Resident access badge (8-hour TTL).
    ResidentBadge,
    /// Visitor pass (4-hour TTL).
    VisitorPass,
    /// Contractor badge (12-hour TTL).
    ContractorBadge,
    /// Emergency access (1-hour TTL).
    EmergencyAccess,
}

impl CredentialType {
    /// Get the standard TTL for this credential type.
    #[must_use]
    pub fn ttl(self) -> Duration {
        match self {
            Self::ResidentBadge => Duration::hours(8),
            Self::VisitorPass => Duration::hours(4),
            Self::ContractorBadge => Duration::hours(12),
            Self::EmergencyAccess => Duration::hours(1),
        }
    }

    /// Get the refresh threshold (75% of TTL per spec).
    #[must_use]
    pub fn refresh_at(self) -> Duration {
        let ttl_secs = self.ttl().num_seconds();
        Duration::seconds(ttl_secs * 3 / 4) // 75%
    }

    /// Get the type string for the W3C VC type array.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ResidentBadge => "ResidentBadge",
            Self::VisitorPass => "VisitorPass",
            Self::ContractorBadge => "ContractorBadge",
            Self::EmergencyAccess => "EmergencyAccess",
        }
    }

    /// Get the selectively disclosable claim paths for this credential type.
    #[must_use]
    pub fn selective_claims(self) -> &'static [&'static str] {
        match self {
            Self::ResidentBadge => &["/credentialSubject/name", "/credentialSubject/unit"],
            Self::VisitorPass => &["/credentialSubject/name"],
            Self::ContractorBadge => &["/credentialSubject/company"],
            Self::EmergencyAccess => &[], // All disclosed
        }
    }
}

impl std::fmt::Display for CredentialType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for CredentialType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ResidentBadge" => Ok(Self::ResidentBadge),
            "VisitorPass" => Ok(Self::VisitorPass),
            "ContractorBadge" => Ok(Self::ContractorBadge),
            "EmergencyAccess" => Ok(Self::EmergencyAccess),
            _ => Err(format!("Unknown credential type: {s}")),
        }
    }
}

/// Resident badge credential (8-hour TTL).
///
/// Claims: name, unit, floor_access, role, clearance
/// Selective disclosure: name, unit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResidentBadgeSubject {
    /// Holder DID.
    pub id: String,

    /// Property ID (e.g., PRY_01HXK...).
    pub property_id: String,

    /// Resident name (selectively disclosable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Unit identifier, e.g., "12-03" (selectively disclosable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,

    /// Floors the resident can access.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub floor_access: Vec<String>,

    /// Role: owner, tenant, family_member.
    pub role: String,

    /// Security clearance level (0-3).
    #[serde(default)]
    pub clearance: u8,
}

/// Visitor pass credential (4-hour TTL).
///
/// Claims: name, host, purpose, valid_from, valid_until, floors
/// Selective disclosure: name
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisitorPassSubject {
    /// Holder DID.
    pub id: String,

    /// Property ID.
    pub property_id: String,

    /// Visitor name (selectively disclosable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Host resident's ID (who invited the visitor).
    pub host_id: String,

    /// Visit purpose: social, delivery, service, etc.
    pub purpose: String,

    /// Floors the visitor can access.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub floors: Vec<String>,

    /// Visit start time.
    pub valid_from: DateTime<Utc>,

    /// Visit end time.
    pub valid_until: DateTime<Utc>,
}

/// Contractor badge credential (12-hour TTL).
///
/// Claims: company, role, floors, valid_dates
/// Selective disclosure: company
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractorBadgeSubject {
    /// Holder DID.
    pub id: String,

    /// Property ID.
    pub property_id: String,

    /// Company name (selectively disclosable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,

    /// Contractor's role/trade: plumber, electrician, cleaner, etc.
    pub role: String,

    /// Floors the contractor can access.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub floors: Vec<String>,

    /// Work order or project ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_order_id: Option<String>,

    /// Contract start date.
    pub contract_start: DateTime<Utc>,

    /// Contract end date.
    pub contract_end: DateTime<Utc>,
}

/// Emergency access credential (1-hour TTL).
///
/// Claims: authority, scope, reason
/// Selective disclosure: none (all disclosed for accountability)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmergencyAccessSubject {
    /// Holder DID.
    pub id: String,

    /// Property ID (or "*" for all properties).
    pub property_id: String,

    /// Issuing authority: fire_dept, police, ambulance, building_management.
    pub authority: String,

    /// Scope of access: all, specific floors, specific zones.
    pub scope: EmergencyScope,

    /// Reason for emergency access.
    pub reason: String,

    /// Incident ID for audit trail.
    pub incident_id: String,
}

/// Scope of emergency access.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EmergencyScope {
    /// Full property access.
    All,
    /// Specific floors only.
    Floors(Vec<String>),
    /// Specific zones only.
    Zones(Vec<String>),
}

/// Generic VaultPass credential wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultPassCredentialV2 {
    /// JSON-LD context.
    #[serde(rename = "@context")]
    pub context: Vec<String>,

    /// Credential types.
    #[serde(rename = "type")]
    pub credential_type: Vec<String>,

    /// Credential ID (ULID prefixed).
    pub id: String,

    /// Issuer DID.
    pub issuer: String,

    /// Issue time.
    pub valid_from: DateTime<Utc>,

    /// Expiry time.
    pub valid_until: DateTime<Utc>,

    /// The credential subject (typed per credential type).
    pub credential_subject: serde_json::Value,

    /// Status list entry for revocation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_status: Option<BitstringStatusListEntry>,
}

impl VaultPassCredentialV2 {
    /// Create a new VaultPass credential.
    ///
    /// # Arguments
    /// * `id` - Credential ID (CRD_01HXK...)
    /// * `issuer` - Issuer DID
    /// * `cred_type` - Credential type
    /// * `subject` - Serialized credential subject
    pub fn new(
        id: String,
        issuer: String,
        cred_type: CredentialType,
        subject: serde_json::Value,
    ) -> Self {
        let now = Utc::now();
        let ttl = cred_type.ttl();

        Self {
            context: vec![
                "https://www.w3.org/ns/credentials/v2".to_string(),
                "https://sahi.my/ns/vaultpass/v1".to_string(),
            ],
            credential_type: vec![
                "VerifiableCredential".to_string(),
                cred_type.as_str().to_string(),
            ],
            id,
            issuer,
            valid_from: now,
            valid_until: now + ttl,
            credential_subject: subject,
            credential_status: None,
        }
    }

    /// Add status list entry for revocation.
    #[must_use]
    pub fn with_status(mut self, entry: BitstringStatusListEntry) -> Self {
        self.credential_status = Some(entry);
        self
    }

    /// Create a new VaultPass credential with custom validity period.
    pub fn with_validity(
        id: String,
        issuer: String,
        cred_type: CredentialType,
        subject: serde_json::Value,
        valid_from: DateTime<Utc>,
        valid_until: DateTime<Utc>,
    ) -> Self {
        Self {
            context: vec![
                "https://www.w3.org/ns/credentials/v2".to_string(),
                "https://sahi.my/ns/vaultpass/v1".to_string(),
            ],
            credential_type: vec![
                "VerifiableCredential".to_string(),
                cred_type.as_str().to_string(),
            ],
            id,
            issuer,
            valid_from,
            valid_until,
            credential_subject: subject,
            credential_status: None,
        }
    }

    /// Check if the credential is expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.valid_until < Utc::now()
    }

    /// Check if the credential should be refreshed (past 75% of TTL).
    #[must_use]
    pub fn should_refresh(&self) -> bool {
        let total_duration = self.valid_until - self.valid_from;
        let elapsed = Utc::now() - self.valid_from;
        elapsed > (total_duration * 3 / 4)
    }

    /// Get the credential type.
    #[must_use]
    pub fn get_type(&self) -> Option<CredentialType> {
        self.credential_type.get(1).and_then(|t| t.parse().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_type_ttls() {
        assert_eq!(CredentialType::ResidentBadge.ttl(), Duration::hours(8));
        assert_eq!(CredentialType::VisitorPass.ttl(), Duration::hours(4));
        assert_eq!(CredentialType::ContractorBadge.ttl(), Duration::hours(12));
        assert_eq!(CredentialType::EmergencyAccess.ttl(), Duration::hours(1));
    }

    #[test]
    fn credential_type_refresh_at() {
        // 75% of 8 hours = 6 hours
        assert_eq!(
            CredentialType::ResidentBadge.refresh_at(),
            Duration::hours(6)
        );
        // 75% of 4 hours = 3 hours
        assert_eq!(CredentialType::VisitorPass.refresh_at(), Duration::hours(3));
    }

    #[test]
    fn credential_type_selective_claims() {
        let resident_claims = CredentialType::ResidentBadge.selective_claims();
        assert!(resident_claims.contains(&"/credentialSubject/name"));
        assert!(resident_claims.contains(&"/credentialSubject/unit"));
        assert_eq!(resident_claims.len(), 2);

        let emergency_claims = CredentialType::EmergencyAccess.selective_claims();
        assert!(emergency_claims.is_empty()); // All disclosed
    }

    #[test]
    fn credential_type_parse_roundtrip() {
        for cred_type in [
            CredentialType::ResidentBadge,
            CredentialType::VisitorPass,
            CredentialType::ContractorBadge,
            CredentialType::EmergencyAccess,
        ] {
            let s = cred_type.to_string();
            let parsed: CredentialType = s.parse().unwrap();
            assert_eq!(parsed, cred_type);
        }
    }

    #[test]
    fn resident_badge_serializes_correctly() {
        let subject = ResidentBadgeSubject {
            id: "did:key:z6Mkh123".to_string(),
            property_id: "PRY_01HXK4Y5J6P8M2N3Q7R9S0T1".to_string(),
            name: Some("Ahmad bin Ali".to_string()),
            unit: Some("12-03".to_string()),
            floor_access: vec!["L1".to_string(), "L2".to_string(), "B1".to_string()],
            role: "owner".to_string(),
            clearance: 2,
        };

        let json = serde_json::to_value(&subject).unwrap();
        assert_eq!(json["id"], "did:key:z6Mkh123");
        assert_eq!(json["propertyId"], "PRY_01HXK4Y5J6P8M2N3Q7R9S0T1");
        assert_eq!(json["name"], "Ahmad bin Ali");
        assert_eq!(json["unit"], "12-03");
        assert_eq!(json["role"], "owner");
        assert_eq!(json["clearance"], 2);
    }

    #[test]
    fn visitor_pass_serializes_correctly() {
        let now = Utc::now();
        let subject = VisitorPassSubject {
            id: "did:key:z6Mkv456".to_string(),
            property_id: "PRY_01HXK4Y5J6P8M2N3Q7R9S0T1".to_string(),
            name: Some("Visitor Name".to_string()),
            host_id: "USR_01HXK4Y5J6P8M2N3Q7R9S0T1".to_string(),
            purpose: "delivery".to_string(),
            floors: vec!["L12".to_string()],
            valid_from: now,
            valid_until: now + Duration::hours(4),
        };

        let json = serde_json::to_value(&subject).unwrap();
        assert_eq!(json["purpose"], "delivery");
        assert_eq!(json["hostId"], "USR_01HXK4Y5J6P8M2N3Q7R9S0T1");
    }

    #[test]
    fn emergency_access_serializes_correctly() {
        let subject = EmergencyAccessSubject {
            id: "did:key:z6Mke789".to_string(),
            property_id: "*".to_string(), // All properties
            authority: "fire_dept".to_string(),
            scope: EmergencyScope::All,
            reason: "Fire emergency evacuation".to_string(),
            incident_id: "INC_01HXK4Y5J6P8M2N3Q7R9S0T1".to_string(),
        };

        let json = serde_json::to_value(&subject).unwrap();
        assert_eq!(json["authority"], "fire_dept");
        assert_eq!(json["propertyId"], "*");
        assert_eq!(json["scope"], "all");
    }

    #[test]
    fn vaultpass_credential_v2_expiry() {
        let subject = serde_json::json!({
            "id": "did:key:z6Mkh123",
            "propertyId": "PRY_01HXK"
        });

        let cred = VaultPassCredentialV2::new(
            "CRD_01HXK".to_string(),
            "did:web:issuer.sahi.my".to_string(),
            CredentialType::ResidentBadge,
            subject,
        );

        assert!(!cred.is_expired());
        assert!(!cred.should_refresh()); // Just issued

        // Check type extraction
        assert_eq!(cred.get_type(), Some(CredentialType::ResidentBadge));
    }

    #[test]
    fn vaultpass_credential_v2_context() {
        let subject = serde_json::json!({"id": "did:key:z6Mkh123"});

        let cred = VaultPassCredentialV2::new(
            "CRD_01HXK".to_string(),
            "did:web:issuer.sahi.my".to_string(),
            CredentialType::VisitorPass,
            subject,
        );

        let json = serde_json::to_value(&cred).unwrap();
        assert_eq!(json["@context"][0], "https://www.w3.org/ns/credentials/v2");
        assert_eq!(json["@context"][1], "https://sahi.my/ns/vaultpass/v1");
        assert_eq!(json["type"][0], "VerifiableCredential");
        assert_eq!(json["type"][1], "VisitorPass");
    }
}
