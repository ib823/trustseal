//! TM-1: Signing Ceremony Domain Types
//!
//! Core types for signing ceremonies including documents, configuration,
//! and metadata.

#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::trivially_copy_pass_by_ref)]

use serde::{Deserialize, Serialize};

use super::signer::SignerSlot;
use super::state_machine::CeremonyState;

/// Ceremony ID with CER_ prefix per ULID convention.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CeremonyId(pub String);

impl CeremonyId {
    /// Create a new ceremony ID (must start with CER_).
    pub fn new(id: impl Into<String>) -> Option<Self> {
        let id = id.into();
        if id.starts_with("CER_") && id.len() == 30 {
            Some(Self(id))
        } else {
            None
        }
    }

    /// Get the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CeremonyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type of signing ceremony.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CeremonyType {
    /// Single document, single signer
    SingleSigner,
    /// Single document, multiple signers (sequential order)
    MultiSignerSequential,
    /// Single document, multiple signers (parallel, any order)
    MultiSignerParallel,
    /// Multiple documents in a batch
    Batch,
}

impl CeremonyType {
    /// Database representation
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::SingleSigner => "single_signer",
            Self::MultiSignerSequential => "multi_signer_sequential",
            Self::MultiSignerParallel => "multi_signer_parallel",
            Self::Batch => "batch",
        }
    }

    /// Parse from database string.
    #[must_use]
    #[allow(clippy::match_same_arms)] // Wildcard fallback is intentional for safety
    pub fn from_str(s: &str) -> Self {
        match s {
            "single_signer" => Self::SingleSigner,
            "multi_signer_sequential" => Self::MultiSignerSequential,
            "multi_signer_parallel" => Self::MultiSignerParallel,
            "batch" => Self::Batch,
            _ => Self::SingleSigner, // DB constraints prevent this
        }
    }
}

/// Document to be signed in a ceremony.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CeremonyDocument {
    /// Document ID with DOC_ prefix
    pub id: String,
    /// Original filename
    pub filename: String,
    /// MIME type (application/pdf)
    pub content_type: String,
    /// SHA-256 hash of the document content (hex-encoded)
    pub content_hash: String,
    /// Size in bytes
    pub size_bytes: u64,
    /// S3 storage key
    pub storage_key: String,
    /// Signature field coordinates (optional, for visual signatures)
    pub signature_fields: Vec<SignatureField>,
}

/// Signature field coordinates in a PDF.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureField {
    /// Field name/ID
    pub name: String,
    /// Page number (1-indexed)
    pub page: u32,
    /// X coordinate (points from left)
    pub x: f32,
    /// Y coordinate (points from bottom)
    pub y: f32,
    /// Width (points)
    pub width: f32,
    /// Height (points)
    pub height: f32,
    /// Which signer slot this field is for
    pub signer_slot_index: usize,
}

/// Ceremony configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CeremonyConfig {
    /// Ceremony TTL in hours (default: 72)
    pub ttl_hours: u32,
    /// Require all signers or just minimum
    pub require_all_signers: bool,
    /// Minimum number of signers required (for threshold signing)
    pub min_signers: Option<u32>,
    /// Allow signers to decline
    pub allow_decline: bool,
    /// Send reminder emails
    pub send_reminders: bool,
    /// Reminder interval in hours
    pub reminder_interval_hours: Option<u32>,
    /// Require eKYC verification for signers
    pub require_ekyc: bool,
    /// Minimum assurance level (P1, P2, P3)
    pub min_assurance_level: String,
}

impl Default for CeremonyConfig {
    fn default() -> Self {
        Self {
            ttl_hours: 72,
            require_all_signers: true,
            min_signers: None,
            allow_decline: true,
            send_reminders: true,
            reminder_interval_hours: Some(24),
            require_ekyc: false,
            min_assurance_level: "P1".to_string(),
        }
    }
}

/// Ceremony metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CeremonyMetadata {
    /// Human-readable title
    pub title: String,
    /// Description
    pub description: Option<String>,
    /// Reference number (e.g., contract number)
    pub reference: Option<String>,
    /// Tags for organization
    pub tags: Vec<String>,
}

/// A signing ceremony.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ceremony {
    /// Unique ceremony ID
    pub id: CeremonyId,
    /// Tenant ID
    pub tenant_id: String,
    /// Creator user ID
    pub created_by: String,
    /// Current state
    pub state: CeremonyState,
    /// State before abort (for resume)
    pub state_before_abort: Option<CeremonyState>,
    /// Ceremony type
    pub ceremony_type: CeremonyType,
    /// Document to be signed
    pub document: CeremonyDocument,
    /// Signer slots
    pub signers: Vec<SignerSlot>,
    /// Configuration
    pub config: CeremonyConfig,
    /// Metadata
    pub metadata: CeremonyMetadata,
    /// Version for optimistic locking
    pub version: i64,
    /// Creation timestamp (RFC 3339)
    pub created_at: String,
    /// Last update timestamp (RFC 3339)
    pub updated_at: String,
    /// Expiry timestamp (RFC 3339)
    pub expires_at: String,
}

impl Ceremony {
    /// Check if the ceremony has expired.
    #[must_use]
    pub fn is_expired(&self, now: &str) -> bool {
        let expires_at = chrono::DateTime::parse_from_rfc3339(&self.expires_at)
            .map(|value| value.with_timezone(&chrono::Utc));
        let current = chrono::DateTime::parse_from_rfc3339(now)
            .map(|value| value.with_timezone(&chrono::Utc));

        match (expires_at, current) {
            (Ok(expires_at), Ok(current)) => expires_at <= current,
            _ => false,
        }
    }

    /// Check if all required signers have signed.
    #[must_use]
    pub fn all_required_signed(&self) -> bool {
        if self.config.require_all_signers {
            self.signers
                .iter()
                .filter(|s| s.is_required)
                .all(|s| s.status.is_signed())
        } else if let Some(min) = self.config.min_signers {
            let signed_count = self.signers.iter().filter(|s| s.status.is_signed()).count();
            signed_count >= min as usize
        } else {
            self.signers.iter().any(|s| s.status.is_signed())
        }
    }

    /// Get the number of signers who have signed.
    #[must_use]
    pub fn signed_count(&self) -> usize {
        self.signers.iter().filter(|s| s.status.is_signed()).count()
    }

    /// Get the number of signers who are pending.
    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.signers
            .iter()
            .filter(|s| !s.status.is_terminal())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ceremony_id_validation() {
        // Valid ceremony ID
        let valid = CeremonyId::new("CER_01HXK5M4N8JXJXJXJXJXJXJXJX");
        assert!(valid.is_some());

        // Invalid prefix
        let invalid_prefix = CeremonyId::new("DOC_01HXK5M4N8JXJXJXJXJXJXJXJX");
        assert!(invalid_prefix.is_none());

        // Too short
        let too_short = CeremonyId::new("CER_01HXK5");
        assert!(too_short.is_none());
    }

    #[test]
    fn test_ceremony_type_serialization() {
        let ct = CeremonyType::MultiSignerSequential;
        let json = serde_json::to_string(&ct).unwrap();
        assert_eq!(json, "\"multi_signer_sequential\"");
    }

    #[test]
    fn test_default_config() {
        let config = CeremonyConfig::default();
        assert_eq!(config.ttl_hours, 72);
        assert!(config.require_all_signers);
        assert!(config.allow_decline);
        assert_eq!(config.min_assurance_level, "P1");
    }
}
