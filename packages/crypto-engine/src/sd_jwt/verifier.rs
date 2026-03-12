//! SD-JWT verification.

use std::time::Instant;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::Utc;
use ring::signature::{self, UnparsedPublicKey};
use sd_jwt_payload::{Hasher, SdJwt, Sha256Hasher};
use serde_json::Value;
use tracing::{debug, instrument, warn};

use crate::error::CryptoError;

use super::types::VerificationResult;

/// SD-JWT verifier for credential presentations.
///
/// Verifies:
/// 1. Issuer signature (using provided public key)
/// 2. Expiration (validUntil claim)
/// 3. Key Binding JWT (holder proof of possession)
/// 4. Disclosure integrity (hashes match)
///
/// # Performance Target
///
/// < 50ms for cryptographic verification (no network calls)
pub struct SdJwtVerifier {
    /// Expected audience for KB-JWT (verifier's identifier)
    expected_audience: Option<String>,

    /// Maximum age of KB-JWT in seconds
    max_kb_jwt_age_secs: i64,
}

impl SdJwtVerifier {
    /// Create a new verifier.
    pub fn new() -> Self {
        Self {
            expected_audience: None,
            max_kb_jwt_age_secs: 300, // 5 minutes default
        }
    }

    /// Set the expected audience for KB-JWT validation.
    #[must_use]
    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.expected_audience = Some(audience.into());
        self
    }

    /// Set the maximum age for KB-JWT.
    #[must_use]
    pub fn with_max_kb_jwt_age(mut self, seconds: i64) -> Self {
        self.max_kb_jwt_age_secs = seconds;
        self
    }

    /// Verify an SD-JWT presentation.
    ///
    /// # Arguments
    /// * `sd_jwt` - The SD-JWT presentation to verify
    /// * `issuer_public_key` - The issuer's public key (Ed25519, 32 bytes)
    /// * `expected_nonce` - The nonce that was sent in the challenge (for KB-JWT)
    ///
    /// # Returns
    /// A `VerificationResult` with:
    /// - `signature_valid`: Whether the issuer signature is valid
    /// - `expired`: Whether the credential is expired
    /// - `disclosed_claims`: The fully resolved claims from disclosures
    /// - `issuer`: The issuer DID from the credential
    /// - `holder`: The holder DID (if KB-JWT present and valid)
    /// - `key_binding_valid`: Whether the KB-JWT is valid
    ///
    /// # Errors
    /// Returns an error if the SD-JWT cannot be parsed or verified.
    #[instrument(skip(self, sd_jwt, issuer_public_key), fields(disclosures = sd_jwt.disclosures().len()))]
    pub fn verify(
        &self,
        sd_jwt: &SdJwt,
        issuer_public_key: &[u8],
        expected_nonce: Option<&str>,
    ) -> Result<VerificationResult, CryptoError> {
        let start = Instant::now();

        // 1. Verify issuer signature
        let signature_valid = Self::verify_issuer_signature(sd_jwt, issuer_public_key)?;

        // 2. Check expiration
        let expired = Self::check_expiration(sd_jwt);

        // 3. Extract issuer
        let issuer = Self::extract_issuer(sd_jwt);

        // 4. Resolve disclosed claims
        let disclosed_claims = Self::resolve_disclosures(sd_jwt)?;

        // 5. Verify key binding JWT (if present)
        let (holder, key_binding_valid) = if let Some(kb_jwt) = sd_jwt.key_binding_jwt() {
            let holder_key = sd_jwt.required_key_bind();
            self.verify_key_binding(sd_jwt, kb_jwt, holder_key, expected_nonce)?
        } else {
            (None, false)
        };

        let elapsed = start.elapsed();
        debug!(
            elapsed_ms = elapsed.as_millis(),
            "SD-JWT verification complete"
        );

        // Performance check (target: <50ms)
        if elapsed.as_millis() > 50 {
            warn!(
                elapsed_ms = elapsed.as_millis(),
                "SD-JWT verification exceeded 50ms target"
            );
        }

        Ok(VerificationResult {
            signature_valid,
            expired,
            disclosed_claims,
            issuer,
            holder,
            key_binding_valid,
        })
    }

    /// Verify an SD-JWT presentation string.
    ///
    /// Convenience method that parses the string first.
    ///
    /// # Errors
    /// Returns an error if the presentation string cannot be parsed or verified.
    pub fn verify_presentation(
        &self,
        presentation: &str,
        issuer_public_key: &[u8],
        expected_nonce: Option<&str>,
    ) -> Result<VerificationResult, CryptoError> {
        let sd_jwt = SdJwt::parse(presentation)
            .map_err(|e| CryptoError::Internal(format!("Failed to parse SD-JWT: {e}")))?;

        self.verify(&sd_jwt, issuer_public_key, expected_nonce)
    }

    /// Verify the issuer signature on the JWT.
    fn verify_issuer_signature(
        sd_jwt: &SdJwt,
        issuer_public_key: &[u8],
    ) -> Result<bool, CryptoError> {
        // Get the raw JWS from the presentation
        let presentation = sd_jwt.presentation();
        let jwt_part = presentation
            .split('~')
            .next()
            .ok_or_else(|| CryptoError::Internal("Invalid SD-JWT: no JWT part".to_string()))?;

        // Split JWS into parts
        let parts: Vec<&str> = jwt_part.split('.').collect();
        if parts.len() != 3 {
            return Err(CryptoError::Internal(
                "Invalid JWS: expected 3 parts".to_string(),
            ));
        }

        let header_b64 = parts[0];
        let payload_b64 = parts[1];
        let signature_b64 = parts[2];

        // Determine algorithm from header
        let header_bytes = URL_SAFE_NO_PAD
            .decode(header_b64)
            .map_err(|e| CryptoError::Internal(format!("Invalid header encoding: {e}")))?;
        let header: Value = serde_json::from_slice(&header_bytes)
            .map_err(|e| CryptoError::Internal(format!("Invalid header JSON: {e}")))?;

        let alg = header
            .get("alg")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CryptoError::Internal("Missing alg in header".to_string()))?;

        // Verify based on algorithm
        let signing_input = format!("{header_b64}.{payload_b64}");
        let signature = URL_SAFE_NO_PAD
            .decode(signature_b64)
            .map_err(|e| CryptoError::Internal(format!("Invalid signature encoding: {e}")))?;

        match alg {
            "EdDSA" => {
                let public_key = UnparsedPublicKey::new(&signature::ED25519, issuer_public_key);
                Ok(public_key
                    .verify(signing_input.as_bytes(), &signature)
                    .is_ok())
            }
            "ES256" => {
                let public_key =
                    UnparsedPublicKey::new(&signature::ECDSA_P256_SHA256_FIXED, issuer_public_key);
                Ok(public_key
                    .verify(signing_input.as_bytes(), &signature)
                    .is_ok())
            }
            _ => Err(CryptoError::Internal(format!(
                "Unsupported algorithm: {alg}"
            ))),
        }
    }

    /// Check if the credential is expired.
    fn check_expiration(sd_jwt: &SdJwt) -> bool {
        let claims = sd_jwt.claims();

        // Check validUntil in properties
        if let Some(valid_until) = claims.get("validUntil").and_then(|v| v.as_str()) {
            if let Ok(exp) = chrono::DateTime::parse_from_rfc3339(valid_until) {
                return exp < Utc::now();
            }
        }

        // Check exp claim (standard JWT)
        if let Some(exp) = claims.get("exp").and_then(Value::as_i64) {
            return exp < Utc::now().timestamp();
        }

        false
    }

    /// Extract issuer from claims.
    fn extract_issuer(sd_jwt: &SdJwt) -> String {
        sd_jwt
            .claims()
            .get("issuer")
            .or_else(|| sd_jwt.claims().get("iss"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// Resolve disclosures into fully expanded claims.
    fn resolve_disclosures(sd_jwt: &SdJwt) -> Result<Value, CryptoError> {
        let disclosed = sd_jwt
            .clone()
            .into_disclosed_object(&Sha256Hasher::new())
            .map_err(|e| CryptoError::Internal(format!("Failed to resolve disclosures: {e}")))?;
        Ok(Value::Object(disclosed))
    }

    /// Verify the Key Binding JWT.
    fn verify_key_binding(
        &self,
        sd_jwt: &SdJwt,
        kb_jwt: &sd_jwt_payload::KeyBindingJwt,
        holder_key: Option<&sd_jwt_payload::RequiredKeyBinding>,
        expected_nonce: Option<&str>,
    ) -> Result<(Option<String>, bool), CryptoError> {
        // Get KB-JWT claims
        let kb_claims = kb_jwt.claims();

        // Verify nonce if expected
        if let Some(expected) = expected_nonce {
            if kb_claims.nonce != expected {
                debug!(
                    expected = expected,
                    actual = kb_claims.nonce,
                    "KB-JWT nonce mismatch"
                );
                return Ok((None, false));
            }
        }

        // Verify audience if configured
        if let Some(ref expected_aud) = self.expected_audience {
            if &kb_claims.aud != expected_aud {
                debug!(
                    expected = expected_aud,
                    actual = kb_claims.aud,
                    "KB-JWT audience mismatch"
                );
                return Ok((None, false));
            }
        }

        // Verify KB-JWT age
        let now = Utc::now().timestamp();
        let age = now - kb_claims.iat;
        if age < 0 || age > self.max_kb_jwt_age_secs {
            debug!(age_secs = age, "KB-JWT too old or issued in future");
            return Ok((None, false));
        }

        let kb_str = kb_jwt.to_string();
        let parts: Vec<&str> = kb_str.split('.').collect();
        if parts.len() != 3 {
            return Ok((None, false));
        }

        let header_bytes = URL_SAFE_NO_PAD
            .decode(parts[0])
            .map_err(|_| CryptoError::Internal("Invalid KB-JWT header encoding".to_string()))?;
        let header: Value = serde_json::from_slice(&header_bytes)
            .map_err(|_| CryptoError::Internal("Invalid KB-JWT header JSON".to_string()))?;
        let alg = header
            .get("alg")
            .and_then(Value::as_str)
            .ok_or_else(|| CryptoError::Internal("Missing KB-JWT alg".to_string()))?;

        let presentation_without_kb = presentation_without_kb(sd_jwt);
        let expected_sd_hash =
            URL_SAFE_NO_PAD.encode(Sha256Hasher::new().digest(presentation_without_kb.as_bytes()));
        if kb_claims.sd_hash != expected_sd_hash {
            debug!(
                expected = expected_sd_hash,
                actual = kb_claims.sd_hash,
                "KB-JWT sd_hash mismatch"
            );
            return Ok((None, false));
        }

        // Extract holder public key from cnf claim
        let (holder_public_key, holder_did) = match holder_key {
            Some(sd_jwt_payload::RequiredKeyBinding::Jwk(jwk)) => {
                let key = Self::extract_verification_key_from_jwk_object(jwk, alg)?;
                (key, None)
            }
            Some(sd_jwt_payload::RequiredKeyBinding::Kid(kid)) => {
                let holder_did = if kid.starts_with("did:") {
                    Some(kid.clone())
                } else {
                    None
                };
                let key = match alg {
                    "EdDSA" => VerificationKey::Ed25519(Vec::new()),
                    "ES256" => VerificationKey::P256(Vec::new()),
                    _ => return Ok((None, false)),
                };
                (key, holder_did)
            }
            _ => return Ok((None, false)),
        };

        // Verify KB-JWT signature
        let signing_input = format!("{}.{}", parts[0], parts[1]);
        let signature = URL_SAFE_NO_PAD
            .decode(parts[2])
            .map_err(|_| CryptoError::Internal("Invalid KB-JWT signature encoding".to_string()))?;

        let valid = match &holder_public_key {
            VerificationKey::Ed25519(public_key) => {
                let public_key = UnparsedPublicKey::new(&signature::ED25519, public_key);
                public_key
                    .verify(signing_input.as_bytes(), &signature)
                    .is_ok()
            }
            VerificationKey::P256(public_key) => {
                let public_key =
                    UnparsedPublicKey::new(&signature::ECDSA_P256_SHA256_FIXED, public_key);
                public_key
                    .verify(signing_input.as_bytes(), &signature)
                    .is_ok()
            }
        };

        let holder_did = holder_did.or_else(|| match &holder_public_key {
            VerificationKey::Ed25519(public_key) => Some(Self::derive_did_key(public_key)),
            VerificationKey::P256(_) => None,
        });

        Ok((holder_did, valid))
    }

    /// Extract a verification key from a JWK object (Map<String, Value>).
    fn extract_verification_key_from_jwk_object(
        jwk: &sd_jwt_payload::JsonObject,
        alg: &str,
    ) -> Result<VerificationKey, CryptoError> {
        let kty = jwk.get("kty").and_then(|v| v.as_str());
        let crv = jwk.get("crv").and_then(|v| v.as_str());
        let x = jwk.get("x").and_then(|v| v.as_str());
        let y = jwk.get("y").and_then(|v| v.as_str());

        match (alg, kty, crv, x, y) {
            ("EdDSA", Some("OKP"), Some("Ed25519"), Some(x_b64), _) => URL_SAFE_NO_PAD
                .decode(x_b64)
                .map(VerificationKey::Ed25519)
                .map_err(|e| CryptoError::Internal(format!("Invalid JWK x coordinate: {e}"))),
            ("ES256", Some("EC"), Some("P-256"), Some(x_b64), Some(y_b64)) => {
                let x = URL_SAFE_NO_PAD
                    .decode(x_b64)
                    .map_err(|e| CryptoError::Internal(format!("Invalid JWK x coordinate: {e}")))?;
                let y = URL_SAFE_NO_PAD
                    .decode(y_b64)
                    .map_err(|e| CryptoError::Internal(format!("Invalid JWK y coordinate: {e}")))?;
                let mut public_key = vec![0x04];
                public_key.extend_from_slice(&x);
                public_key.extend_from_slice(&y);
                Ok(VerificationKey::P256(public_key))
            }
            _ => Err(CryptoError::Internal(
                "JWK is not valid for the KB-JWT algorithm".to_string(),
            )),
        }
    }

    /// Derive a did:key from an Ed25519 public key.
    fn derive_did_key(public_key: &[u8]) -> String {
        // Multicodec prefix for Ed25519 public key: 0xed01
        let mut multicodec = vec![0xed, 0x01];
        multicodec.extend_from_slice(public_key);

        // Base58-btc encode with 'z' prefix
        let encoded = bs58::encode(&multicodec).into_string();
        format!("did:key:z{encoded}")
    }
}

enum VerificationKey {
    Ed25519(Vec<u8>),
    P256(Vec<u8>),
}

fn presentation_without_kb(sd_jwt: &SdJwt) -> String {
    let presentation = sd_jwt.presentation();
    let mut segments: Vec<&str> = presentation.split('~').collect();
    if sd_jwt.key_binding_jwt().is_some() {
        let _ = segments.pop();
    }
    format!("{}~", segments.join("~"))
}

impl Default for SdJwtVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kms::{KeyAlgorithm, KmsProvider, SoftwareKmsProvider};
    use crate::sd_jwt::issuer::{HolderKeyBuilder, SdJwtIssuer};
    use crate::sd_jwt::types::{
        ClaimPath, CredentialSubject, IssuanceOptions, VaultPassCredential,
    };
    use chrono::Duration;
    use std::sync::Arc;

    #[tokio::test]
    async fn verify_valid_sd_jwt() {
        let kms: Arc<dyn KmsProvider> = Arc::new(SoftwareKmsProvider::new());

        // Generate issuer key
        let issuer_key = kms
            .generate_key(KeyAlgorithm::Ed25519, "issuer", None)
            .await
            .unwrap();
        let issuer_public = kms.export_public_key(&issuer_key).await.unwrap();

        // Generate holder key
        let holder_key = kms
            .generate_key(KeyAlgorithm::Ed25519, "holder", None)
            .await
            .unwrap();
        let holder_public = kms.export_public_key(&holder_key).await.unwrap();
        let holder_jwk = HolderKeyBuilder::ed25519_jwk(holder_public.as_bytes());

        // Issue credential
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
        let sd_jwt = issuer
            .issue(
                &credential,
                IssuanceOptions {
                    key_handle: issuer_key.id().to_string(),
                    concealable_claims: vec![ClaimPath::new(ClaimPath::NAME)],
                    decoy_count: 2,
                    holder_public_key: Some(holder_jwk),
                },
            )
            .await
            .unwrap();

        // Verify
        let verifier = SdJwtVerifier::new();
        let result = verifier
            .verify(&sd_jwt, issuer_public.as_bytes(), None)
            .unwrap();

        assert!(result.signature_valid, "Signature should be valid");
        assert!(!result.expired, "Credential should not be expired");
        assert_eq!(result.issuer, "did:web:issuer.sahi.my");
    }

    #[tokio::test]
    async fn verify_rejects_tampered_signature() {
        let kms: Arc<dyn KmsProvider> = Arc::new(SoftwareKmsProvider::new());

        let issuer_key = kms
            .generate_key(KeyAlgorithm::Ed25519, "issuer", None)
            .await
            .unwrap();
        let wrong_key = kms
            .generate_key(KeyAlgorithm::Ed25519, "wrong", None)
            .await
            .unwrap();
        let wrong_public = kms.export_public_key(&wrong_key).await.unwrap();

        let subject = CredentialSubject::new(
            "did:key:z6Mkh".to_string(),
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
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        // Verify with wrong key
        let verifier = SdJwtVerifier::new();
        let result = verifier
            .verify(&sd_jwt, wrong_public.as_bytes(), None)
            .unwrap();

        assert!(
            !result.signature_valid,
            "Signature should be invalid with wrong key"
        );
    }

    #[tokio::test]
    async fn verify_detects_expired_credential() {
        let kms: Arc<dyn KmsProvider> = Arc::new(SoftwareKmsProvider::new());

        let issuer_key = kms
            .generate_key(KeyAlgorithm::Ed25519, "issuer", None)
            .await
            .unwrap();
        let issuer_public = kms.export_public_key(&issuer_key).await.unwrap();

        let subject = CredentialSubject::new(
            "did:key:z6Mkh".to_string(),
            "PRY_01HXK".to_string(),
            "visitor".to_string(),
        );
        // Set expiration in the past
        let credential = VaultPassCredential::new(
            "did:web:issuer.sahi.my".to_string(),
            subject,
            Utc::now() - Duration::days(2),
            Some(Utc::now() - Duration::days(1)),
        );

        let issuer = SdJwtIssuer::new(Arc::clone(&kms));
        let sd_jwt = issuer
            .issue(
                &credential,
                IssuanceOptions {
                    key_handle: issuer_key.id().to_string(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        let verifier = SdJwtVerifier::new();
        let result = verifier
            .verify(&sd_jwt, issuer_public.as_bytes(), None)
            .unwrap();

        assert!(result.expired, "Credential should be expired");
    }

    #[test]
    fn derive_did_key_format_is_correct() {
        let fake_key = [0u8; 32];
        let did = SdJwtVerifier::derive_did_key(&fake_key);

        assert!(did.starts_with("did:key:z"));
    }
}
