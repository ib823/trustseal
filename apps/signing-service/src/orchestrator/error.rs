//! Orchestrator error types with SAHI error codes.

use thiserror::Error;

use crate::domain::StateTransitionError;

/// Orchestrator errors with SAHI error codes.
///
/// Signing Ceremony domain: `SAHI_3004`-3020
/// (`SAHI_3001`-3003 already used in sahi-core, `SAHI_3100`+ is `PAdES` domain)
#[derive(Debug, Error)]
pub enum OrchestratorError {
    /// `SAHI_3004`: Ceremony not found. Verify the ceremony ID and try again.
    #[error("Ceremony not found: {0}. Verify the ceremony ID is correct.")]
    CeremonyNotFound(String),

    /// `SAHI_3005`: Ceremony has expired. Create a new ceremony to continue.
    #[error("Ceremony has expired. Create a new ceremony to continue signing.")]
    CeremonyExpiredError,

    /// `SAHI_3006`: Invalid state transition. Check current state before transitioning.
    #[error("Invalid state transition: {0}. Check ceremony state before attempting.")]
    InvalidTransition(#[from] StateTransitionError),

    /// `SAHI_3007`: Signer not found in ceremony. Verify the signer ID.
    #[error("Signer not found: {0}. Verify the signer ID is correct.")]
    SignerNotFound(String),

    /// `SAHI_3008`: Signer cannot sign yet. Wait for their turn in sequential signing.
    #[error("Signer cannot sign: not their turn. Wait for previous signers to complete.")]
    SignerNotReady,

    /// `SAHI_3009`: Signer already signed. No action needed.
    #[error("Signer has already signed. No further action required.")]
    AlreadySigned,

    /// `SAHI_3010`: Document hash mismatch on resume. Upload the original document.
    #[error(
        "Document hash mismatch: expected {expected}, got {actual}. Use the original document."
    )]
    DocumentHashMismatch { expected: String, actual: String },

    /// `SAHI_3011`: Not all required signers have completed. Wait for pending signers.
    #[error("Not all required signers have completed. {pending} signers still pending.")]
    SignersIncomplete { pending: usize },

    /// `SAHI_3012`: `WebAuthn` verification failed. Re-authenticate with your passkey.
    #[error("WebAuthn verification failed: {0}. Re-authenticate with a valid passkey.")]
    WebAuthnFailed(String),

    /// `SAHI_3013`: Assurance level too low. Use a higher assurance authenticator.
    #[error("Assurance level {actual} is below required {required}. Use P2/P3 authenticator.")]
    AssuranceLevelTooLow { required: String, actual: String },

    /// `SAHI_3014`: Cannot abort ceremony in current state.
    #[error("Cannot abort ceremony: {0}")]
    AbortFailed(String),

    /// `SAHI_3015`: Cannot resume ceremony. Verify document hash matches.
    #[error("Cannot resume ceremony: {0}. Verify document hash matches original.")]
    ResumeFailed(String),

    /// `SAHI_3016`: Database error. Retry the operation or contact support.
    #[error("Database error: {0}. Retry or contact support.")]
    DatabaseError(String),

    /// `SAHI_3017`: Merkle log error. Audit trail may be inconsistent.
    #[error("Merkle log error: {0}. Contact support for audit verification.")]
    MerkleLogError(String),

    /// `SAHI_3018`: KMS error. Signing key unavailable.
    #[error("KMS error: {0}. Retry or contact support.")]
    KmsError(String),

    /// `SAHI_3019`: Document processing error. Verify document format.
    #[error("Document processing error: {0}. Verify PDF format is valid.")]
    DocumentError(String),

    /// `SAHI_3020`: Timestamp authority unreachable. Retry later.
    #[error("Timestamp authority error: {0}. Retry timestamping later.")]
    TimestampError(String),
}

impl OrchestratorError {
    /// Get the SAHI error code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::CeremonyNotFound(_) => "SAHI_3004",
            Self::CeremonyExpiredError => "SAHI_3005",
            Self::InvalidTransition(_) => "SAHI_3006",
            Self::SignerNotFound(_) => "SAHI_3007",
            Self::SignerNotReady => "SAHI_3008",
            Self::AlreadySigned => "SAHI_3009",
            Self::DocumentHashMismatch { .. } => "SAHI_3010",
            Self::SignersIncomplete { .. } => "SAHI_3011",
            Self::WebAuthnFailed(_) => "SAHI_3012",
            Self::AssuranceLevelTooLow { .. } => "SAHI_3013",
            Self::AbortFailed(_) => "SAHI_3014",
            Self::ResumeFailed(_) => "SAHI_3015",
            Self::DatabaseError(_) => "SAHI_3016",
            Self::MerkleLogError(_) => "SAHI_3017",
            Self::KmsError(_) => "SAHI_3018",
            Self::DocumentError(_) => "SAHI_3019",
            Self::TimestampError(_) => "SAHI_3020",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = OrchestratorError::CeremonyNotFound("CER_test".to_string());
        assert_eq!(err.code(), "SAHI_3004");

        let err = OrchestratorError::CeremonyExpiredError;
        assert_eq!(err.code(), "SAHI_3005");

        let err = OrchestratorError::SignersIncomplete { pending: 2 };
        assert_eq!(err.code(), "SAHI_3011");
    }

    #[test]
    fn test_error_display() {
        let err = OrchestratorError::DocumentHashMismatch {
            expected: "abc".to_string(),
            actual: "def".to_string(),
        };
        assert!(err.to_string().contains("abc"));
        assert!(err.to_string().contains("def"));
    }
}
