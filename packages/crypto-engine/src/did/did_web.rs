//! did:web method resolution.
//!
//! `did:web` resolves DIDs by fetching a DID document from a web server.
//! The DID encodes the domain (and optional path) where the document is hosted.
//!
//! Reference: <https://w3c-ccg.github.io/did-method-web/>

use std::time::Duration;

use super::types::DidDocument;

/// Configuration for did:web resolution.
#[derive(Debug, Clone)]
pub struct DidWebConfig {
    /// HTTP request timeout.
    pub timeout: Duration,
    /// Whether to allow HTTP (insecure) for localhost only.
    pub allow_localhost_http: bool,
    /// User-Agent header for requests.
    pub user_agent: String,
    /// Maximum document size in bytes.
    pub max_document_size: usize,
}

impl Default for DidWebConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            allow_localhost_http: true,
            user_agent: "Sahi-DID-Resolver/1.0".to_string(),
            max_document_size: 64 * 1024, // 64KB
        }
    }
}

/// Convert a did:web to the URL where the DID document should be fetched.
///
/// # Examples
/// - `did:web:example.com` → `https://example.com/.well-known/did.json`
/// - `did:web:example.com:user:alice` → `https://example.com/user/alice/did.json`
/// - `did:web:example.com%3A8443` → `https://example.com:8443/.well-known/did.json`
///
/// # Errors
/// Returns an error if the DID format is invalid.
pub fn did_to_url(did: &str) -> Result<String, String> {
    let method_specific_id = did
        .strip_prefix("did:web:")
        .ok_or("Invalid did:web: must start with 'did:web:'")?;

    // Remove any fragment
    let method_specific_id = method_specific_id
        .split('#')
        .next()
        .unwrap_or(method_specific_id);

    if method_specific_id.is_empty() {
        return Err("Invalid did:web: empty method-specific-id".to_string());
    }

    // Split on colons FIRST (before percent-decoding) to get segments
    // First segment is domain (may contain %3A for port)
    // Subsequent segments are path components
    let segments: Vec<&str> = method_specific_id.split(':').collect();

    // First segment is the domain (with optional percent-encoded port)
    let domain = percent_decode(segments[0])?;

    // Validate domain
    if domain.is_empty() {
        return Err("Invalid did:web: empty domain".to_string());
    }

    // Build the URL
    // Check if localhost BEFORE the port (e.g., "localhost:8080" → check "localhost")
    let host = domain.split(':').next().unwrap_or(&domain);
    let scheme = if is_localhost(host) { "http" } else { "https" };

    if segments.len() == 1 {
        // No path: use .well-known/did.json
        Ok(format!("{scheme}://{domain}/.well-known/did.json"))
    } else {
        // Has path: decode each segment and join with /
        let path_segments: Result<Vec<String>, String> =
            segments[1..].iter().map(|s| percent_decode(s)).collect();
        let path = path_segments?.join("/");
        Ok(format!("{scheme}://{domain}/{path}/did.json"))
    }
}

/// Percent-decode a string.
fn percent_decode(s: &str) -> Result<String, String> {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() != 2 {
                return Err(format!("Invalid percent encoding: %{hex}"));
            }
            let byte = u8::from_str_radix(&hex, 16)
                .map_err(|_| format!("Invalid hex in percent encoding: %{hex}"))?;
            result.push(byte as char);
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

/// Check if a domain is localhost.
fn is_localhost(domain: &str) -> bool {
    let host = domain.split(':').next().unwrap_or(domain);
    host == "localhost" || host == "127.0.0.1" || host == "::1"
}

/// Resolve a did:web by fetching the DID document from the web.
///
/// This is an async function that makes an HTTP(S) request.
///
/// # Arguments
/// * `did` - The did:web to resolve
/// * `config` - Resolution configuration
///
/// # Returns
/// The resolved DID document.
///
/// # Errors
/// Returns an error if:
/// - The DID format is invalid
/// - The HTTP request fails
/// - The response is not valid JSON
/// - The document doesn't match the DID
#[cfg(feature = "http")]
pub async fn resolve(did: &str, config: &DidWebConfig) -> Result<DidDocument, String> {
    let url = did_to_url(did)?;

    // Make HTTP request
    let client = reqwest::Client::builder()
        .timeout(config.timeout)
        .user_agent(&config.user_agent)
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let response = client
        .get(&url)
        .header("Accept", "application/did+ld+json, application/json")
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {} from {url}", response.status()));
    }

    // Check content length
    if let Some(len) = response.content_length() {
        if len > config.max_document_size as u64 {
            return Err(format!("DID document too large: {len} bytes"));
        }
    }

    // Parse response
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {e}"))?;

    if body.len() > config.max_document_size {
        return Err(format!("DID document too large: {} bytes", body.len()));
    }

    let doc: DidDocument =
        serde_json::from_str(&body).map_err(|e| format!("Invalid DID document JSON: {e}"))?;

    // Validate that the document ID matches the DID
    let expected_did = did.split('#').next().unwrap_or(did);
    if doc.id != expected_did {
        return Err(format!(
            "DID document ID mismatch: expected {expected_did}, got {}",
            doc.id
        ));
    }

    Ok(doc)
}

/// Resolve a did:web synchronously (for use in non-async contexts).
///
/// This creates a temporary tokio runtime to execute the async resolution.
///
/// # Errors
/// Returns an error if resolution fails.
#[cfg(feature = "http")]
pub fn resolve_sync(did: &str, config: &DidWebConfig) -> Result<DidDocument, String> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create runtime: {e}"))?
        .block_on(resolve(did, config))
}

/// Validate a pre-fetched DID document against a did:web.
///
/// Use this when you've already fetched the document and just need to validate it.
///
/// # Errors
/// Returns an error if validation fails.
pub fn validate_document(did: &str, doc: &DidDocument) -> Result<(), String> {
    let expected_did = did.split('#').next().unwrap_or(did);

    if doc.id != expected_did {
        return Err(format!(
            "DID document ID mismatch: expected {expected_did}, got {}",
            doc.id
        ));
    }

    // Ensure at least one verification method exists
    if doc.verification_method.is_empty() {
        return Err("DID document has no verification methods".to_string());
    }

    // Validate verification method references
    for rel in doc
        .authentication
        .iter()
        .chain(doc.assertion_method.iter())
        .chain(doc.key_agreement.iter())
        .chain(doc.capability_invocation.iter())
        .chain(doc.capability_delegation.iter())
    {
        if let super::types::VerificationRelationship::Reference(ref_id) = rel {
            // Reference must be resolvable
            if doc.find_verification_method(ref_id).is_none() {
                return Err(format!(
                    "Unresolvable verification method reference: {ref_id}"
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn did_to_url_simple_domain() {
        let url = did_to_url("did:web:example.com").unwrap();
        assert_eq!(url, "https://example.com/.well-known/did.json");
    }

    #[test]
    fn did_to_url_with_path() {
        let url = did_to_url("did:web:example.com:user:alice").unwrap();
        assert_eq!(url, "https://example.com/user/alice/did.json");
    }

    #[test]
    fn did_to_url_with_port() {
        let url = did_to_url("did:web:example.com%3A8443").unwrap();
        assert_eq!(url, "https://example.com:8443/.well-known/did.json");
    }

    #[test]
    fn did_to_url_localhost_uses_http() {
        let url = did_to_url("did:web:localhost").unwrap();
        assert_eq!(url, "http://localhost/.well-known/did.json");

        let url = did_to_url("did:web:localhost%3A8080").unwrap();
        assert_eq!(url, "http://localhost:8080/.well-known/did.json");
    }

    #[test]
    fn did_to_url_with_subdomain() {
        let url = did_to_url("did:web:api.issuer.sahi.my").unwrap();
        assert_eq!(url, "https://api.issuer.sahi.my/.well-known/did.json");
    }

    #[test]
    fn did_to_url_with_fragment_ignored() {
        let url = did_to_url("did:web:example.com#key-1").unwrap();
        assert_eq!(url, "https://example.com/.well-known/did.json");
    }

    #[test]
    fn did_to_url_invalid() {
        assert!(did_to_url("did:key:z6Mkh...").is_err());
        assert!(did_to_url("did:web:").is_err());
        assert!(did_to_url("not-a-did").is_err());
    }

    #[test]
    fn percent_decode_works() {
        assert_eq!(percent_decode("hello").unwrap(), "hello");
        assert_eq!(percent_decode("hello%20world").unwrap(), "hello world");
        assert_eq!(percent_decode("%3A").unwrap(), ":");
    }

    #[test]
    fn validate_document_success() {
        use super::super::types::*;

        let doc = DidDocument {
            context: DidContext::default(),
            id: "did:web:example.com".to_string(),
            controller: None,
            verification_method: vec![VerificationMethod {
                id: "did:web:example.com#key-1".to_string(),
                method_type: "Ed25519VerificationKey2020".to_string(),
                controller: "did:web:example.com".to_string(),
                public_key_jwk: Some(serde_json::json!({
                    "kty": "OKP",
                    "crv": "Ed25519",
                    "x": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
                })),
                public_key_multibase: None,
                public_key_base58: None,
            }],
            authentication: vec![VerificationRelationship::Reference(
                "did:web:example.com#key-1".to_string(),
            )],
            assertion_method: vec![],
            key_agreement: vec![],
            capability_invocation: vec![],
            capability_delegation: vec![],
            service: vec![],
            also_known_as: vec![],
        };

        assert!(validate_document("did:web:example.com", &doc).is_ok());
    }

    #[test]
    fn validate_document_id_mismatch() {
        use super::super::types::*;

        let doc = DidDocument {
            context: DidContext::default(),
            id: "did:web:other.com".to_string(),
            controller: None,
            verification_method: vec![VerificationMethod {
                id: "did:web:other.com#key-1".to_string(),
                method_type: "Ed25519VerificationKey2020".to_string(),
                controller: "did:web:other.com".to_string(),
                public_key_jwk: None,
                public_key_multibase: None,
                public_key_base58: None,
            }],
            authentication: vec![],
            assertion_method: vec![],
            key_agreement: vec![],
            capability_invocation: vec![],
            capability_delegation: vec![],
            service: vec![],
            also_known_as: vec![],
        };

        let result = validate_document("did:web:example.com", &doc);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("mismatch"));
    }
}
