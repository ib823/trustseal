//! SD-JWT verification for edge verifier.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::DateTime;
use ring::signature::{self, UnparsedPublicKey};
use sd_jwt_payload::{SdJwt, Sha256Hasher};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;
use tracing::debug;

/// SD-JWT verification errors.
#[derive(Debug, Error)]
pub enum VerificationError {
    #[error("Invalid SD-JWT format")]
    InvalidFormat,

    #[error("Missing issuer JWT")]
    MissingIssuerJwt,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    #[error("Expired credential")]
    Expired,

    #[error("Not yet valid")]
    NotYetValid,

    #[error("Invalid disclosure: {0}")]
    InvalidDisclosure(String),

    #[error("Invalid key binding JWT")]
    InvalidKeyBinding,

    #[error("Key binding required but missing")]
    KeyBindingRequired,

    #[error("DID resolution failed: {0}")]
    DidResolutionFailed(String),

    #[error("Unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),

    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),
}

/// Verification result with disclosed claims.
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// The issuer DID.
    pub issuer_did: String,

    /// The holder DID (from key binding or cnf claim).
    pub holder_did: Option<String>,

    /// Credential type.
    pub credential_type: Option<String>,

    /// Disclosed claims.
    pub claims: Value,

    /// Expiration timestamp.
    pub exp: Option<i64>,

    /// Not before timestamp.
    pub nbf: Option<i64>,

    /// Issued at timestamp.
    pub iat: Option<i64>,

    /// Credential status (for revocation checking).
    pub status: Option<CredentialStatus>,
}

/// Credential status information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialStatus {
    /// Status list URL.
    pub status_list_credential: String,

    /// Index in the status list.
    pub status_list_index: usize,

    /// Purpose (e.g., "revocation").
    pub status_purpose: String,
}

/// SD-JWT verifier.
pub struct SdJwtVerifier {
    /// DID resolver.
    resolver: super::did_resolver::DidResolver,

    /// Whether key binding is required.
    require_key_binding: bool,
}

impl SdJwtVerifier {
    /// Create a new verifier.
    pub fn new(resolver: super::did_resolver::DidResolver) -> Self {
        Self {
            resolver,
            require_key_binding: true,
        }
    }

    /// Set whether key binding is required.
    pub fn require_key_binding(mut self, required: bool) -> Self {
        self.require_key_binding = required;
        self
    }

    /// Verify an SD-JWT presentation.
    pub async fn verify(
        &self,
        sd_jwt: &str,
        nonce: Option<&str>,
        audience: Option<&str>,
    ) -> Result<VerificationResult, VerificationError> {
        // Parse SD-JWT format: <issuer-jwt>~<disclosure>~...~[<kb-jwt>]
        let parts: Vec<&str> = sd_jwt.split('~').collect();
        if parts.is_empty() {
            return Err(VerificationError::InvalidFormat);
        }

        let issuer_jwt = parts[0];
        if issuer_jwt.is_empty() {
            return Err(VerificationError::MissingIssuerJwt);
        }

        // Collect disclosures (middle parts)
        let disclosures: Vec<&str> = if parts.len() > 1 {
            // Last part might be KB-JWT (doesn't start with 'e' typically for base64)
            let last = parts.last().unwrap_or(&"");
            if last.is_empty() || !last.contains('.') {
                parts[1..]
                    .iter()
                    .filter(|s| !s.is_empty())
                    .copied()
                    .collect()
            } else {
                // Has KB-JWT
                parts[1..parts.len() - 1]
                    .iter()
                    .filter(|s| !s.is_empty())
                    .copied()
                    .collect()
            }
        } else {
            Vec::new()
        };

        // Check for key binding JWT
        let kb_jwt = if parts.len() > 1 {
            let last = *parts.last().unwrap();
            if !last.is_empty() && last.contains('.') {
                Some(last)
            } else {
                None
            }
        } else {
            None
        };

        if self.require_key_binding && kb_jwt.is_none() {
            return Err(VerificationError::KeyBindingRequired);
        }

        // Parse issuer JWT
        let jwt_parts: Vec<&str> = issuer_jwt.split('.').collect();
        if jwt_parts.len() != 3 {
            return Err(VerificationError::InvalidFormat);
        }

        let header: Value = Self::decode_jwt_part(jwt_parts[0])?;
        let payload: Value = Self::decode_jwt_part(jwt_parts[1])?;

        debug!("SD-JWT header: {:?}", header);

        // Get algorithm
        let alg = header["alg"]
            .as_str()
            .ok_or(VerificationError::InvalidFormat)?;

        // Get issuer
        let issuer_did = payload["iss"]
            .as_str()
            .or_else(|| payload["issuer"].as_str())
            .ok_or(VerificationError::InvalidFormat)?
            .to_string();

        // Resolve issuer DID
        let did_doc = self
            .resolver
            .resolve(&issuer_did)
            .await
            .map_err(|e| VerificationError::DidResolutionFailed(e.to_string()))?;

        // Verify signature (stub - would use actual crypto)
        self.verify_signature(issuer_jwt, &did_doc, alg)?;

        // Process disclosures
        let disclosed_claims = Self::resolve_disclosures(sd_jwt)?;
        let presentation_without_kb =
            Self::presentation_without_key_binding(issuer_jwt, &disclosures);

        // Verify key binding if present
        let holder_did = if let Some(kb) = kb_jwt {
            self.verify_key_binding(kb, &payload, nonce, audience, &presentation_without_kb)
                .await?
        } else {
            // Try to get holder from cnf claim
            payload["cnf"]["kid"].as_str().map(String::from)
        };

        // Check expiration
        let exp = Self::extract_timestamp(&disclosed_claims, &["exp"], &["validUntil"]);
        let nbf = Self::extract_timestamp(&disclosed_claims, &["nbf"], &["validFrom"]);
        let iat = Self::extract_timestamp(&disclosed_claims, &["iat"], &["issuedAt"]);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as i64;

        if let Some(exp_time) = exp {
            if now > exp_time {
                return Err(VerificationError::Expired);
            }
        }

        if let Some(nbf_time) = nbf {
            if now < nbf_time {
                return Err(VerificationError::NotYetValid);
            }
        }

        // Extract credential type
        let credential_type = Self::extract_credential_type(&disclosed_claims);

        // Extract status
        let status = Self::extract_status(&disclosed_claims);

        Ok(VerificationResult {
            issuer_did,
            holder_did,
            credential_type,
            claims: disclosed_claims,
            exp,
            nbf,
            iat,
            status,
        })
    }

    fn decode_jwt_part(part: &str) -> Result<Value, VerificationError> {
        let bytes = URL_SAFE_NO_PAD.decode(part)?;
        serde_json::from_slice(&bytes).map_err(Into::into)
    }

    fn verify_signature(
        &self,
        jwt: &str,
        did_doc: &super::did_resolver::DidDocument,
        alg: &str,
    ) -> Result<(), VerificationError> {
        let method = Self::select_verification_method(
            did_doc,
            did_doc.assertion_method.first().map(String::as_str),
            false,
        )?;
        let key = Self::verification_key_from_method(method)?;
        Self::verify_jws_signature(jwt, alg, &key)
    }

    fn resolve_disclosures(presentation: &str) -> Result<Value, VerificationError> {
        let sd_jwt = SdJwt::parse(presentation)
            .map_err(|e| VerificationError::InvalidDisclosure(e.to_string()))?;
        let disclosed = sd_jwt
            .into_disclosed_object(&Sha256Hasher::new())
            .map_err(|e| VerificationError::InvalidDisclosure(e.to_string()))?;
        Ok(Value::Object(disclosed))
    }

    fn extract_credential_type(claims: &Value) -> Option<String> {
        if let Some(vct) = claims.get("vct").and_then(Value::as_str) {
            return Some(vct.to_string());
        }

        if let Some(claim_type) = claims.get("type") {
            if let Some(as_str) = claim_type.as_str() {
                return Some(as_str.to_string());
            }

            if let Some(as_array) = claim_type.as_array() {
                return as_array
                    .iter()
                    .rev()
                    .filter_map(Value::as_str)
                    .find(|value| *value != "VerifiableCredential")
                    .or_else(|| as_array.iter().find_map(Value::as_str))
                    .map(str::to_string);
            }
        }

        None
    }

    fn extract_status(claims: &Value) -> Option<CredentialStatus> {
        let status_obj = claims
            .get("credentialStatus")
            .or_else(|| claims.get("status"))?
            .as_object()?;

        let status_list_credential = status_obj
            .get("statusListCredential")
            .or_else(|| status_obj.get("status_list_credential"))
            .and_then(Value::as_str)?
            .to_string();
        let status_purpose = status_obj
            .get("statusPurpose")
            .or_else(|| status_obj.get("status_purpose"))
            .and_then(Value::as_str)?
            .to_string();
        let status_list_index = status_obj
            .get("statusListIndex")
            .or_else(|| status_obj.get("status_list_index"))
            .and_then(|value| {
                value
                    .as_u64()
                    .and_then(|number| usize::try_from(number).ok())
                    .or_else(|| value.as_str()?.parse::<usize>().ok())
            })?;

        Some(CredentialStatus {
            status_list_credential,
            status_list_index,
            status_purpose,
        })
    }

    fn extract_timestamp(
        claims: &Value,
        numeric_keys: &[&str],
        rfc3339_keys: &[&str],
    ) -> Option<i64> {
        for key in numeric_keys {
            if let Some(timestamp) = claims.get(*key).and_then(Value::as_i64) {
                return Some(timestamp);
            }
        }

        for key in rfc3339_keys {
            if let Some(timestamp) = claims
                .get(*key)
                .and_then(Value::as_str)
                .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
                .map(|value| value.timestamp())
            {
                return Some(timestamp);
            }
        }

        None
    }

    async fn verify_key_binding(
        &self,
        kb_jwt: &str,
        issuer_payload: &Value,
        expected_nonce: Option<&str>,
        expected_aud: Option<&str>,
        presentation_without_kb: &str,
    ) -> Result<Option<String>, VerificationError> {
        let parts: Vec<&str> = kb_jwt.split('.').collect();
        if parts.len() != 3 {
            return Err(VerificationError::InvalidKeyBinding);
        }

        let header: Value = Self::decode_jwt_part(parts[0])?;
        let payload: Value = Self::decode_jwt_part(parts[1])?;
        let alg = header["alg"]
            .as_str()
            .ok_or(VerificationError::InvalidKeyBinding)?;

        // Verify nonce if provided
        if let Some(nonce) = expected_nonce {
            let kb_nonce = payload["nonce"].as_str();
            if kb_nonce != Some(nonce) {
                return Err(VerificationError::InvalidKeyBinding);
            }
        }

        // Verify audience if provided
        if let Some(aud) = expected_aud {
            let kb_aud = payload["aud"].as_str();
            if kb_aud != Some(aud) {
                return Err(VerificationError::InvalidKeyBinding);
            }
        }

        let sd_hash = payload["sd_hash"]
            .as_str()
            .ok_or(VerificationError::InvalidKeyBinding)?;
        let expected_sd_hash =
            URL_SAFE_NO_PAD.encode(Sha256::digest(presentation_without_kb.as_bytes()));
        if sd_hash != expected_sd_hash {
            return Err(VerificationError::InvalidKeyBinding);
        }

        let holder_did = issuer_payload["cnf"]["kid"]
            .as_str()
            .or_else(|| issuer_payload["sub"].as_str())
            .map(String::from);

        let key = self
            .resolve_holder_key(issuer_payload)
            .await?
            .ok_or(VerificationError::InvalidKeyBinding)?;
        Self::verify_jws_signature(kb_jwt, alg, &key)
            .map_err(|_| VerificationError::InvalidKeyBinding)?;

        Ok(holder_did)
    }

    fn presentation_without_key_binding(issuer_jwt: &str, disclosures: &[&str]) -> String {
        if disclosures.is_empty() {
            format!("{issuer_jwt}~")
        } else {
            format!("{issuer_jwt}~{}~", disclosures.join("~"))
        }
    }

    async fn resolve_holder_key(
        &self,
        issuer_payload: &Value,
    ) -> Result<Option<VerificationKey>, VerificationError> {
        if let Some(jwk) = issuer_payload.get("cnf").and_then(|cnf| cnf.get("jwk")) {
            return Ok(Some(Self::verification_key_from_jwk(jwk)?));
        }

        let holder_reference = issuer_payload
            .get("cnf")
            .and_then(|cnf| cnf.get("kid"))
            .and_then(Value::as_str)
            .or_else(|| issuer_payload.get("sub").and_then(Value::as_str));

        let Some(holder_reference) = holder_reference else {
            return Ok(None);
        };
        if !holder_reference.starts_with("did:") {
            return Ok(None);
        }

        let did_doc = self
            .resolver
            .resolve(holder_reference)
            .await
            .map_err(|e| VerificationError::DidResolutionFailed(e.to_string()))?;
        let method = Self::select_verification_method(&did_doc, Some(holder_reference), true)?;
        Ok(Some(Self::verification_key_from_method(method)?))
    }

    fn verify_jws_signature(
        jwt: &str,
        alg: &str,
        key: &VerificationKey,
    ) -> Result<(), VerificationError> {
        let parts: Vec<&str> = jwt.split('.').collect();
        if parts.len() != 3 {
            return Err(VerificationError::InvalidFormat);
        }

        let signing_input = format!("{}.{}", parts[0], parts[1]);
        let signature = URL_SAFE_NO_PAD.decode(parts[2])?;

        match (alg, key) {
            ("EdDSA", VerificationKey::Ed25519(public_key)) => {
                let verifier = UnparsedPublicKey::new(&signature::ED25519, public_key);
                verifier
                    .verify(signing_input.as_bytes(), &signature)
                    .map_err(|_| VerificationError::SignatureVerificationFailed)
            }
            ("ES256", VerificationKey::P256(public_key)) => {
                let verifier =
                    UnparsedPublicKey::new(&signature::ECDSA_P256_SHA256_FIXED, public_key);
                verifier
                    .verify(signing_input.as_bytes(), &signature)
                    .map_err(|_| VerificationError::SignatureVerificationFailed)
            }
            ("EdDSA", _) | ("ES256", _) => Err(VerificationError::InvalidSignature),
            _ => Err(VerificationError::UnsupportedAlgorithm(alg.to_string())),
        }
    }

    fn select_verification_method<'a>(
        did_doc: &'a super::did_resolver::DidDocument,
        preferred_id: Option<&str>,
        prefer_authentication: bool,
    ) -> Result<&'a super::did_resolver::VerificationMethod, VerificationError> {
        if let Some(preferred_id) = preferred_id {
            if let Some(method) = did_doc
                .verification_method
                .iter()
                .find(|method| method.id == preferred_id)
            {
                return Ok(method);
            }
        }

        let relationship_ids = if prefer_authentication {
            &did_doc.authentication
        } else {
            &did_doc.assertion_method
        };

        for method_id in relationship_ids {
            if let Some(method) = did_doc
                .verification_method
                .iter()
                .find(|candidate| candidate.id == *method_id)
            {
                return Ok(method);
            }
        }

        did_doc.verification_method.first().ok_or_else(|| {
            VerificationError::DidResolutionFailed("No verification method found".to_string())
        })
    }

    fn verification_key_from_method(
        method: &super::did_resolver::VerificationMethod,
    ) -> Result<VerificationKey, VerificationError> {
        if let Some(jwk) = &method.public_key_jwk {
            return Self::verification_key_from_jwk(jwk);
        }

        if let Some(multibase) = &method.public_key_multibase {
            return Self::verification_key_from_multibase(multibase);
        }

        Err(VerificationError::DidResolutionFailed(
            "Verification method does not contain a supported key format".to_string(),
        ))
    }

    fn verification_key_from_jwk(jwk: &Value) -> Result<VerificationKey, VerificationError> {
        let kty = jwk.get("kty").and_then(Value::as_str).unwrap_or_default();
        let crv = jwk.get("crv").and_then(Value::as_str).unwrap_or_default();

        match (kty, crv) {
            ("OKP", "Ed25519") => {
                let x = jwk
                    .get("x")
                    .and_then(Value::as_str)
                    .ok_or(VerificationError::InvalidSignature)?;
                let key = URL_SAFE_NO_PAD.decode(x)?;
                Ok(VerificationKey::Ed25519(key))
            }
            ("EC", "P-256") => {
                let x = jwk
                    .get("x")
                    .and_then(Value::as_str)
                    .ok_or(VerificationError::InvalidSignature)?;
                let y = jwk
                    .get("y")
                    .and_then(Value::as_str)
                    .ok_or(VerificationError::InvalidSignature)?;
                let x = URL_SAFE_NO_PAD.decode(x)?;
                let y = URL_SAFE_NO_PAD.decode(y)?;
                let mut key = vec![0x04];
                key.extend_from_slice(&x);
                key.extend_from_slice(&y);
                Ok(VerificationKey::P256(key))
            }
            _ => Err(VerificationError::InvalidSignature),
        }
    }

    fn verification_key_from_multibase(
        multibase: &str,
    ) -> Result<VerificationKey, VerificationError> {
        let encoded = multibase
            .strip_prefix('z')
            .ok_or(VerificationError::InvalidSignature)?;
        let decoded = bs58::decode(encoded)
            .into_vec()
            .map_err(|_| VerificationError::InvalidSignature)?;
        if decoded.len() < 3 {
            return Err(VerificationError::InvalidSignature);
        }

        match (decoded[0], decoded[1]) {
            (0xed, 0x01) => Ok(VerificationKey::Ed25519(decoded[2..].to_vec())),
            (0x80, 0x24) => {
                let raw_key = decoded[2..].to_vec();
                if raw_key.len() == 65 && raw_key.first() == Some(&0x04) {
                    Ok(VerificationKey::P256(raw_key))
                } else {
                    Err(VerificationError::InvalidSignature)
                }
            }
            _ => Err(VerificationError::InvalidSignature),
        }
    }
}

enum VerificationKey {
    Ed25519(Vec<u8>),
    P256(Vec<u8>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use crypto_engine::did::from_ed25519_public_key;
    use crypto_engine::kms::{KeyAlgorithm, KmsProvider, SoftwareKmsProvider};
    use crypto_engine::sd_jwt::{
        ClaimPath, CredentialSubject, HolderKeyBuilder, IssuanceOptions, PresentationOptions,
        SdJwtHolder, SdJwtIssuer, VaultPassCredential,
    };
    use std::sync::Arc;

    fn create_test_verifier() -> SdJwtVerifier {
        let resolver = super::super::did_resolver::DidResolver::new();
        SdJwtVerifier::new(resolver).require_key_binding(false)
    }

    #[test]
    fn test_decode_jwt_part() {
        let payload = r#"{"iss":"did:web:test","sub":"holder"}"#;
        let encoded = URL_SAFE_NO_PAD.encode(payload);

        let decoded = SdJwtVerifier::decode_jwt_part(&encoded).unwrap();
        assert_eq!(decoded["iss"], "did:web:test");
    }

    #[test]
    fn test_invalid_format() {
        let verifier = create_test_verifier();

        // Empty string
        let result = tokio_test::block_on(verifier.verify("", None, None));
        assert!(matches!(result, Err(VerificationError::MissingIssuerJwt)));
    }

    #[test]
    fn test_verification_result() {
        let result = VerificationResult {
            issuer_did: "did:web:sahi.my".to_string(),
            holder_did: Some("did:key:z6Mk...".to_string()),
            credential_type: Some("ResidentBadge".to_string()),
            claims: serde_json::json!({"name": "Test"}),
            exp: Some(1893456000),
            nbf: None,
            iat: Some(1704067200),
            status: None,
        };

        assert_eq!(result.issuer_did, "did:web:sahi.my");
        assert!(result.holder_did.is_some());
    }

    #[test]
    fn test_credential_status_parse() {
        let json = serde_json::json!({
            "status_list_credential": "https://api.sahi.my/status/1",
            "status_list_index": 42,
            "status_purpose": "revocation"
        });

        let status: CredentialStatus = serde_json::from_value(json).unwrap();
        assert_eq!(status.status_list_index, 42);
        assert_eq!(status.status_purpose, "revocation");
    }

    #[test]
    fn test_extract_credential_type_from_vc_type_array() {
        let claims = serde_json::json!({
            "type": ["VerifiableCredential", "AccessBadge"]
        });

        assert_eq!(
            SdJwtVerifier::extract_credential_type(&claims).as_deref(),
            Some("AccessBadge")
        );
    }

    #[tokio::test]
    async fn test_verify_signed_presentation() {
        let kms: Arc<dyn KmsProvider> = Arc::new(SoftwareKmsProvider::new());
        let issuer_key = kms
            .generate_key(KeyAlgorithm::Ed25519, "issuer", None)
            .await
            .unwrap();
        let issuer_public = kms.export_public_key(&issuer_key).await.unwrap();
        let issuer_did = from_ed25519_public_key(issuer_public.as_bytes());

        let holder_key = kms
            .generate_key(KeyAlgorithm::Ed25519, "holder", None)
            .await
            .unwrap();
        let holder_public = kms.export_public_key(&holder_key).await.unwrap();
        let holder_jwk = HolderKeyBuilder::ed25519_jwk(holder_public.as_bytes());

        let credential = VaultPassCredential::new(
            issuer_did,
            CredentialSubject {
                id: "did:key:z6Mkh123".to_string(),
                property_id: "PRY_01HXK".to_string(),
                unit: Some("12-03".to_string()),
                name: Some("Test User".to_string()),
                role: "resident".to_string(),
                access_zones: vec!["lobby".to_string()],
                time_restrictions: None,
            },
            Utc::now(),
            Some(Utc::now() + Duration::days(30)),
        );

        let issuer = SdJwtIssuer::new(Arc::clone(&kms));
        let sd_jwt = issuer
            .issue(
                &credential,
                IssuanceOptions {
                    key_handle: issuer_key.id().to_string(),
                    concealable_claims: vec![ClaimPath::new(ClaimPath::NAME)],
                    decoy_count: 1,
                    holder_public_key: Some(holder_jwk),
                },
            )
            .await
            .unwrap();

        let holder = SdJwtHolder::new(Arc::clone(&kms));
        let presentation = holder
            .derive_presentation_serialized(
                &sd_jwt,
                PresentationOptions {
                    disclosed_claims: vec![ClaimPath::new(ClaimPath::NAME)],
                    audience: "VRF_site".to_string(),
                    nonce: "nonce-123".to_string(),
                    holder_key_handle: holder_key.id().to_string(),
                },
            )
            .await
            .unwrap();

        let resolver = super::super::did_resolver::DidResolver::new();
        let verifier = SdJwtVerifier::new(resolver);
        let result = verifier
            .verify(&presentation, Some("nonce-123"), Some("VRF_site"))
            .await
            .unwrap();

        assert_eq!(result.issuer_did, credential.issuer);
        assert_eq!(result.holder_did, None);
        assert_eq!(result.credential_type.as_deref(), Some("AccessBadge"));
        assert_eq!(result.claims["credentialSubject"]["name"], "Test User");
    }
}
