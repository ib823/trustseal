//! PKCE (Proof Key for Code Exchange) helpers for OAuth 2.0.
//!
//! Implements RFC 7636 for secure authorization code flow.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ring::rand::{SecureRandom, SystemRandom};
use sha2::{Digest, Sha256};

/// PKCE parameters for OAuth 2.0 authorization.
#[derive(Debug, Clone)]
pub struct PkceParams {
    /// Code verifier: cryptographically random string (43-128 chars).
    pub code_verifier: String,
    /// Code challenge: SHA256 hash of verifier, Base64URL-encoded.
    pub code_challenge: String,
    /// Challenge method: always S256.
    pub code_challenge_method: &'static str,
    /// State: cryptographically random string for CSRF protection.
    pub state: String,
}

impl PkceParams {
    /// Generate new PKCE parameters.
    ///
    /// # Errors
    ///
    /// Returns error if random number generation fails.
    pub fn generate() -> Result<Self, PkceError> {
        let rng = SystemRandom::new();

        // Generate 32-byte random verifier (results in 43-char Base64URL string)
        let mut verifier_bytes = [0u8; 32];
        rng.fill(&mut verifier_bytes)
            .map_err(|_| PkceError::RandomGenerationFailed)?;
        let code_verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);

        // Generate code challenge: SHA256(verifier) -> Base64URL
        let code_challenge = Self::compute_challenge(&code_verifier);

        // Generate 32-byte random state
        let mut state_bytes = [0u8; 32];
        rng.fill(&mut state_bytes)
            .map_err(|_| PkceError::RandomGenerationFailed)?;
        let state = URL_SAFE_NO_PAD.encode(state_bytes);

        Ok(Self {
            code_verifier,
            code_challenge,
            code_challenge_method: "S256",
            state,
        })
    }

    /// Compute S256 challenge from verifier.
    pub fn compute_challenge(verifier: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        URL_SAFE_NO_PAD.encode(hash)
    }

    /// Verify that a code verifier matches a code challenge.
    pub fn verify(code_verifier: &str, code_challenge: &str) -> bool {
        let computed = Self::compute_challenge(code_verifier);
        // Constant-time comparison to prevent timing attacks
        constant_time_eq(computed.as_bytes(), code_challenge.as_bytes())
    }
}

/// Constant-time byte comparison.
///
/// Prevents timing attacks on security-sensitive comparisons (state, PKCE).
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

/// PKCE-related errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum PkceError {
    #[error("Failed to generate random bytes")]
    RandomGenerationFailed,
    #[error("Invalid code verifier")]
    InvalidCodeVerifier,
    #[error("Code challenge mismatch")]
    ChallengeMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_generation() {
        let params = PkceParams::generate().expect("PKCE generation should succeed");

        // Verifier should be 43 chars (32 bytes -> 43 Base64URL chars)
        assert_eq!(params.code_verifier.len(), 43);

        // Challenge should be 43 chars (32-byte SHA256 -> 43 Base64URL chars)
        assert_eq!(params.code_challenge.len(), 43);

        // State should be 43 chars
        assert_eq!(params.state.len(), 43);

        // Method should be S256
        assert_eq!(params.code_challenge_method, "S256");
    }

    #[test]
    fn test_pkce_verification() {
        let params = PkceParams::generate().expect("PKCE generation should succeed");

        // Verification should succeed with correct verifier
        assert!(PkceParams::verify(
            &params.code_verifier,
            &params.code_challenge
        ));

        // Verification should fail with wrong verifier
        assert!(!PkceParams::verify(
            "wrong_verifier",
            &params.code_challenge
        ));
    }

    #[test]
    fn test_pkce_uniqueness() {
        let params1 = PkceParams::generate().expect("PKCE generation should succeed");
        let params2 = PkceParams::generate().expect("PKCE generation should succeed");

        // Each generation should produce unique values
        assert_ne!(params1.code_verifier, params2.code_verifier);
        assert_ne!(params1.code_challenge, params2.code_challenge);
        assert_ne!(params1.state, params2.state);
    }

    #[test]
    fn test_challenge_computation() {
        // Known test vector
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let expected = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
        let computed = PkceParams::compute_challenge(verifier);
        assert_eq!(computed, expected);
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"hello", b"hell"));
    }
}
