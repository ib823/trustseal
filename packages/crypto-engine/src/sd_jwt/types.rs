//! SD-JWT types for VaultPass credentials.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// VaultPass credential claims following W3C VC Data Model 2.0.
///
/// Structure per MASTER_PLAN §2.4:
/// ```json
/// {
///   "@context": ["https://www.w3.org/ns/credentials/v2"],
///   "type": ["VerifiableCredential", "AccessBadge"],
///   "issuer": "did:web:issuer.sahi.my",
///   "validFrom": "2026-03-10T00:00:00Z",
///   "validUntil": "2027-03-10T00:00:00Z",
///   "credentialSubject": { ... }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultPassCredential {
    #[serde(rename = "@context")]
    pub context: Vec<String>,

    #[serde(rename = "type")]
    pub credential_type: Vec<String>,

    pub issuer: String,

    #[serde(rename = "validFrom")]
    pub valid_from: DateTime<Utc>,

    #[serde(rename = "validUntil", skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>,

    #[serde(rename = "credentialSubject")]
    pub credential_subject: CredentialSubject,

    #[serde(rename = "credentialStatus", skip_serializing_if = "Option::is_none")]
    pub credential_status: Option<CredentialStatus>,
}

impl VaultPassCredential {
    /// Create a new VaultPass credential with the standard context and type.
    pub fn new(
        issuer: String,
        subject: CredentialSubject,
        valid_from: DateTime<Utc>,
        valid_until: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            context: vec!["https://www.w3.org/ns/credentials/v2".to_string()],
            credential_type: vec![
                "VerifiableCredential".to_string(),
                "AccessBadge".to_string(),
            ],
            issuer,
            valid_from,
            valid_until,
            credential_subject: subject,
            credential_status: None,
        }
    }

    /// Add a credential status (Status List 2021) for revocation checking.
    #[must_use]
    pub fn with_status(mut self, status: CredentialStatus) -> Self {
        self.credential_status = Some(status);
        self
    }
}

/// VaultPass credential subject — the resident/visitor claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialSubject {
    /// DID of the credential holder (e.g., did:key:z6Mkh...)
    pub id: String,

    /// Property ID (e.g., PRY_01HXK...)
    #[serde(rename = "propertyId")]
    pub property_id: String,

    /// Unit/floor identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,

    /// Resident name (selectively disclosable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Role (resident, visitor, staff, contractor)
    pub role: String,

    /// Access zones (lobby, parking, gym, pool, etc.)
    #[serde(rename = "accessZones", skip_serializing_if = "Vec::is_empty", default)]
    pub access_zones: Vec<String>,

    /// Time-based access restrictions
    #[serde(rename = "timeRestrictions", skip_serializing_if = "Option::is_none")]
    pub time_restrictions: Option<TimeRestrictions>,
}

impl CredentialSubject {
    pub fn new(id: String, property_id: String, role: String) -> Self {
        Self {
            id,
            property_id,
            unit: None,
            name: None,
            role,
            access_zones: Vec::new(),
            time_restrictions: None,
        }
    }
}

/// Time-based access restrictions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRestrictions {
    /// Days of week (0=Sunday, 6=Saturday)
    #[serde(rename = "daysOfWeek", skip_serializing_if = "Vec::is_empty", default)]
    pub days_of_week: Vec<u8>,

    /// Start time (HH:MM in 24h format)
    #[serde(rename = "startTime", skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,

    /// End time (HH:MM in 24h format)
    #[serde(rename = "endTime", skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
}

/// Credential status for revocation checking (Status List 2021).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialStatus {
    pub id: String,

    #[serde(rename = "type")]
    pub status_type: String,

    #[serde(rename = "statusPurpose")]
    pub status_purpose: String,

    #[serde(rename = "statusListIndex")]
    pub status_list_index: String,

    #[serde(rename = "statusListCredential")]
    pub status_list_credential: String,
}

impl CredentialStatus {
    /// Create a new Status List 2021 status entry.
    pub fn new(index: u64, status_list_url: String) -> Self {
        Self {
            id: format!("{status_list_url}#{index}"),
            status_type: "StatusList2021Entry".to_string(),
            status_purpose: "revocation".to_string(),
            status_list_index: index.to_string(),
            status_list_credential: status_list_url,
        }
    }
}

/// Access badge claims for the simplified gate entry flow.
///
/// This is a flattened subset of `VaultPassCredential` optimized for
/// edge verifier processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessBadgeClaims {
    /// Credential holder DID
    pub sub: String,

    /// Property ID
    pub property_id: String,

    /// Unit identifier (if residential)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,

    /// Role
    pub role: String,

    /// Access zones
    #[serde(default)]
    pub zones: Vec<String>,

    /// Issued at (Unix timestamp)
    pub iat: i64,

    /// Expires at (Unix timestamp)
    pub exp: i64,
}

/// JSON Pointer path for selective disclosure.
///
/// Per MASTER_PLAN §2.4, selectively disclosable claims:
/// - `/credentialSubject/name` — resident name
/// - `/credentialSubject/unit` — unit identifier
/// - `/credentialSubject/accessZones` — individual zone entries
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimPath(pub String);

impl ClaimPath {
    pub const NAME: &'static str = "/credentialSubject/name";
    pub const UNIT: &'static str = "/credentialSubject/unit";
    pub const ACCESS_ZONES: &'static str = "/credentialSubject/accessZones";

    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ClaimPath {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Options for credential issuance.
#[derive(Debug, Clone)]
pub struct IssuanceOptions {
    /// Key handle for signing (from KMS)
    pub key_handle: String,

    /// Claims to make selectively disclosable (JSON Pointer paths)
    pub concealable_claims: Vec<ClaimPath>,

    /// Number of decoy digests to add (2-4 recommended)
    pub decoy_count: usize,

    /// Key binding: holder's public key (JWK format)
    pub holder_public_key: Option<Value>,
}

impl Default for IssuanceOptions {
    fn default() -> Self {
        Self {
            key_handle: String::new(),
            // No default concealable claims - let the caller specify based on actual credential content
            concealable_claims: Vec::new(),
            decoy_count: 3, // Middle of 2-4 range
            holder_public_key: None,
        }
    }
}

/// Options for deriving a presentation.
#[derive(Debug, Clone)]
pub struct PresentationOptions {
    /// Claims to disclose (JSON Pointer paths)
    pub disclosed_claims: Vec<ClaimPath>,

    /// Key binding JWT: audience (verifier DID or URL)
    pub audience: String,

    /// Key binding JWT: nonce from verifier challenge
    pub nonce: String,

    /// Key handle for holder's key binding signature
    pub holder_key_handle: String,
}

/// Result of verifying an SD-JWT presentation.
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether the signature is valid
    pub signature_valid: bool,

    /// Whether the credential is expired
    pub expired: bool,

    /// Disclosed claims (fully resolved JSON)
    pub disclosed_claims: Value,

    /// Issuer DID
    pub issuer: String,

    /// Holder DID (from key binding)
    pub holder: Option<String>,

    /// Key binding verified
    pub key_binding_valid: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn vaultpass_credential_serializes_correctly() {
        let subject = CredentialSubject {
            id: "did:key:z6Mkh123".to_string(),
            property_id: "PRY_01HXK4Y5J6P8M2N3Q7R9S0T1".to_string(),
            unit: Some("12-03".to_string()),
            name: Some("Ahmad bin Ali".to_string()),
            role: "resident".to_string(),
            access_zones: vec!["lobby".to_string(), "parking".to_string()],
            time_restrictions: None,
        };

        let now = Utc::now();
        let credential = VaultPassCredential::new(
            "did:web:issuer.sahi.my".to_string(),
            subject,
            now,
            Some(now + Duration::days(365)),
        );

        let json = serde_json::to_value(&credential).unwrap();
        assert_eq!(json["@context"][0], "https://www.w3.org/ns/credentials/v2");
        assert_eq!(json["type"][0], "VerifiableCredential");
        assert_eq!(json["type"][1], "AccessBadge");
        assert_eq!(json["issuer"], "did:web:issuer.sahi.my");
        assert_eq!(
            json["credentialSubject"]["propertyId"],
            "PRY_01HXK4Y5J6P8M2N3Q7R9S0T1"
        );
    }

    #[test]
    fn credential_status_formats_correctly() {
        let status = CredentialStatus::new(
            42,
            "https://status.sahi.my/list/v1/properties/123".to_string(),
        );

        assert_eq!(status.status_type, "StatusList2021Entry");
        assert_eq!(status.status_purpose, "revocation");
        assert_eq!(status.status_list_index, "42");
        assert!(status.id.contains("#42"));
    }

    #[test]
    fn claim_paths_are_valid_json_pointers() {
        assert!(ClaimPath::NAME.starts_with('/'));
        assert!(ClaimPath::UNIT.starts_with('/'));
        assert!(ClaimPath::ACCESS_ZONES.starts_with('/'));
    }
}
