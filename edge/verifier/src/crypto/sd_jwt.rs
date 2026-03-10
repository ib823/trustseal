//! SD-JWT verification for edge verifier.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;
use tracing::{debug, warn};

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
                parts[1..].iter().filter(|s| !s.is_empty()).copied().collect()
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
        let disclosed_claims = self.process_disclosures(&payload, &disclosures)?;

        // Verify key binding if present
        let holder_did = if let Some(kb) = kb_jwt {
            self.verify_key_binding(kb, &payload, nonce, audience)
                .await?
        } else {
            // Try to get holder from cnf claim
            payload["cnf"]["kid"].as_str().map(String::from)
        };

        // Check expiration
        let exp = payload["exp"].as_i64();
        let nbf = payload["nbf"].as_i64();
        let iat = payload["iat"].as_i64();

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
        let credential_type = disclosed_claims["vct"]
            .as_str()
            .or_else(|| disclosed_claims["type"].as_str())
            .map(String::from);

        // Extract status
        let status = if let Some(status_obj) = disclosed_claims.get("status") {
            serde_json::from_value(status_obj.clone()).ok()
        } else {
            None
        };

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
        _jwt: &str,
        _did_doc: &super::did_resolver::DidDocument,
        alg: &str,
    ) -> Result<(), VerificationError> {
        // Stub: In production, this would:
        // 1. Get the verification method from DID document
        // 2. Extract the public key
        // 3. Verify the JWT signature using the appropriate algorithm

        match alg {
            "ES256" | "ES384" | "EdDSA" => {
                warn!("Signature verification stubbed - would verify {} signature", alg);
                Ok(())
            }
            _ => Err(VerificationError::UnsupportedAlgorithm(alg.to_string())),
        }
    }

    fn process_disclosures(
        &self,
        payload: &Value,
        disclosures: &[&str],
    ) -> Result<Value, VerificationError> {
        let mut result = payload.clone();

        // Build disclosure hash map
        let mut disclosure_map: std::collections::HashMap<String, Value> =
            std::collections::HashMap::new();

        for disclosure in disclosures {
            let bytes = URL_SAFE_NO_PAD
                .decode(disclosure)
                .map_err(|_| VerificationError::InvalidDisclosure(disclosure.to_string()))?;

            let disclosure_array: Value = serde_json::from_slice(&bytes)
                .map_err(|_| VerificationError::InvalidDisclosure(disclosure.to_string()))?;

            // Calculate hash
            let hash = {
                let mut hasher = Sha256::new();
                hasher.update(disclosure.as_bytes());
                URL_SAFE_NO_PAD.encode(hasher.finalize())
            };

            // Disclosure format: [salt, claim_name, claim_value] or [salt, array_element]
            if let Some(arr) = disclosure_array.as_array() {
                if arr.len() == 3 {
                    // Object property disclosure
                    let claim_name = arr[1]
                        .as_str()
                        .ok_or_else(|| VerificationError::InvalidDisclosure(disclosure.to_string()))?;
                    let claim_value = arr[2].clone();
                    disclosure_map.insert(hash.clone(), serde_json::json!({
                        "name": claim_name,
                        "value": claim_value
                    }));
                } else if arr.len() == 2 {
                    // Array element disclosure
                    disclosure_map.insert(hash.clone(), arr[1].clone());
                }
            }
        }

        // Collect disclosed claims to add
        let mut claims_to_add: Vec<(String, Value)> = Vec::new();

        if let Some(sd_claims) = result.get("_sd").and_then(|v| v.as_array()) {
            for hash in sd_claims {
                if let Some(hash_str) = hash.as_str() {
                    if let Some(disclosure) = disclosure_map.get(hash_str) {
                        if let Some(obj) = disclosure.as_object() {
                            if let (Some(name), Some(value)) = (obj.get("name"), obj.get("value")) {
                                if let Some(name_str) = name.as_str() {
                                    claims_to_add.push((name_str.to_string(), value.clone()));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Apply disclosed claims and remove _sd/_sd_alg
        if let Some(obj) = result.as_object_mut() {
            for (name, value) in claims_to_add {
                obj.insert(name, value);
            }
            obj.remove("_sd");
            obj.remove("_sd_alg");
        }

        Ok(result)
    }

    async fn verify_key_binding(
        &self,
        kb_jwt: &str,
        issuer_payload: &Value,
        expected_nonce: Option<&str>,
        expected_aud: Option<&str>,
    ) -> Result<Option<String>, VerificationError> {
        let parts: Vec<&str> = kb_jwt.split('.').collect();
        if parts.len() != 3 {
            return Err(VerificationError::InvalidKeyBinding);
        }

        let _header: Value = Self::decode_jwt_part(parts[0])?;
        let payload: Value = Self::decode_jwt_part(parts[1])?;

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

        // Get holder DID from cnf claim in issuer JWT
        let holder_did = issuer_payload["cnf"]["kid"]
            .as_str()
            .or_else(|| issuer_payload["sub"].as_str())
            .map(String::from);

        // In production, would verify KB-JWT signature against holder's key
        warn!("Key binding verification stubbed");

        Ok(holder_did)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
