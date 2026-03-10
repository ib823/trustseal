//! SD-JWT credential operations for VaultPass (VP-1).
//!
//! This module provides:
//! - `SdJwtIssuer`: Create SD-JWT credentials with selective disclosure
//! - `SdJwtHolder`: Derive presentations with selected claims
//! - `SdJwtVerifier`: Verify presentations and extract claims
//!
//! # Architecture
//!
//! All signing operations delegate to `KmsProvider`, ensuring HSM compatibility.
//! The `JwsSigner` trait adapter (`KmsSigner`) bridges sd-jwt-payload with our KMS.
//!
//! # Performance Target
//!
//! SD-JWT verification: < 50ms (crypto only, no network)

mod holder;
mod issuer;
mod signer;
mod types;
mod verifier;

pub use holder::SdJwtHolder;
pub use issuer::{HolderKeyBuilder, SdJwtIssuer};
pub use signer::KmsSigner;
pub use types::{
    AccessBadgeClaims, ClaimPath, CredentialSubject, IssuanceOptions, PresentationOptions,
    VaultPassCredential, VerificationResult,
};
pub use verifier::SdJwtVerifier;

// Re-export key types from sd-jwt-payload for convenience
pub use sd_jwt_payload::{Disclosure, KeyBindingJwtClaims, SdJwt, SdJwtClaims};
