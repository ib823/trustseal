//! DID Document types following W3C DID Core 1.0.
//!
//! Reference: <https://www.w3.org/TR/did-core/>

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A Decentralized Identifier (DID).
///
/// DIDs are URIs that associate a DID subject with a DID document,
/// enabling trustable interactions with the subject.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Did(pub String);

impl Did {
    /// Parse a DID string into its components.
    ///
    /// # Returns
    /// `None` if the string is not a valid DID.
    #[must_use]
    pub fn parse(s: &str) -> Option<DidComponents> {
        let s = s.trim();
        if !s.starts_with("did:") {
            return None;
        }

        let rest = &s[4..];
        let (method, rest) = rest.split_once(':')?;

        // Method must be lowercase alphanumeric
        if method.is_empty()
            || !method
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        {
            return None;
        }

        // The rest is method-specific-id, possibly with path/query/fragment
        let (method_specific_id, path, query, fragment) = parse_did_suffix(rest);

        Some(DidComponents {
            method: method.to_string(),
            method_specific_id: method_specific_id.to_string(),
            path,
            query,
            fragment,
        })
    }

    /// Returns the DID method (e.g., "web", "key", "peer").
    #[must_use]
    pub fn method(&self) -> Option<&str> {
        self.0
            .strip_prefix("did:")
            .and_then(|s| s.split(':').next())
    }

    /// Returns whether this is a did:key DID.
    #[must_use]
    pub fn is_did_key(&self) -> bool {
        self.0.starts_with("did:key:")
    }

    /// Returns whether this is a did:web DID.
    #[must_use]
    pub fn is_did_web(&self) -> bool {
        self.0.starts_with("did:web:")
    }

    /// Returns whether this is a did:peer DID.
    #[must_use]
    pub fn is_did_peer(&self) -> bool {
        self.0.starts_with("did:peer:")
    }

    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Did {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Did {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Did {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Parsed components of a DID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DidComponents {
    /// The DID method (e.g., "web", "key", "peer").
    pub method: String,
    /// The method-specific identifier.
    pub method_specific_id: String,
    /// Optional path component.
    pub path: Option<String>,
    /// Optional query component.
    pub query: Option<String>,
    /// Optional fragment component.
    pub fragment: Option<String>,
}

/// Parse the suffix after `did:method:` into method_specific_id and optional components.
fn parse_did_suffix(s: &str) -> (&str, Option<String>, Option<String>, Option<String>) {
    // Find fragment first
    let (rest, fragment) = match s.split_once('#') {
        Some((before, frag)) => (before, Some(frag.to_string())),
        None => (s, None),
    };

    // Find query
    let (rest, query) = match rest.split_once('?') {
        Some((before, q)) => (before, Some(q.to_string())),
        None => (rest, None),
    };

    // Find path (for did:web)
    let (method_specific_id, path) = match rest.split_once('/') {
        Some((id, p)) => (id, Some(format!("/{p}"))),
        None => (rest, None),
    };

    (method_specific_id, path, query, fragment)
}

/// A DID Document as defined by W3C DID Core 1.0.
///
/// The DID document contains metadata and cryptographic keys
/// associated with the DID subject.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidDocument {
    /// JSON-LD context (required for LD processing).
    #[serde(rename = "@context")]
    pub context: DidContext,

    /// The DID subject of this document.
    pub id: String,

    /// Optional controller(s) of this DID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controller: Option<ControllerSet>,

    /// Verification methods (public keys, etc.).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub verification_method: Vec<VerificationMethod>,

    /// Authentication verification relationships.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authentication: Vec<VerificationRelationship>,

    /// Assertion method verification relationships.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assertion_method: Vec<VerificationRelationship>,

    /// Key agreement verification relationships.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub key_agreement: Vec<VerificationRelationship>,

    /// Capability invocation verification relationships.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capability_invocation: Vec<VerificationRelationship>,

    /// Capability delegation verification relationships.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capability_delegation: Vec<VerificationRelationship>,

    /// Service endpoints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub service: Vec<ServiceEndpoint>,

    /// Also known as (alternative identifiers).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub also_known_as: Vec<String>,
}

impl DidDocument {
    /// Find a verification method by ID (full or fragment).
    #[must_use]
    pub fn find_verification_method(&self, id: &str) -> Option<&VerificationMethod> {
        self.verification_method.iter().find(|vm| {
            vm.id == id
                || vm.id.ends_with(&format!("#{id}"))
                || id.ends_with(&format!("#{}", vm.id.split('#').next_back().unwrap_or("")))
        })
    }

    /// Get verification methods for a specific purpose.
    #[must_use]
    pub fn verification_methods_for_purpose(
        &self,
        purpose: VerificationPurpose,
    ) -> Vec<&VerificationMethod> {
        let relationships = match purpose {
            VerificationPurpose::Authentication => &self.authentication,
            VerificationPurpose::AssertionMethod => &self.assertion_method,
            VerificationPurpose::KeyAgreement => &self.key_agreement,
            VerificationPurpose::CapabilityInvocation => &self.capability_invocation,
            VerificationPurpose::CapabilityDelegation => &self.capability_delegation,
        };

        relationships
            .iter()
            .filter_map(|rel| match rel {
                VerificationRelationship::Reference(id) => self.find_verification_method(id),
                VerificationRelationship::Embedded(vm) => Some(vm),
            })
            .collect()
    }

    /// Get the first verification method for assertion (signing).
    #[must_use]
    pub fn assertion_key(&self) -> Option<&VerificationMethod> {
        self.verification_methods_for_purpose(VerificationPurpose::AssertionMethod)
            .into_iter()
            .next()
    }

    /// Get the first verification method for authentication.
    #[must_use]
    pub fn authentication_key(&self) -> Option<&VerificationMethod> {
        self.verification_methods_for_purpose(VerificationPurpose::Authentication)
            .into_iter()
            .next()
    }

    /// Find a service endpoint by type.
    #[must_use]
    pub fn find_service_by_type(&self, service_type: &str) -> Option<&ServiceEndpoint> {
        self.service.iter().find(|s| s.service_type == service_type)
    }
}

/// JSON-LD context for DID documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DidContext {
    /// Single context URL.
    Single(String),
    /// Multiple context URLs/objects.
    Multiple(Vec<Value>),
}

impl Default for DidContext {
    fn default() -> Self {
        Self::Single("https://www.w3.org/ns/did/v1".to_string())
    }
}

/// Controller specification (single or multiple).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ControllerSet {
    /// Single controller DID.
    Single(String),
    /// Multiple controller DIDs.
    Multiple(Vec<String>),
}

/// A verification method (cryptographic key or other mechanism).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationMethod {
    /// Unique identifier for this verification method.
    pub id: String,

    /// The type of verification method.
    #[serde(rename = "type")]
    pub method_type: String,

    /// The DID that controls this verification method.
    pub controller: String,

    /// Public key in JWK format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_jwk: Option<Value>,

    /// Public key in multibase encoding.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_multibase: Option<String>,

    /// Public key in base58 encoding (deprecated but still used).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_base58: Option<String>,
}

impl VerificationMethod {
    /// Extract the raw public key bytes (tries all known formats).
    ///
    /// # Errors
    /// Returns an error if the key format is not recognized or decoding fails.
    pub fn public_key_bytes(&self) -> Result<Vec<u8>, String> {
        // Try JWK first
        if let Some(ref jwk) = self.public_key_jwk {
            return extract_key_from_jwk(jwk);
        }

        // Try multibase
        if let Some(ref mb) = self.public_key_multibase {
            return decode_multibase(mb);
        }

        // Try base58
        if let Some(ref b58) = self.public_key_base58 {
            return bs58::decode(b58)
                .into_vec()
                .map_err(|e| format!("Invalid base58: {e}"));
        }

        Err("No recognized public key format".to_string())
    }
}

/// Extract raw key bytes from a JWK.
fn extract_key_from_jwk(jwk: &Value) -> Result<Vec<u8>, String> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

    let kty = jwk.get("kty").and_then(|v| v.as_str()).unwrap_or("");
    let crv = jwk.get("crv").and_then(|v| v.as_str()).unwrap_or("");

    match (kty, crv) {
        // Ed25519 (OKP curve)
        ("OKP", "Ed25519") => {
            let x = jwk
                .get("x")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'x' in Ed25519 JWK")?;
            URL_SAFE_NO_PAD
                .decode(x)
                .map_err(|e| format!("Invalid x: {e}"))
        }
        // X25519 (OKP curve for key agreement)
        ("OKP", "X25519") => {
            let x = jwk
                .get("x")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'x' in X25519 JWK")?;
            URL_SAFE_NO_PAD
                .decode(x)
                .map_err(|e| format!("Invalid x: {e}"))
        }
        // P-256 (EC curve)
        ("EC", "P-256") => {
            let x = jwk
                .get("x")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'x' in P-256 JWK")?;
            let y = jwk
                .get("y")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'y' in P-256 JWK")?;
            let x_bytes = URL_SAFE_NO_PAD
                .decode(x)
                .map_err(|e| format!("Invalid x: {e}"))?;
            let y_bytes = URL_SAFE_NO_PAD
                .decode(y)
                .map_err(|e| format!("Invalid y: {e}"))?;
            // Return uncompressed format: 0x04 || x || y
            let mut result = vec![0x04];
            result.extend(x_bytes);
            result.extend(y_bytes);
            Ok(result)
        }
        _ => Err(format!("Unsupported JWK type: kty={kty}, crv={crv}")),
    }
}

/// Decode a multibase-encoded string.
fn decode_multibase(s: &str) -> Result<Vec<u8>, String> {
    if s.is_empty() {
        return Err("Empty multibase string".to_string());
    }

    let prefix = s.chars().next().unwrap();
    let data = &s[1..];

    match prefix {
        'z' => bs58::decode(data)
            .into_vec()
            .map_err(|e| format!("Invalid base58btc: {e}")),
        'f' => hex::decode(data).map_err(|e| format!("Invalid hex: {e}")),
        'u' => {
            use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
            URL_SAFE_NO_PAD
                .decode(data)
                .map_err(|e| format!("Invalid base64url: {e}"))
        }
        'm' => {
            use base64::{engine::general_purpose::STANDARD, Engine};
            STANDARD
                .decode(data)
                .map_err(|e| format!("Invalid base64: {e}"))
        }
        _ => Err(format!("Unsupported multibase prefix: {prefix}")),
    }
}

/// Verification relationship (can be reference or embedded).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VerificationRelationship {
    /// Reference to a verification method by ID.
    Reference(String),
    /// Embedded verification method.
    Embedded(VerificationMethod),
}

/// Purpose of a verification method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationPurpose {
    /// Proving control of the DID.
    Authentication,
    /// Making verifiable claims.
    AssertionMethod,
    /// Establishing secure communication.
    KeyAgreement,
    /// Invoking capabilities.
    CapabilityInvocation,
    /// Delegating capabilities.
    CapabilityDelegation,
}

/// A service endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceEndpoint {
    /// Unique identifier for this service.
    pub id: String,

    /// Service type.
    #[serde(rename = "type")]
    pub service_type: String,

    /// Service endpoint URL or object.
    pub service_endpoint: ServiceEndpointValue,
}

/// Service endpoint value (URL or complex object).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServiceEndpointValue {
    /// Simple URL string.
    Url(String),
    /// Multiple URLs.
    Urls(Vec<String>),
    /// Complex endpoint object.
    Object(Value),
}

/// Resolution metadata returned alongside a DID document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolutionMetadata {
    /// Content type of the returned document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    /// Error code if resolution failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// When the document was retrieved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieved: Option<DateTime<Utc>>,

    /// Time taken for resolution in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,

    /// Whether the result came from cache.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached: Option<bool>,
}

impl Default for ResolutionMetadata {
    fn default() -> Self {
        Self {
            content_type: Some("application/did+ld+json".to_string()),
            error: None,
            retrieved: Some(Utc::now()),
            duration_ms: None,
            cached: Some(false),
        }
    }
}

/// Complete resolution result.
#[derive(Debug, Clone)]
pub struct ResolutionResult {
    /// The resolved DID document (if successful).
    pub document: Option<DidDocument>,
    /// Resolution metadata.
    pub metadata: ResolutionMetadata,
}

impl ResolutionResult {
    /// Create a successful resolution result.
    #[must_use]
    pub fn success(document: DidDocument, duration_ms: u64, cached: bool) -> Self {
        Self {
            document: Some(document),
            metadata: ResolutionMetadata {
                content_type: Some("application/did+ld+json".to_string()),
                error: None,
                retrieved: Some(Utc::now()),
                duration_ms: Some(duration_ms),
                cached: Some(cached),
            },
        }
    }

    /// Create a failed resolution result.
    #[must_use]
    pub fn error(error: impl Into<String>) -> Self {
        Self {
            document: None,
            metadata: ResolutionMetadata {
                content_type: None,
                error: Some(error.into()),
                retrieved: Some(Utc::now()),
                duration_ms: None,
                cached: Some(false),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_did_key() {
        let components =
            Did::parse("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK").unwrap();
        assert_eq!(components.method, "key");
        assert!(components.method_specific_id.starts_with("z6Mkh"));
        assert!(components.path.is_none());
        assert!(components.query.is_none());
        assert!(components.fragment.is_none());
    }

    #[test]
    fn parse_did_web_simple() {
        let components = Did::parse("did:web:example.com").unwrap();
        assert_eq!(components.method, "web");
        assert_eq!(components.method_specific_id, "example.com");
    }

    #[test]
    fn parse_did_web_with_path() {
        let components = Did::parse("did:web:example.com:user:alice").unwrap();
        assert_eq!(components.method, "web");
        assert_eq!(components.method_specific_id, "example.com:user:alice");
    }

    #[test]
    fn parse_did_with_fragment() {
        let components = Did::parse("did:key:z6Mkh123#key-1").unwrap();
        assert_eq!(components.method, "key");
        assert_eq!(components.method_specific_id, "z6Mkh123");
        assert_eq!(components.fragment, Some("key-1".to_string()));
    }

    #[test]
    fn invalid_did_rejected() {
        assert!(Did::parse("not-a-did").is_none());
        assert!(Did::parse("did:").is_none());
        assert!(Did::parse("did::id").is_none());
        assert!(Did::parse("did:UPPER:id").is_none());
    }

    #[test]
    fn did_method_detection() {
        assert!(Did::from("did:key:z6Mkh123").is_did_key());
        assert!(Did::from("did:web:example.com").is_did_web());
        assert!(Did::from("did:peer:2.Ez6L...").is_did_peer());
    }

    #[test]
    fn decode_multibase_base58btc() {
        // 'z' prefix = base58btc
        let bytes = decode_multibase("z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK").unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn verification_method_extraction() {
        let doc = DidDocument {
            context: DidContext::default(),
            id: "did:key:z6Mkh123".to_string(),
            controller: None,
            verification_method: vec![VerificationMethod {
                id: "did:key:z6Mkh123#key-1".to_string(),
                method_type: "Ed25519VerificationKey2020".to_string(),
                controller: "did:key:z6Mkh123".to_string(),
                public_key_jwk: Some(serde_json::json!({
                    "kty": "OKP",
                    "crv": "Ed25519",
                    "x": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
                })),
                public_key_multibase: None,
                public_key_base58: None,
            }],
            authentication: vec![VerificationRelationship::Reference(
                "did:key:z6Mkh123#key-1".to_string(),
            )],
            assertion_method: vec![VerificationRelationship::Reference(
                "did:key:z6Mkh123#key-1".to_string(),
            )],
            key_agreement: vec![],
            capability_invocation: vec![],
            capability_delegation: vec![],
            service: vec![],
            also_known_as: vec![],
        };

        let auth_key = doc.authentication_key().unwrap();
        assert_eq!(auth_key.method_type, "Ed25519VerificationKey2020");

        let assertion_key = doc.assertion_key().unwrap();
        assert_eq!(assertion_key.id, "did:key:z6Mkh123#key-1");
    }
}
