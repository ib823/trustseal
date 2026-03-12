//! SD-JWT holder operations (derive presentations).

use std::sync::Arc;

use chrono::Utc;
use sd_jwt_payload::{KeyBindingJwt, SdJwt, Sha256Hasher};

use crate::error::CryptoError;
use crate::kms::{KeyAlgorithm, KeyHandle, KmsProvider};

use super::signer::{KmsSigner, SigningAlgorithm};
use super::types::PresentationOptions;

/// SD-JWT holder for deriving presentations.
///
/// The holder takes an issued SD-JWT and creates a presentation with:
/// 1. Selected disclosures (revealing only chosen claims)
/// 2. A Key Binding JWT proving possession of the holder's private key
///
/// # Example
///
/// ```ignore
/// let holder = SdJwtHolder::new(kms);
/// let presentation = holder.derive_presentation(
///     &sd_jwt,
///     PresentationOptions {
///         disclosed_claims: vec![ClaimPath::new("/credentialSubject/role")],
///         audience: "did:web:verifier.sahi.my".to_string(),
///         nonce: "abc123".to_string(),
///         holder_key_handle: "KEY_01HXK...".to_string(),
///     }
/// ).await?;
/// ```
pub struct SdJwtHolder {
    kms: Arc<dyn KmsProvider>,
}

impl SdJwtHolder {
    /// Create a new holder with the given KMS provider.
    pub fn new(kms: Arc<dyn KmsProvider>) -> Self {
        Self { kms }
    }

    /// Derive a presentation from an SD-JWT with selected disclosures.
    ///
    /// # Arguments
    /// * `sd_jwt` - The original SD-JWT issued to this holder
    /// * `options` - Presentation options (disclosed claims, audience, nonce, holder key)
    ///
    /// # Returns
    /// A new SD-JWT with:
    /// - Only the selected disclosures included
    /// - A Key Binding JWT attached
    ///
    /// # Errors
    /// Returns an error if the holder key cannot sign or disclosures cannot be filtered.
    pub async fn derive_presentation(
        &self,
        sd_jwt: &SdJwt,
        options: PresentationOptions,
    ) -> Result<SdJwt, CryptoError> {
        // Filter disclosures to only include those in disclosed_claims
        let filtered_disclosures: Vec<_> = sd_jwt
            .disclosures()
            .iter()
            .filter(|d| {
                // Check if this disclosure's claim name matches any in disclosed_claims
                options.disclosed_claims.iter().any(|path| {
                    // Simple matching: disclosure claim name is the last segment of the path
                    d.claim_name
                        .as_ref()
                        .is_some_and(|name| path.as_str().ends_with(&format!("/{name}")))
                })
            })
            .cloned()
            .collect();

        // Create the KB-JWT signer
        let key_handle = KeyHandle::new(options.holder_key_handle);
        let metadata = self.kms.get_key_metadata(&key_handle).await?;
        let signing_algorithm = match metadata.algorithm {
            KeyAlgorithm::Ed25519 => SigningAlgorithm::EdDSA,
            KeyAlgorithm::EcdsaP256 => SigningAlgorithm::ES256,
        };
        let signer = match signing_algorithm {
            SigningAlgorithm::EdDSA => KmsSigner::ed25519(Arc::clone(&self.kms), key_handle),
            SigningAlgorithm::ES256 => KmsSigner::es256(Arc::clone(&self.kms), key_handle),
        };

        // Build SD-JWT for KB-JWT (without existing KB-JWT)
        let mut sd_jwt_for_kb = rebuild_with_disclosures(sd_jwt.clone(), &filtered_disclosures);

        // Create the Key Binding JWT using the builder
        let hasher = Sha256Hasher::new();
        let kb_jwt = KeyBindingJwt::builder()
            .iat(Utc::now().timestamp())
            .aud(&options.audience)
            .nonce(&options.nonce)
            .finish(&sd_jwt_for_kb, &hasher, signing_algorithm.as_str(), &signer)
            .await
            .map_err(|e| CryptoError::Internal(format!("Failed to create KB-JWT: {e}")))?;

        // Attach the key binding JWT
        sd_jwt_for_kb.attach_key_binding_jwt(kb_jwt);

        Ok(sd_jwt_for_kb)
    }

    /// Derive a presentation and return the serialized string.
    ///
    /// This is a convenience method that returns the final presentation string
    /// ready for transmission to the verifier.
    ///
    /// # Errors
    /// Returns an error if presentation derivation fails.
    pub async fn derive_presentation_serialized(
        &self,
        sd_jwt: &SdJwt,
        options: PresentationOptions,
    ) -> Result<String, CryptoError> {
        let presentation = self.derive_presentation(sd_jwt, options).await?;
        Ok(presentation.presentation())
    }
}

/// Rebuild an SD-JWT with a filtered set of disclosures.
fn rebuild_with_disclosures(original: SdJwt, disclosures: &[sd_jwt_payload::Disclosure]) -> SdJwt {
    // Get the raw JWT string (first segment before ~)
    let presentation = original.presentation();
    let jwt_part = presentation.split('~').next().unwrap_or(&presentation);

    // Build new presentation string with filtered disclosures
    let disclosure_parts: String = disclosures
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join("~");

    let new_presentation = if disclosure_parts.is_empty() {
        format!("{jwt_part}~")
    } else {
        format!("{jwt_part}~{disclosure_parts}~")
    };

    // Parse and return (this should always succeed as we're just reorganizing)
    SdJwt::parse(&new_presentation).unwrap_or(original)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kms::{KeyAlgorithm, KmsProvider, SoftwareKmsProvider};
    use crate::sd_jwt::issuer::{HolderKeyBuilder, SdJwtIssuer};
    use crate::sd_jwt::types::{
        ClaimPath, CredentialSubject, IssuanceOptions, VaultPassCredential,
    };
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use chrono::Duration;

    #[tokio::test]
    async fn derive_presentation_with_selected_disclosures() {
        let kms: Arc<dyn KmsProvider> = Arc::new(SoftwareKmsProvider::new());

        // Generate issuer and holder keys
        let issuer_key = kms
            .generate_key(KeyAlgorithm::Ed25519, "issuer", None)
            .await
            .unwrap();
        let holder_key = kms
            .generate_key(KeyAlgorithm::Ed25519, "holder", None)
            .await
            .unwrap();
        let holder_public = kms.export_public_key(&holder_key).await.unwrap();
        let holder_jwk = HolderKeyBuilder::ed25519_jwk(holder_public.as_bytes());

        // Issue a credential with concealable claims
        let subject = CredentialSubject {
            id: "did:key:z6Mkh123".to_string(),
            property_id: "PRY_01HXK".to_string(),
            unit: Some("12-03".to_string()),
            name: Some("Test User".to_string()),
            role: "resident".to_string(),
            access_zones: vec!["lobby".to_string()],
            time_restrictions: None,
        };

        let credential = VaultPassCredential::new(
            "did:web:issuer.sahi.my".to_string(),
            subject,
            Utc::now(),
            Some(Utc::now() + Duration::days(365)),
        );

        let issuer = SdJwtIssuer::new(Arc::clone(&kms));
        let issue_options = IssuanceOptions {
            key_handle: issuer_key.id().to_string(),
            concealable_claims: vec![
                ClaimPath::new(ClaimPath::NAME),
                ClaimPath::new(ClaimPath::UNIT),
            ],
            decoy_count: 2,
            holder_public_key: Some(holder_jwk),
        };

        let sd_jwt = issuer.issue(&credential, issue_options).await.unwrap();

        // Derive presentation revealing only name (not unit)
        let holder = SdJwtHolder::new(Arc::clone(&kms));
        let present_options = PresentationOptions {
            disclosed_claims: vec![ClaimPath::new(ClaimPath::NAME)],
            audience: "did:web:verifier.sahi.my".to_string(),
            nonce: "challenge-nonce-123".to_string(),
            holder_key_handle: holder_key.id().to_string(),
        };

        let presentation = holder
            .derive_presentation(&sd_jwt, present_options)
            .await
            .unwrap();

        // Should have key binding JWT attached
        assert!(
            presentation.key_binding_jwt().is_some(),
            "Presentation should have KB-JWT"
        );
    }

    #[tokio::test]
    async fn derive_presentation_serialized_returns_string() {
        let kms: Arc<dyn KmsProvider> = Arc::new(SoftwareKmsProvider::new());
        let issuer_key = kms
            .generate_key(KeyAlgorithm::Ed25519, "issuer", None)
            .await
            .unwrap();
        let holder_key = kms
            .generate_key(KeyAlgorithm::Ed25519, "holder", None)
            .await
            .unwrap();
        let holder_public = kms.export_public_key(&holder_key).await.unwrap();
        let holder_jwk = HolderKeyBuilder::ed25519_jwk(holder_public.as_bytes());

        let subject = CredentialSubject::new(
            "did:key:z6Mkh123".to_string(),
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
        let sd_jwt = issuer
            .issue(
                &credential,
                IssuanceOptions {
                    key_handle: issuer_key.id().to_string(),
                    holder_public_key: Some(holder_jwk),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let holder = SdJwtHolder::new(Arc::clone(&kms));
        let presentation_str = holder
            .derive_presentation_serialized(
                &sd_jwt,
                PresentationOptions {
                    disclosed_claims: vec![],
                    audience: "aud".to_string(),
                    nonce: "nonce".to_string(),
                    holder_key_handle: holder_key.id().to_string(),
                },
            )
            .await
            .unwrap();

        // Should be a valid SD-JWT string with ~ separators
        assert!(presentation_str.contains('~'));
    }

    #[tokio::test]
    async fn derive_presentation_uses_es256_for_p256_holder_keys() {
        let kms: Arc<dyn KmsProvider> = Arc::new(SoftwareKmsProvider::new());
        let issuer_key = kms
            .generate_key(KeyAlgorithm::Ed25519, "issuer", None)
            .await
            .unwrap();
        let holder_key = kms
            .generate_key(KeyAlgorithm::EcdsaP256, "holder-p256", None)
            .await
            .unwrap();
        let holder_public = kms.export_public_key(&holder_key).await.unwrap();
        let holder_bytes = holder_public.as_bytes();
        let holder_jwk = HolderKeyBuilder::es256_jwk(&holder_bytes[1..33], &holder_bytes[33..65]);

        let subject = CredentialSubject::new(
            "did:key:z6Mkh123".to_string(),
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
        let sd_jwt = issuer
            .issue(
                &credential,
                IssuanceOptions {
                    key_handle: issuer_key.id().to_string(),
                    holder_public_key: Some(holder_jwk),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let holder = SdJwtHolder::new(Arc::clone(&kms));
        let presentation = holder
            .derive_presentation(
                &sd_jwt,
                PresentationOptions {
                    disclosed_claims: vec![],
                    audience: "aud".to_string(),
                    nonce: "nonce".to_string(),
                    holder_key_handle: holder_key.id().to_string(),
                },
            )
            .await
            .unwrap();

        let kb_jwt = presentation.key_binding_jwt().unwrap().to_string();
        let header = kb_jwt.split('.').next().unwrap();
        let header_json: serde_json::Value =
            serde_json::from_slice(&URL_SAFE_NO_PAD.decode(header).unwrap()).unwrap();
        assert_eq!(header_json["alg"], "ES256");
    }
}
