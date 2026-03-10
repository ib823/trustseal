//! Cryptographic operations for VaultPass wallet.
//!
//! All crypto operations are delegated to crypto-engine.
//! This module provides the FFI interface for flutter_rust_bridge.

use std::collections::HashMap;

/// Create a Verifiable Presentation from an SD-JWT credential.
///
/// # Arguments
/// * `sd_jwt` - The SD-JWT credential string
/// * `nonce` - Challenge nonce from the verifier
/// * `disclosed_claims` - Claims to selectively disclose
///
/// # Returns
/// The VP as bytes (JSON-encoded for BLE).
pub fn create_presentation(
    _sd_jwt: String,
    _nonce: Vec<u8>,
    _disclosed_claims: Vec<String>,
) -> Result<Vec<u8>, String> {
    // TODO: Integrate with crypto-engine when flutter_rust_bridge is set up
    //
    // This would:
    // 1. Parse the SD-JWT
    // 2. Select disclosures for the requested claims
    // 3. Create a key binding JWT with the nonce
    // 4. Assemble the VP
    //
    // For now, return a placeholder error
    Err("FFI bridge not yet integrated with crypto-engine".to_string())
}

/// Sign data with the device key.
///
/// Note: The actual signing happens via platform channels (Android Keystore / iOS Secure Enclave).
/// This function prepares the data for signing.
pub fn prepare_for_signing(data: Vec<u8>) -> Result<Vec<u8>, String> {
    // Prepare data hash for signing
    // The actual signature is obtained via platform channels
    Ok(data)
}

/// Verify an SD-JWT credential locally.
///
/// Checks:
/// - Signature validity
/// - Expiration
///
/// Does NOT check:
/// - Revocation status (requires network/cache)
/// - Issuer DID resolution (requires network/cache)
pub fn verify_credential(_sd_jwt: String) -> Result<bool, String> {
    // TODO: Integrate with crypto-engine
    Err("FFI bridge not yet integrated with crypto-engine".to_string())
}

/// Parse an SD-JWT to extract claims.
///
/// Returns a map of claim names to values.
pub fn parse_credential(_sd_jwt: String) -> Result<HashMap<String, String>, String> {
    // TODO: Integrate with crypto-engine
    Err("FFI bridge not yet integrated with crypto-engine".to_string())
}

/// Generate a DID:key from a public key.
///
/// # Arguments
/// * `public_key` - The public key bytes (Ed25519 or P-256)
///
/// # Returns
/// The DID:key string.
pub fn generate_did_key(_public_key: Vec<u8>) -> Result<String, String> {
    // TODO: Integrate with crypto-engine did module
    Err("FFI bridge not yet integrated with crypto-engine".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_presentation_placeholder() {
        let result = create_presentation(
            "test.jwt".to_string(),
            vec![1, 2, 3, 4],
            vec!["name".to_string()],
        );
        assert!(result.is_err());
    }
}
