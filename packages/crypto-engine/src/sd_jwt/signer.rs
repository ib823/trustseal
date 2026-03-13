//! KMS-backed JWS signer for SD-JWT operations.

use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use sd_jwt_payload::{JsonObject, JwsSigner};
use serde_json::Value;

use crate::kms::{KeyAlgorithm, KeyHandle, KmsProvider};

/// JWS signer that delegates to `KmsProvider`.
///
/// This bridges the sd-jwt-payload crate with our KMS abstraction,
/// enabling HSM-backed signing in production.
pub struct KmsSigner {
    kms: Arc<dyn KmsProvider>,
    key_handle: KeyHandle,
    algorithm: SigningAlgorithm,
}

impl KmsSigner {
    /// Create a new KMS signer.
    ///
    /// # Arguments
    /// * `kms` - The KMS provider instance
    /// * `key_handle` - Handle to the signing key
    /// * `algorithm` - The signing algorithm (must match the key type)
    pub fn new(
        kms: Arc<dyn KmsProvider>,
        key_handle: KeyHandle,
        algorithm: SigningAlgorithm,
    ) -> Self {
        Self {
            kms,
            key_handle,
            algorithm,
        }
    }

    /// Create a signer for Ed25519 keys (used for SD-JWT issuer signatures).
    pub fn ed25519(kms: Arc<dyn KmsProvider>, key_handle: KeyHandle) -> Self {
        Self::new(kms, key_handle, SigningAlgorithm::EdDSA)
    }

    /// Create a signer for ECDSA P-256 keys (used for PAdES/COSE).
    pub fn es256(kms: Arc<dyn KmsProvider>, key_handle: KeyHandle) -> Self {
        Self::new(kms, key_handle, SigningAlgorithm::ES256)
    }
}

/// Supported signing algorithms for JWS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigningAlgorithm {
    /// EdDSA with Ed25519 (preferred for SD-JWT)
    EdDSA,
    /// ECDSA with P-256 (for compatibility)
    ES256,
}

impl SigningAlgorithm {
    /// Returns the JWS algorithm name (for JWT header).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EdDSA => "EdDSA",
            Self::ES256 => "ES256",
        }
    }

    /// Returns the corresponding KMS key algorithm.
    pub fn key_algorithm(self) -> KeyAlgorithm {
        match self {
            Self::EdDSA => KeyAlgorithm::Ed25519,
            Self::ES256 => KeyAlgorithm::EcdsaP256,
        }
    }

    /// Auto-detect the signing algorithm from a KMS key algorithm.
    pub fn from_key_algorithm(algorithm: KeyAlgorithm) -> Self {
        match algorithm {
            KeyAlgorithm::Ed25519 => Self::EdDSA,
            KeyAlgorithm::EcdsaP256 => Self::ES256,
        }
    }
}

impl fmt::Display for SigningAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Error type for KMS signing operations.
#[derive(Debug)]
pub struct KmsSignerError(pub String);

impl fmt::Display for KmsSignerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KMS signing error: {}", self.0)
    }
}

impl std::error::Error for KmsSignerError {}

#[async_trait]
impl JwsSigner for KmsSigner {
    type Error = KmsSignerError;

    /// Sign the JWT payload using the KMS.
    ///
    /// Creates a JWS with the format: `base64url(header).base64url(payload).base64url(signature)`
    async fn sign(
        &self,
        header: &JsonObject,
        payload: &JsonObject,
    ) -> Result<Vec<u8>, Self::Error> {
        // Ensure the algorithm in header matches our signer
        let mut header = header.clone();
        header.insert(
            "alg".to_string(),
            Value::String(self.algorithm.as_str().to_string()),
        );

        // Encode header and payload
        let header_json = serde_json::to_vec(&header)
            .map_err(|e| KmsSignerError(format!("Failed to serialize header: {e}")))?;
        let payload_json = serde_json::to_vec(&payload)
            .map_err(|e| KmsSignerError(format!("Failed to serialize payload: {e}")))?;

        let header_b64 = URL_SAFE_NO_PAD.encode(&header_json);
        let payload_b64 = URL_SAFE_NO_PAD.encode(&payload_json);

        // The signing input is: base64url(header).base64url(payload)
        let signing_input = format!("{header_b64}.{payload_b64}");

        // Sign using KMS
        let signature = self
            .kms
            .sign(&self.key_handle, signing_input.as_bytes())
            .await
            .map_err(|e| KmsSignerError(format!("KMS sign failed: {e}")))?;

        // Encode signature
        let signature_b64 = URL_SAFE_NO_PAD.encode(signature.as_bytes());

        // Return the complete JWS
        let jws = format!("{signing_input}.{signature_b64}");
        Ok(jws.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kms::{KmsProvider, SoftwareKmsProvider};

    #[tokio::test]
    async fn kms_signer_produces_valid_jws() {
        let kms: Arc<dyn KmsProvider> = Arc::new(SoftwareKmsProvider::new());
        let key_handle = kms
            .generate_key(KeyAlgorithm::Ed25519, "test-issuer", None)
            .await
            .unwrap();

        let signer = KmsSigner::ed25519(Arc::clone(&kms), key_handle);

        let mut header = JsonObject::new();
        header.insert("typ".to_string(), Value::String("sd-jwt".to_string()));

        let mut payload = JsonObject::new();
        payload.insert("sub".to_string(), Value::String("test-subject".to_string()));
        payload.insert("iss".to_string(), Value::String("test-issuer".to_string()));

        let jws_bytes = signer.sign(&header, &payload).await.unwrap();
        let jws = String::from_utf8(jws_bytes).unwrap();

        // JWS should have 3 parts separated by dots
        let parts: Vec<&str> = jws.split('.').collect();
        assert_eq!(parts.len(), 3, "JWS should have header.payload.signature");

        // Verify header contains our algorithm
        let decoded_header = URL_SAFE_NO_PAD.decode(parts[0]).unwrap();
        let header: JsonObject = serde_json::from_slice(&decoded_header).unwrap();
        assert_eq!(header.get("alg").unwrap(), "EdDSA");
    }

    #[tokio::test]
    async fn es256_signer_uses_correct_algorithm() {
        let kms: Arc<dyn KmsProvider> = Arc::new(SoftwareKmsProvider::new());
        let key_handle = kms
            .generate_key(KeyAlgorithm::EcdsaP256, "test-ecdsa", None)
            .await
            .unwrap();

        let signer = KmsSigner::es256(Arc::clone(&kms), key_handle);

        let header = JsonObject::new();
        let mut payload = JsonObject::new();
        payload.insert("test".to_string(), Value::Bool(true));

        let jws_bytes = signer.sign(&header, &payload).await.unwrap();
        let jws = String::from_utf8(jws_bytes).unwrap();

        let parts: Vec<&str> = jws.split('.').collect();
        let decoded_header = URL_SAFE_NO_PAD.decode(parts[0]).unwrap();
        let header: JsonObject = serde_json::from_slice(&decoded_header).unwrap();
        assert_eq!(header.get("alg").unwrap(), "ES256");
    }

    #[test]
    fn signing_algorithm_names_are_correct() {
        assert_eq!(SigningAlgorithm::EdDSA.as_str(), "EdDSA");
        assert_eq!(SigningAlgorithm::ES256.as_str(), "ES256");
    }
}
