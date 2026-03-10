//! did:key method resolution.
//!
//! `did:key` is a purely cryptographic DID method where the DID encodes
//! a public key directly. No network resolution is required.
//!
//! Reference: <https://w3c-ccg.github.io/did-method-key/>

use serde_json::json;

use super::types::{
    DidContext, DidDocument, VerificationMethod, VerificationRelationship,
};

/// Multicodec prefixes for key types.
mod multicodec {
    /// Ed25519 public key (0xed01)
    pub const ED25519_PUB: [u8; 2] = [0xed, 0x01];
    /// X25519 public key (0xec01)
    pub const X25519_PUB: [u8; 2] = [0xec, 0x01];
    /// P-256 public key (0x8024)
    pub const P256_PUB: [u8; 2] = [0x80, 0x24];
    /// secp256k1 public key (0xe701)
    pub const SECP256K1_PUB: [u8; 2] = [0xe7, 0x01];
}

/// Key type detected from multicodec prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    /// Ed25519 signing key
    Ed25519,
    /// X25519 key agreement key
    X25519,
    /// P-256 (secp256r1) key
    P256,
    /// secp256k1 key
    Secp256k1,
}

impl KeyType {
    /// Returns the verification method type string.
    #[must_use]
    pub fn verification_method_type(self) -> &'static str {
        match self {
            Self::Ed25519 => "Ed25519VerificationKey2020",
            Self::X25519 => "X25519KeyAgreementKey2020",
            Self::P256 => "JsonWebKey2020",
            Self::Secp256k1 => "EcdsaSecp256k1VerificationKey2019",
        }
    }

    /// Returns the JWK curve name.
    #[must_use]
    pub fn jwk_curve(self) -> &'static str {
        match self {
            Self::Ed25519 => "Ed25519",
            Self::X25519 => "X25519",
            Self::P256 => "P-256",
            Self::Secp256k1 => "secp256k1",
        }
    }

    /// Returns the JWK key type.
    #[must_use]
    pub fn jwk_kty(self) -> &'static str {
        match self {
            Self::Ed25519 | Self::X25519 => "OKP",
            Self::P256 | Self::Secp256k1 => "EC",
        }
    }
}

/// Resolve a did:key to a DID Document.
///
/// # Arguments
/// * `did` - The full did:key string (e.g., `did:key:z6Mkh...`)
///
/// # Returns
/// A DID Document derived from the encoded public key.
///
/// # Errors
/// Returns an error if the DID format is invalid or the key type is unsupported.
pub fn resolve(did: &str) -> Result<DidDocument, String> {
    // Validate and extract the multibase-encoded key
    let key_part = did
        .strip_prefix("did:key:")
        .ok_or("Invalid did:key: must start with 'did:key:'")?;

    // Remove any fragment
    let key_part = key_part.split('#').next().unwrap_or(key_part);

    // Must start with 'z' (base58btc)
    if !key_part.starts_with('z') {
        return Err("Invalid did:key: multibase must start with 'z' (base58btc)".to_string());
    }

    // Decode the multibase key
    let decoded = bs58::decode(&key_part[1..])
        .into_vec()
        .map_err(|e| format!("Invalid base58 encoding: {e}"))?;

    if decoded.len() < 2 {
        return Err("Invalid did:key: decoded key too short".to_string());
    }

    // Detect key type from multicodec prefix
    let prefix = [decoded[0], decoded[1]];
    let raw_key = &decoded[2..];

    let (key_type, expected_len) = match prefix {
        multicodec::ED25519_PUB => (KeyType::Ed25519, 32),
        multicodec::X25519_PUB => (KeyType::X25519, 32),
        multicodec::P256_PUB => (KeyType::P256, 33), // Compressed point
        multicodec::SECP256K1_PUB => (KeyType::Secp256k1, 33), // Compressed point
        _ => return Err(format!("Unsupported multicodec prefix: {:02x}{:02x}", prefix[0], prefix[1])),
    };

    // Validate key length (P-256/secp256k1 can be 33 compressed or 65 uncompressed)
    let valid_len = match key_type {
        KeyType::Ed25519 | KeyType::X25519 => raw_key.len() == expected_len,
        KeyType::P256 | KeyType::Secp256k1 => raw_key.len() == 33 || raw_key.len() == 65,
    };

    if !valid_len {
        return Err(format!(
            "Invalid key length for {:?}: got {}, expected {}",
            key_type,
            raw_key.len(),
            expected_len
        ));
    }

    // Build the DID Document
    build_did_document(did, key_part, key_type, raw_key)
}

/// Build a DID Document from the decoded key.
fn build_did_document(
    did: &str,
    key_multibase: &str,
    key_type: KeyType,
    raw_key: &[u8],
) -> Result<DidDocument, String> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

    // The base DID (without fragment)
    let base_did = did.split('#').next().unwrap_or(did);
    let key_id = format!("{base_did}#{key_multibase}");

    // Build JWK
    let jwk = match key_type {
        KeyType::Ed25519 | KeyType::X25519 => {
            json!({
                "kty": key_type.jwk_kty(),
                "crv": key_type.jwk_curve(),
                "x": URL_SAFE_NO_PAD.encode(raw_key)
            })
        }
        KeyType::P256 | KeyType::Secp256k1 => {
            // For compressed points, we'd need to decompress
            // For uncompressed (65 bytes: 0x04 || x || y), extract x and y
            if raw_key.len() == 65 && raw_key[0] == 0x04 {
                let x = &raw_key[1..33];
                let y = &raw_key[33..65];
                json!({
                    "kty": "EC",
                    "crv": key_type.jwk_curve(),
                    "x": URL_SAFE_NO_PAD.encode(x),
                    "y": URL_SAFE_NO_PAD.encode(y)
                })
            } else if raw_key.len() == 33 {
                // Compressed point - store as multibase for now
                // Full decompression requires elliptic curve math
                json!({
                    "kty": "EC",
                    "crv": key_type.jwk_curve(),
                    "x": URL_SAFE_NO_PAD.encode(&raw_key[1..])  // Skip compression byte
                })
            } else {
                return Err("Invalid EC key format".to_string());
            }
        }
    };

    // Create verification method
    let verification_method = VerificationMethod {
        id: key_id.clone(),
        method_type: key_type.verification_method_type().to_string(),
        controller: base_did.to_string(),
        public_key_jwk: Some(jwk),
        public_key_multibase: Some(format!("z{key_multibase}")),
        public_key_base58: None,
    };

    // For Ed25519, also derive X25519 key agreement key
    let (verification_methods, key_agreement) = if key_type == KeyType::Ed25519 {
        // Derive X25519 from Ed25519 (not implemented here - would need curve conversion)
        // For now, just use the Ed25519 key
        (vec![verification_method.clone()], vec![])
    } else if key_type == KeyType::X25519 {
        (vec![verification_method.clone()], vec![VerificationRelationship::Reference(key_id.clone())])
    } else {
        (vec![verification_method.clone()], vec![])
    };

    // Build relationships (X25519 is for key agreement only, not signing)
    let authentication = if key_type == KeyType::X25519 {
        vec![]
    } else {
        vec![VerificationRelationship::Reference(key_id.clone())]
    };

    let assertion_method = if key_type == KeyType::X25519 {
        vec![]
    } else {
        vec![VerificationRelationship::Reference(key_id.clone())]
    };

    let capability_invocation = if key_type == KeyType::X25519 {
        vec![]
    } else {
        vec![VerificationRelationship::Reference(key_id.clone())]
    };

    let capability_delegation = if key_type == KeyType::X25519 {
        vec![]
    } else {
        vec![VerificationRelationship::Reference(key_id)]
    };

    Ok(DidDocument {
        context: DidContext::Multiple(vec![
            json!("https://www.w3.org/ns/did/v1"),
            json!("https://w3id.org/security/suites/ed25519-2020/v1"),
            json!("https://w3id.org/security/suites/x25519-2020/v1"),
        ]),
        id: base_did.to_string(),
        controller: None,
        verification_method: verification_methods,
        authentication,
        assertion_method,
        key_agreement,
        capability_invocation,
        capability_delegation,
        service: vec![],
        also_known_as: vec![],
    })
}

/// Generate a did:key from a raw Ed25519 public key.
///
/// # Arguments
/// * `public_key` - 32-byte Ed25519 public key
///
/// # Returns
/// The did:key string.
#[must_use]
pub fn from_ed25519_public_key(public_key: &[u8]) -> String {
    let mut multicodec = Vec::with_capacity(2 + public_key.len());
    multicodec.extend_from_slice(&multicodec::ED25519_PUB);
    multicodec.extend_from_slice(public_key);

    let encoded = bs58::encode(&multicodec).into_string();
    format!("did:key:z{encoded}")
}

/// Generate a did:key from raw P-256 public key coordinates.
///
/// # Arguments
/// * `x` - 32-byte X coordinate
/// * `y` - 32-byte Y coordinate
///
/// # Returns
/// The did:key string (using uncompressed point format).
#[must_use]
pub fn from_p256_public_key(x: &[u8], y: &[u8]) -> String {
    let mut multicodec = Vec::with_capacity(2 + 65);
    multicodec.extend_from_slice(&multicodec::P256_PUB);
    multicodec.push(0x04); // Uncompressed point prefix
    multicodec.extend_from_slice(x);
    multicodec.extend_from_slice(y);

    let encoded = bs58::encode(&multicodec).into_string();
    format!("did:key:z{encoded}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_ed25519_did_key() {
        // Test vector from did-key spec
        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        let doc = resolve(did).unwrap();

        assert_eq!(doc.id, did);
        assert_eq!(doc.verification_method.len(), 1);
        assert_eq!(
            doc.verification_method[0].method_type,
            "Ed25519VerificationKey2020"
        );
        assert!(doc.verification_method[0].public_key_jwk.is_some());
    }

    #[test]
    fn resolve_did_key_with_fragment() {
        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        let doc = resolve(did).unwrap();

        // Document ID should not have fragment
        assert!(!doc.id.contains('#'));
    }

    #[test]
    fn from_ed25519_creates_valid_did() {
        let public_key = [0u8; 32];
        let did = from_ed25519_public_key(&public_key);

        assert!(did.starts_with("did:key:z"));

        // Should be resolvable
        let doc = resolve(&did).unwrap();
        assert_eq!(doc.verification_method.len(), 1);
    }

    #[test]
    fn roundtrip_ed25519() {
        let public_key = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
            0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
        ];

        let did = from_ed25519_public_key(&public_key);
        let doc = resolve(&did).unwrap();

        // Extract public key from resolved doc
        let vm = &doc.verification_method[0];
        let extracted = vm.public_key_bytes().unwrap();

        assert_eq!(extracted, public_key);
    }

    #[test]
    fn invalid_did_key_rejected() {
        assert!(resolve("did:web:example.com").is_err());
        assert!(resolve("did:key:invalid").is_err());
        assert!(resolve("did:key:").is_err());
    }

    #[test]
    fn ed25519_has_correct_relationships() {
        let did = from_ed25519_public_key(&[0u8; 32]);
        let doc = resolve(&did).unwrap();

        // Ed25519 should have authentication, assertion, invocation, delegation
        assert!(!doc.authentication.is_empty());
        assert!(!doc.assertion_method.is_empty());
        assert!(!doc.capability_invocation.is_empty());
        assert!(!doc.capability_delegation.is_empty());

        // But no key agreement (X25519 would have that)
        assert!(doc.key_agreement.is_empty());
    }
}
