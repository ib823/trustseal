use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::Zeroize;

/// Supported key algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyAlgorithm {
    /// Ed25519 — SD-JWT issuer/holder signing, Merkle log signing
    Ed25519,
    /// ECDSA P-256 — PAdES document signing, COSE label tokens
    EcdsaP256,
}

impl fmt::Display for KeyAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ed25519 => write!(f, "Ed25519"),
            Self::EcdsaP256 => write!(f, "ECDSA-P256"),
        }
    }
}

/// Opaque handle to a key stored in the KMS.
/// Contains only the key ID — never the key material.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyHandle {
    /// ULID with KEY_ prefix
    pub id: String,
}

impl KeyHandle {
    #[must_use]
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl fmt::Display for KeyHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// Cryptographic signature bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub bytes: Vec<u8>,
    pub algorithm: KeyAlgorithm,
}

/// Public key bytes — safe to export and distribute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyBytes {
    pub bytes: Vec<u8>,
    pub algorithm: KeyAlgorithm,
}

/// Key metadata — returned by list_keys. Never contains private key material.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    pub handle: KeyHandle,
    pub algorithm: KeyAlgorithm,
    pub label: String,
    pub state: KeyState,
    pub created_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub tenant_id: Option<String>,
}

/// Key lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyState {
    /// Key can sign and verify
    Active,
    /// Key can only verify (post-rotation grace period)
    VerifyOnly,
    /// Key is scheduled for destruction
    PendingDestruction,
    /// Key has been destroyed (metadata retained for audit)
    Destroyed,
}

impl fmt::Display for KeyState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::VerifyOnly => write!(f, "verify_only"),
            Self::PendingDestruction => write!(f, "pending_destruction"),
            Self::Destroyed => write!(f, "destroyed"),
        }
    }
}

/// Result of a key rotation operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationResult {
    /// The new active key
    pub new_handle: KeyHandle,
    /// The old key (now in VerifyOnly state)
    pub old_handle: KeyHandle,
    /// When the old key will be fully retired
    pub old_key_valid_until: DateTime<Utc>,
}

/// Confirmation token for key destruction (dual-auth requirement).
#[derive(Debug, Clone, Zeroize)]
pub struct DestroyConfirmation {
    /// The key handle being destroyed (must match)
    pub key_id: String,
    /// Confirmation phrase: "DESTROY {key_id}"
    pub confirmation_phrase: String,
    /// Actor performing the destruction
    pub actor_id: String,
}

impl DestroyConfirmation {
    /// Validates that the confirmation matches the expected key.
    #[must_use]
    pub fn is_valid_for(&self, key_handle: &KeyHandle) -> bool {
        self.key_id == key_handle.id
            && self.confirmation_phrase == format!("DESTROY {}", key_handle.id)
    }
}
