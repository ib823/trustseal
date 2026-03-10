use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::kms::Signature;

/// The Trust Registry answers: "Is this DID authorized to issue credentials of type X?"
///
/// Published as a signed JSON document at `/.well-known/trust-registry.json`.
/// Edge verifiers cache locally with 1-4 hour TTL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustRegistry {
    /// Monotonically increasing version number
    pub version: u64,
    /// When this version was published
    pub published: DateTime<Utc>,
    /// The governance DID that signs the registry
    pub governance_did: String,
    /// All trusted issuers and their permissions
    pub entries: Vec<TrustEntry>,
    /// Ed25519 signature over the canonical JSON (excluding this field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<Signature>,
}

/// A single entry in the Trust Registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEntry {
    /// The issuer's DID (e.g., "did:web:issuer.property.sahi.my")
    pub issuer_did: String,
    /// Credential types this issuer is authorized to issue
    pub credential_types: Vec<String>,
    /// When this entry becomes valid
    pub valid_from: DateTime<Utc>,
    /// When this entry expires (None = no expiry)
    pub valid_until: Option<DateTime<Utc>>,
    /// Current trust status
    pub status: TrustStatus,
    /// Which verifier DIDs trust this issuer
    pub verifier_dids: Vec<String>,
    /// Property IDs this issuer serves
    pub properties: Vec<String>,
}

/// Trust status for a registry entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustStatus {
    Active,
    Suspended,
    Revoked,
}

impl TrustRegistry {
    /// Create a new empty registry.
    pub fn new(governance_did: &str) -> Self {
        Self {
            version: 1,
            published: Utc::now(),
            governance_did: governance_did.to_string(),
            entries: Vec::new(),
            signature: None,
        }
    }

    /// Check if a given issuer DID is authorized for a specific credential type.
    pub fn is_authorized(&self, issuer_did: &str, credential_type: &str) -> bool {
        self.entries.iter().any(|entry| {
            entry.issuer_did == issuer_did
                && entry.status == TrustStatus::Active
                && entry.credential_types.iter().any(|ct| ct == credential_type)
                && entry.valid_from <= Utc::now()
                && entry
                    .valid_until
                    .is_none_or(|until| until > Utc::now())
        })
    }

    /// Add a trusted issuer entry.
    pub fn add_entry(&mut self, entry: TrustEntry) {
        self.entries.push(entry);
        self.version += 1;
        self.published = Utc::now();
    }

    /// Revoke an issuer by DID.
    pub fn revoke_issuer(&mut self, issuer_did: &str) {
        for entry in &mut self.entries {
            if entry.issuer_did == issuer_did {
                entry.status = TrustStatus::Revoked;
            }
        }
        self.version += 1;
        self.published = Utc::now();
    }
}

impl std::fmt::Display for TrustStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Suspended => write!(f, "suspended"),
            Self::Revoked => write!(f, "revoked"),
        }
    }
}
