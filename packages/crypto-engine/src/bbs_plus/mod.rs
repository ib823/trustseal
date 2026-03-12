//! BBS+ Signatures for Unlinkable Presentations (Phase 2)
//!
//! This module provides the abstraction layer for BBS+ signatures using BLS12-381.
//! BBS+ enables unlinkable presentations where the same credential can be presented
//! multiple times without correlation.
//!
//! # Phase 1 Status
//!
//! This is a **stub implementation**. The actual BBS+ cryptography will be implemented
//! in Phase 2 using the `bbs` crate. For Phase 1, VaultPass uses SD-JWT with selective
//! disclosure.
//!
//! # Architecture
//!
//! The `CredentialProof` trait provides a common interface for both SD-JWT and BBS+:
//!
//! ```ignore
//! // Phase 1: SD-JWT
//! let proof = SdJwtProof::new(kms);
//! let secured = proof.sign(&credential, &issuer_key)?;
//!
//! // Phase 2: BBS+ (additive, same interface)
//! let proof = BbsPlusProof::new(kms);
//! let secured = proof.sign(&credential, &issuer_key)?;
//! ```
//!
//! # Migration Strategy
//!
//! - Dual issuance during transition: wallet stores both SD-JWT and BBS+ formats
//! - Verifier capability negotiation: `supported_cryptosuites: ["sd-jwt", "bbs-2023"]`
//! - No breaking changes to existing SD-JWT credentials

mod types;

pub use types::{
    BbsPlusError, BbsPlusKeyPair, BbsPlusPresentation, BbsPlusProof, BbsPlusSignature,
    CredentialClaims, CredentialProof, SecuredCredential, VerificationResult,
};

/// BBS+ is not yet implemented. This constant can be checked at runtime.
pub const BBS_PLUS_AVAILABLE: bool = false;

/// Supported cryptographic suites for credential proofs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoSuite {
    /// SD-JWT with selective disclosure (Phase 1, production-ready).
    SdJwt,
    /// BBS+ with unlinkable presentations (Phase 2, stub).
    Bbs2023,
}

impl CryptoSuite {
    /// Check if this suite is currently available.
    #[must_use]
    pub const fn is_available(self) -> bool {
        match self {
            Self::SdJwt => true,
            Self::Bbs2023 => BBS_PLUS_AVAILABLE,
        }
    }

    /// Get the suite identifier string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SdJwt => "sd-jwt",
            Self::Bbs2023 => "bbs-2023",
        }
    }
}

impl std::fmt::Display for CryptoSuite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sd_jwt_available() {
        assert!(CryptoSuite::SdJwt.is_available());
    }

    #[test]
    fn test_bbs_plus_not_available() {
        assert!(!CryptoSuite::Bbs2023.is_available());
        assert!(!BBS_PLUS_AVAILABLE);
    }

    #[test]
    fn test_suite_strings() {
        assert_eq!(CryptoSuite::SdJwt.as_str(), "sd-jwt");
        assert_eq!(CryptoSuite::Bbs2023.as_str(), "bbs-2023");
    }

    #[test]
    fn test_bbs_plus_proof_not_implemented() {
        let result = BbsPlusProof::new();
        assert!(result.is_err());
    }
}
