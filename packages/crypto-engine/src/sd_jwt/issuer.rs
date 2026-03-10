//! SD-JWT credential issuance.

use std::sync::Arc;

use sd_jwt_payload::{RequiredKeyBinding, SdJwt, SdJwtBuilder};
use serde::Serialize;
use serde_json::Value;

use crate::error::CryptoError;
use crate::kms::{KeyHandle, KmsProvider};

use super::signer::{KmsSigner, SigningAlgorithm};
use super::types::{IssuanceOptions, VaultPassCredential};

/// SD-JWT credential issuer.
///
/// Creates SD-JWT credentials with selective disclosure, backed by KMS signing.
///
/// # Example
///
/// ```ignore
/// let issuer = SdJwtIssuer::new(kms);
/// let credential = VaultPassCredential::new(...);
/// let options = IssuanceOptions {
///     key_handle: "KEY_01HXK...".to_string(),
///     concealable_claims: vec![ClaimPath::NAME, ClaimPath::UNIT],
///     decoy_count: 3,
///     holder_public_key: Some(holder_jwk),
/// };
/// let sd_jwt = issuer.issue(&credential, options).await?;
/// ```
pub struct SdJwtIssuer {
    kms: Arc<dyn KmsProvider>,
}

impl SdJwtIssuer {
    /// Create a new issuer with the given KMS provider.
    pub fn new(kms: Arc<dyn KmsProvider>) -> Self {
        Self { kms }
    }

    /// Issue an SD-JWT credential.
    ///
    /// # Arguments
    /// * `credential` - The VaultPass credential to issue
    /// * `options` - Issuance options (key, concealable claims, decoys, holder key)
    ///
    /// # Returns
    /// The signed SD-JWT with disclosures.
    ///
    /// # Errors
    /// Returns an error if signing fails or the credential structure is invalid.
    pub async fn issue(
        &self,
        credential: &VaultPassCredential,
        options: IssuanceOptions,
    ) -> Result<SdJwt, CryptoError> {
        self.issue_generic(credential, options).await
    }

    /// Issue an SD-JWT from any serializable object.
    ///
    /// This is the generic version that accepts any type implementing `Serialize`.
    ///
    /// # Errors
    /// Returns an error if signing fails or the object structure is invalid.
    pub async fn issue_generic<T: Serialize>(
        &self,
        claims: &T,
        options: IssuanceOptions,
    ) -> Result<SdJwt, CryptoError> {
        // Build the SD-JWT
        let mut builder = SdJwtBuilder::new(claims)
            .map_err(|e| CryptoError::Internal(format!("Failed to create SD-JWT builder: {e}")))?;

        // Make specified claims concealable (selective disclosure)
        for path in &options.concealable_claims {
            builder = builder.make_concealable(path.as_str()).map_err(|e| {
                CryptoError::Internal(format!(
                    "Failed to make claim concealable at {}: {e}",
                    path.as_str()
                ))
            })?;
        }

        // Add decoy digests (prevents claim-count inference)
        if options.decoy_count > 0 {
            builder = builder
                .add_decoys("", options.decoy_count)
                .map_err(|e| CryptoError::Internal(format!("Failed to add decoys: {e}")))?;
        }

        // Add key binding if holder public key provided
        if let Some(ref holder_key) = options.holder_public_key {
            // Convert Value to JsonObject (Map<String, Value>)
            let jwk_object = holder_key.as_object().cloned().unwrap_or_default();
            let key_binding = RequiredKeyBinding::Jwk(jwk_object);
            builder = builder.require_key_binding(key_binding);
        }

        // Create the signer
        let key_handle = KeyHandle::new(options.key_handle);
        let signer = KmsSigner::ed25519(Arc::clone(&self.kms), key_handle);

        // Sign and finalize
        let sd_jwt = builder
            .finish(&signer, SigningAlgorithm::EdDSA.as_str())
            .await
            .map_err(|e| CryptoError::Internal(format!("Failed to sign SD-JWT: {e}")))?;

        Ok(sd_jwt)
    }

    /// Issue a credential and return the serialized SD-JWT string.
    ///
    /// This is a convenience method that returns the final presentation string
    /// ready for transmission.
    ///
    /// # Errors
    /// Returns an error if credential issuance fails.
    pub async fn issue_serialized(
        &self,
        credential: &VaultPassCredential,
        options: IssuanceOptions,
    ) -> Result<String, CryptoError> {
        let sd_jwt = self.issue(credential, options).await?;
        Ok(sd_jwt.presentation())
    }
}

/// Builder for creating holder public key JWK for key binding.
pub struct HolderKeyBuilder;

impl HolderKeyBuilder {
    /// Create an Ed25519 JWK from a raw public key.
    ///
    /// The public key should be 32 bytes (Ed25519 public key).
    pub fn ed25519_jwk(public_key: &[u8]) -> Value {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
        serde_json::json!({
            "kty": "OKP",
            "crv": "Ed25519",
            "x": URL_SAFE_NO_PAD.encode(public_key)
        })
    }

    /// Create an ES256 (P-256) JWK from raw public key coordinates.
    ///
    /// The x and y coordinates should each be 32 bytes.
    pub fn es256_jwk(x: &[u8], y: &[u8]) -> Value {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
        serde_json::json!({
            "kty": "EC",
            "crv": "P-256",
            "x": URL_SAFE_NO_PAD.encode(x),
            "y": URL_SAFE_NO_PAD.encode(y)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kms::{KeyAlgorithm, KmsProvider, SoftwareKmsProvider};
    use crate::sd_jwt::types::{ClaimPath, CredentialSubject};
    use chrono::{Duration, Utc};

    #[tokio::test]
    async fn issue_vaultpass_credential() {
        let kms: Arc<dyn KmsProvider> = Arc::new(SoftwareKmsProvider::new());

        // Generate issuer signing key
        let issuer_key = kms
            .generate_key(KeyAlgorithm::Ed25519, "issuer-key", Some("TNT_test"))
            .await
            .unwrap();

        // Generate holder binding key
        let holder_key = kms
            .generate_key(KeyAlgorithm::Ed25519, "holder-key", None)
            .await
            .unwrap();
        let holder_public = kms.export_public_key(&holder_key).await.unwrap();
        let holder_jwk = HolderKeyBuilder::ed25519_jwk(holder_public.as_bytes());

        // Create credential
        let subject = CredentialSubject {
            id: "did:key:z6Mkh123".to_string(),
            property_id: "PRY_01HXK4Y5J6P8M2N3Q7R9S0T1".to_string(),
            unit: Some("12-03".to_string()),
            name: Some("Ahmad bin Ali".to_string()),
            role: "resident".to_string(),
            access_zones: vec!["lobby".to_string(), "parking".to_string()],
            time_restrictions: None,
        };

        let now = Utc::now();
        let credential = VaultPassCredential::new(
            "did:web:issuer.sahi.my".to_string(),
            subject,
            now,
            Some(now + Duration::days(365)),
        );

        // Issue SD-JWT
        let issuer = SdJwtIssuer::new(Arc::clone(&kms));
        let options = IssuanceOptions {
            key_handle: issuer_key.id().to_string(),
            concealable_claims: vec![ClaimPath::new(ClaimPath::NAME)],
            decoy_count: 2,
            holder_public_key: Some(holder_jwk),
        };

        let sd_jwt = issuer.issue(&credential, options).await.unwrap();

        // Verify structure
        let presentation = sd_jwt.presentation();
        assert!(
            presentation.contains('~'),
            "SD-JWT should contain ~ separators"
        );

        // Should have at least one disclosure (for name)
        assert!(
            !sd_jwt.disclosures().is_empty(),
            "SD-JWT should have at least one disclosure"
        );

        // Should have key binding requirement
        assert!(
            sd_jwt.required_key_bind().is_some(),
            "SD-JWT should require key binding"
        );
    }

    #[tokio::test]
    async fn issue_with_multiple_concealable_claims() {
        let kms: Arc<dyn KmsProvider> = Arc::new(SoftwareKmsProvider::new());
        let issuer_key = kms
            .generate_key(KeyAlgorithm::Ed25519, "issuer-key", None)
            .await
            .unwrap();

        let subject = CredentialSubject::new(
            "did:key:z6Mkh456".to_string(),
            "PRY_01HXK".to_string(),
            "visitor".to_string(),
        );

        let credential = VaultPassCredential::new(
            "did:web:issuer.sahi.my".to_string(),
            subject,
            Utc::now(),
            None,
        );

        let issuer = SdJwtIssuer::new(Arc::clone(&kms));
        let options = IssuanceOptions {
            key_handle: issuer_key.id().to_string(),
            concealable_claims: vec![
                ClaimPath::new("/credentialSubject/role"),
                ClaimPath::new("/credentialSubject/propertyId"),
            ],
            decoy_count: 3,
            holder_public_key: None,
        };

        let sd_jwt = issuer.issue(&credential, options).await.unwrap();

        // Should have 2 disclosures (role + propertyId)
        assert_eq!(
            sd_jwt.disclosures().len(),
            2,
            "Should have 2 disclosures for concealable claims"
        );
    }

    #[test]
    fn holder_key_builder_creates_valid_jwk() {
        let fake_key = [0u8; 32];
        let jwk = HolderKeyBuilder::ed25519_jwk(&fake_key);

        assert_eq!(jwk["kty"], "OKP");
        assert_eq!(jwk["crv"], "Ed25519");
        assert!(jwk["x"].is_string());
    }
}
