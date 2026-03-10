//! Status List Credential types.
//!
//! A Status List Credential is a W3C Verifiable Credential that contains
//! a compressed bitstring representing the revocation status of many credentials.
//!
//! Reference: W3C Bitstring Status List v1.0

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use super::bitstring::Bitstring;

/// A Bitstring Status List Credential.
///
/// This is a W3C Verifiable Credential that contains a compressed bitstring
/// where each bit represents the status of one credential.
///
/// # Example JSON
/// ```json
/// {
///   "@context": [
///     "https://www.w3.org/ns/credentials/v2",
///     "https://www.w3.org/ns/credentials/status-list/v1"
///   ],
///   "type": ["VerifiableCredential", "BitstringStatusListCredential"],
///   "issuer": "did:web:status.sahi.my",
///   "validFrom": "2026-03-10T00:00:00Z",
///   "credentialSubject": {
///     "id": "https://sahi.my/api/v1/status/TNT_01HXK/ResidentBadge#list",
///     "type": "BitstringStatusList",
///     "statusPurpose": "revocation",
///     "encodedList": "H4sIAAAAAAAAA..."
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusListCredential {
    /// JSON-LD context.
    #[serde(rename = "@context")]
    pub context: Vec<String>,

    /// Credential types.
    #[serde(rename = "type")]
    pub credential_type: Vec<String>,

    /// Status list credential ID (URL where this credential is published).
    pub id: String,

    /// Issuer DID.
    pub issuer: String,

    /// When this status list was issued.
    pub valid_from: DateTime<Utc>,

    /// When this status list expires (optional, typically short-lived).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>,

    /// The status list subject (contains the encoded bitstring).
    pub credential_subject: StatusListSubject,
}

/// The credential subject of a Status List Credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusListSubject {
    /// ID of this status list (typically same as credential ID with #list suffix).
    pub id: String,

    /// Type: "BitstringStatusList".
    #[serde(rename = "type")]
    pub subject_type: String,

    /// Purpose of this status list (e.g., "revocation", "suspension").
    pub status_purpose: String,

    /// The gzip-compressed, base64-encoded bitstring.
    pub encoded_list: String,
}

impl StatusListCredential {
    /// Create a new Status List Credential.
    ///
    /// # Arguments
    /// * `id` - URL where this credential will be published
    /// * `issuer` - Issuer DID (e.g., "did:web:status.sahi.my")
    /// * `bitstring` - The status bitstring
    /// * `purpose` - Status purpose (typically "revocation")
    /// * `ttl` - Time-to-live for this credential
    ///
    /// # Errors
    /// Returns an error if the bitstring cannot be encoded.
    pub fn new(
        id: &str,
        issuer: String,
        bitstring: &Bitstring,
        purpose: StatusPurpose,
        ttl: Duration,
    ) -> Result<Self, String> {
        let now = Utc::now();
        let encoded_list = bitstring.encode()?;

        Ok(Self {
            context: vec![
                "https://www.w3.org/ns/credentials/v2".to_string(),
                "https://www.w3.org/ns/credentials/status-list/v1".to_string(),
            ],
            credential_type: vec![
                "VerifiableCredential".to_string(),
                "BitstringStatusListCredential".to_string(),
            ],
            id: id.to_string(),
            issuer,
            valid_from: now,
            valid_until: Some(now + ttl),
            credential_subject: StatusListSubject {
                id: format!("{id}#list"),
                subject_type: "BitstringStatusList".to_string(),
                status_purpose: purpose.as_str().to_string(),
                encoded_list,
            },
        })
    }

    /// Create a revocation status list credential.
    ///
    /// # Errors
    /// Returns an error if the bitstring cannot be encoded.
    pub fn for_revocation(
        id: &str,
        issuer: String,
        bitstring: &Bitstring,
        ttl: Duration,
    ) -> Result<Self, String> {
        Self::new(id, issuer, bitstring, StatusPurpose::Revocation, ttl)
    }

    /// Extract and decode the bitstring from this credential.
    ///
    /// # Errors
    /// Returns an error if decoding fails.
    pub fn decode_bitstring(&self) -> Result<Bitstring, String> {
        Bitstring::decode(&self.credential_subject.encoded_list)
    }

    /// Check if a specific credential index is revoked.
    ///
    /// # Errors
    /// Returns an error if the bitstring cannot be decoded.
    pub fn is_revoked(&self, index: usize) -> Result<bool, String> {
        let bitstring = self.decode_bitstring()?;
        bitstring
            .is_revoked(index)
            .ok_or_else(|| format!("Index {index} out of bounds"))
    }

    /// Check if this status list credential has expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.valid_until.is_some_and(|exp| exp < Utc::now())
    }

    /// Get the status purpose.
    #[must_use]
    pub fn status_purpose(&self) -> &str {
        &self.credential_subject.status_purpose
    }
}

/// Purpose of a status list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusPurpose {
    /// Credential has been revoked (permanent).
    Revocation,
    /// Credential has been suspended (temporary).
    Suspension,
    /// Custom status message.
    Message,
}

impl StatusPurpose {
    /// Returns the string representation.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Revocation => "revocation",
            Self::Suspension => "suspension",
            Self::Message => "message",
        }
    }
}

impl std::str::FromStr for StatusPurpose {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "revocation" => Ok(Self::Revocation),
            "suspension" => Ok(Self::Suspension),
            "message" => Ok(Self::Message),
            _ => Err(format!("Unknown status purpose: {s}")),
        }
    }
}

/// Entry pointing to a status in a Status List Credential.
///
/// This is embedded in issued credentials to reference their status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BitstringStatusListEntry {
    /// Unique ID for this status entry.
    pub id: String,

    /// Type: "BitstringStatusListEntry".
    #[serde(rename = "type")]
    pub entry_type: String,

    /// Purpose (must match the status list's purpose).
    pub status_purpose: String,

    /// Index into the bitstring.
    pub status_list_index: String,

    /// URL of the Status List Credential.
    pub status_list_credential: String,
}

impl BitstringStatusListEntry {
    /// Create a new status list entry.
    ///
    /// # Arguments
    /// * `index` - The credential's index in the bitstring
    /// * `status_list_url` - URL of the Status List Credential
    /// * `purpose` - Status purpose (must match the status list)
    #[must_use]
    pub fn new(index: usize, status_list_url: String, purpose: StatusPurpose) -> Self {
        Self {
            id: format!("{status_list_url}#{index}"),
            entry_type: "BitstringStatusListEntry".to_string(),
            status_purpose: purpose.as_str().to_string(),
            status_list_index: index.to_string(),
            status_list_credential: status_list_url,
        }
    }

    /// Create a revocation status entry.
    #[must_use]
    pub fn for_revocation(index: usize, status_list_url: String) -> Self {
        Self::new(index, status_list_url, StatusPurpose::Revocation)
    }

    /// Parse the index from this entry.
    ///
    /// # Errors
    /// Returns an error if the index cannot be parsed.
    pub fn parse_index(&self) -> Result<usize, String> {
        self.status_list_index
            .parse()
            .map_err(|e| format!("Invalid status list index: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_status_list_credential() {
        let bitstring = Bitstring::new(1000);
        let credential = StatusListCredential::for_revocation(
            "https://sahi.my/api/v1/status/TNT_01HXK/ResidentBadge",
            "did:web:status.sahi.my".to_string(),
            &bitstring,
            Duration::minutes(15),
        )
        .unwrap();

        assert_eq!(credential.credential_type[1], "BitstringStatusListCredential");
        assert_eq!(credential.credential_subject.status_purpose, "revocation");
        assert!(credential.valid_until.is_some());
    }

    #[test]
    fn check_revocation_status() {
        let mut bitstring = Bitstring::new(1000);
        bitstring.revoke(42);
        bitstring.revoke(100);

        let credential = StatusListCredential::for_revocation(
            "https://sahi.my/status/test",
            "did:web:status.sahi.my".to_string(),
            &bitstring,
            Duration::minutes(15),
        )
        .unwrap();

        assert!(credential.is_revoked(42).unwrap());
        assert!(credential.is_revoked(100).unwrap());
        assert!(!credential.is_revoked(0).unwrap());
        assert!(!credential.is_revoked(50).unwrap());
    }

    #[test]
    fn status_list_entry_format() {
        let entry = BitstringStatusListEntry::for_revocation(
            42,
            "https://sahi.my/status/TNT_01HXK/ResidentBadge".to_string(),
        );

        assert_eq!(entry.entry_type, "BitstringStatusListEntry");
        assert_eq!(entry.status_purpose, "revocation");
        assert_eq!(entry.status_list_index, "42");
        assert!(entry.id.contains("#42"));
        assert_eq!(entry.parse_index().unwrap(), 42);
    }

    #[test]
    fn status_purpose_parsing() {
        assert_eq!(
            "revocation".parse::<StatusPurpose>().unwrap(),
            StatusPurpose::Revocation
        );
        assert_eq!(
            "SUSPENSION".parse::<StatusPurpose>().unwrap(),
            StatusPurpose::Suspension
        );
        assert!("invalid".parse::<StatusPurpose>().is_err());
    }

    #[test]
    fn credential_serializes_correctly() {
        let bitstring = Bitstring::new(100);
        let credential = StatusListCredential::for_revocation(
            "https://sahi.my/status/test",
            "did:web:status.sahi.my".to_string(),
            &bitstring,
            Duration::minutes(15),
        )
        .unwrap();

        let json = serde_json::to_value(&credential).unwrap();

        assert_eq!(json["@context"][0], "https://www.w3.org/ns/credentials/v2");
        assert_eq!(json["@context"][1], "https://www.w3.org/ns/credentials/status-list/v1");
        assert_eq!(json["type"][1], "BitstringStatusListCredential");
        assert_eq!(json["credentialSubject"]["type"], "BitstringStatusList");
        assert!(json["credentialSubject"]["encodedList"].is_string());
    }
}
