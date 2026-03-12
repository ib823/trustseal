//! BBS+ type definitions and trait abstractions.
//!
//! This module defines the `CredentialProof` trait that abstracts over different
//! credential proof mechanisms (SD-JWT, BBS+, future schemes).

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Unified trait for credential proof generation and verification.
///
/// This trait enables the platform to support multiple proof formats
/// (SD-JWT in Phase 1, BBS+ in Phase 2) through a common interface.
///
/// # Implementation Notes
///
/// - Phase 1: Implement `SdJwtProof` in the `sd_jwt` module
/// - Phase 2: Implement `BbsPlusProof` in this module
/// - Verifiers should negotiate supported suites via capability exchange
pub trait CredentialProof: Send + Sync {
    /// Sign a credential with the given key.
    ///
    /// # Arguments
    ///
    /// * `credential` - The credential claims to sign
    /// * `issuer_did` - The issuer's DID for the credential
    /// * `disclosed_claims` - Which claims to disclose (for selective disclosure)
    ///
    /// # Errors
    ///
    /// Returns error if signing fails or key is invalid.
    fn sign(
        &self,
        credential: &CredentialClaims,
        issuer_did: &str,
        disclosed_claims: &HashSet<String>,
    ) -> Result<SecuredCredential, BbsPlusError>;

    /// Verify a secured credential.
    ///
    /// # Arguments
    ///
    /// * `secured` - The secured credential to verify
    ///
    /// # Errors
    ///
    /// Returns error if verification fails.
    fn verify(&self, secured: &SecuredCredential) -> Result<VerificationResult, BbsPlusError>;

    /// Create a derived presentation with selected claims.
    ///
    /// # Arguments
    ///
    /// * `secured` - The secured credential
    /// * `requested_claims` - Claims to include in presentation
    /// * `challenge` - Verifier's challenge nonce
    ///
    /// # Errors
    ///
    /// Returns error if presentation creation fails.
    fn derive_presentation(
        &self,
        secured: &SecuredCredential,
        requested_claims: &HashSet<String>,
        challenge: &str,
    ) -> Result<BbsPlusPresentation, BbsPlusError>;

    /// Verify a derived presentation.
    ///
    /// # Arguments
    ///
    /// * `presentation` - The presentation to verify
    /// * `issuer_public_key` - The issuer's public key bytes
    /// * `expected_nonce` - The nonce that was sent in the challenge
    ///
    /// # Errors
    ///
    /// Returns error if presentation verification fails.
    fn verify_presentation(
        &self,
        presentation: &BbsPlusPresentation,
        issuer_public_key: &[u8],
        expected_nonce: &str,
    ) -> Result<VerificationResult, BbsPlusError>;

    /// Get the cryptographic suite identifier.
    fn suite(&self) -> &'static str;
}

/// Credential claims to be signed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialClaims {
    /// Subject identifier (holder DID).
    pub subject_id: String,
    /// Claim key-value pairs.
    pub claims: std::collections::HashMap<String, serde_json::Value>,
    /// Credential type (e.g., "ResidentBadge", "VisitorPass").
    pub credential_type: String,
    /// Valid from timestamp (ISO 8601).
    pub valid_from: String,
    /// Valid until timestamp (ISO 8601).
    pub valid_until: String,
}

/// A cryptographically secured credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuredCredential {
    /// The cryptographic suite used.
    pub suite: String,
    /// The secured credential data (format depends on suite).
    pub data: String,
    /// Disclosed claim names.
    pub disclosed_claims: HashSet<String>,
}

/// Result of credential verification.
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether the credential is valid.
    pub valid: bool,
    /// Issuer DID.
    pub issuer: String,
    /// Subject DID.
    pub subject: String,
    /// Verified claims.
    pub claims: std::collections::HashMap<String, serde_json::Value>,
    /// Expiration timestamp.
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// BBS+ key pair (BLS12-381).
///
/// # Phase 2 Implementation
///
/// This will wrap the actual BBS+ key material from the `bbs` crate.
/// For now, it's a placeholder struct.
#[derive(Debug, Clone)]
pub struct BbsPlusKeyPair {
    /// Key identifier.
    pub key_id: String,
    /// Public key (placeholder - will be BLS12-381 G2 point).
    pub public_key: Vec<u8>,
    /// Private key (placeholder - will be BLS12-381 scalar).
    /// Note: In production, this would be in HSM.
    #[allow(dead_code)]
    secret_key: Option<Vec<u8>>,
}

impl BbsPlusKeyPair {
    /// Create a new key pair (stub - returns error until Phase 2).
    ///
    /// # Errors
    ///
    /// Always returns `NotImplemented` in Phase 1.
    pub fn generate(_key_id: &str) -> Result<Self, BbsPlusError> {
        Err(BbsPlusError::NotImplemented)
    }

    /// Get the public key bytes.
    #[must_use]
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }
}

/// BBS+ signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BbsPlusSignature {
    /// Signature bytes (BLS12-381 G1 point).
    pub signature: Vec<u8>,
    /// Signed message count.
    pub message_count: usize,
}

/// BBS+ derived presentation (zero-knowledge proof).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BbsPlusPresentation {
    /// The zero-knowledge proof.
    pub proof: Vec<u8>,
    /// Disclosed messages with indices.
    pub disclosed_messages: Vec<(usize, String)>,
    /// Challenge nonce used.
    pub challenge: String,
    /// Cryptographic suite.
    pub suite: String,
}

/// BBS+ proof implementation (stub).
///
/// This struct will implement the `CredentialProof` trait for BBS+ signatures
/// in Phase 2. For now, all methods return `NotImplemented`.
pub struct BbsPlusProof {
    _private: (),
}

impl BbsPlusProof {
    /// Create a new BBS+ proof handler (stub - returns error until Phase 2).
    ///
    /// # Errors
    ///
    /// Always returns `NotImplemented` in Phase 1.
    pub fn new() -> Result<Self, BbsPlusError> {
        Err(BbsPlusError::NotImplemented)
    }
}

impl CredentialProof for BbsPlusProof {
    fn sign(
        &self,
        _credential: &CredentialClaims,
        _issuer_did: &str,
        _disclosed_claims: &HashSet<String>,
    ) -> Result<SecuredCredential, BbsPlusError> {
        Err(BbsPlusError::NotImplemented)
    }

    fn verify(&self, _secured: &SecuredCredential) -> Result<VerificationResult, BbsPlusError> {
        Err(BbsPlusError::NotImplemented)
    }

    fn derive_presentation(
        &self,
        _secured: &SecuredCredential,
        _requested_claims: &HashSet<String>,
        _challenge: &str,
    ) -> Result<BbsPlusPresentation, BbsPlusError> {
        Err(BbsPlusError::NotImplemented)
    }

    fn verify_presentation(
        &self,
        _presentation: &BbsPlusPresentation,
        _issuer_public_key: &[u8],
        _expected_nonce: &str,
    ) -> Result<VerificationResult, BbsPlusError> {
        Err(BbsPlusError::NotImplemented)
    }

    fn suite(&self) -> &'static str {
        "bbs-2023"
    }
}

/// BBS+ errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum BbsPlusError {
    /// BBS+ is not implemented (Phase 2 feature).
    #[error("BBS+ signatures are not yet implemented (Phase 2 feature)")]
    NotImplemented,

    /// Key generation failed.
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),

    /// Signing failed.
    #[error("Signing failed: {0}")]
    SigningFailed(String),

    /// Verification failed.
    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    /// Presentation derivation failed.
    #[error("Presentation derivation failed: {0}")]
    PresentationFailed(String),

    /// Invalid key format.
    #[error("Invalid key format")]
    InvalidKeyFormat,

    /// Invalid signature format.
    #[error("Invalid signature format")]
    InvalidSignatureFormat,

    /// Claim not found for selective disclosure.
    #[error("Claim not found: {0}")]
    ClaimNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bbs_plus_key_generation_not_implemented() {
        let result = BbsPlusKeyPair::generate("test_key");
        assert!(matches!(result, Err(BbsPlusError::NotImplemented)));
    }

    #[test]
    fn test_bbs_plus_proof_not_implemented() {
        let result = BbsPlusProof::new();
        assert!(matches!(result, Err(BbsPlusError::NotImplemented)));
    }

    #[test]
    fn test_credential_claims_serialization() {
        let claims = CredentialClaims {
            subject_id: "did:key:z6Mk...".to_string(),
            claims: [("name".to_string(), serde_json::json!("Alice"))]
                .into_iter()
                .collect(),
            credential_type: "ResidentBadge".to_string(),
            valid_from: "2026-03-11T00:00:00Z".to_string(),
            valid_until: "2026-03-12T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&claims).expect("serialization should work");
        assert!(json.contains("ResidentBadge"));
    }

    #[test]
    fn test_secured_credential_serialization() {
        let secured = SecuredCredential {
            suite: "bbs-2023".to_string(),
            data: "eyJ...".to_string(),
            disclosed_claims: ["name".to_string()].into_iter().collect(),
        };

        let json = serde_json::to_string(&secured).expect("serialization should work");
        assert!(json.contains("bbs-2023"));
    }
}
