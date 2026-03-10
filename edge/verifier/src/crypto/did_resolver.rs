//! DID resolution for edge verifier.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// DID resolution errors.
#[derive(Debug, Error)]
pub enum ResolverError {
    #[error("DID not found: {0}")]
    NotFound(String),

    #[error("Invalid DID format: {0}")]
    InvalidFormat(String),

    #[error("Resolution failed: {0}")]
    ResolutionFailed(String),

    #[error("Unsupported DID method: {0}")]
    UnsupportedMethod(String),

    #[error("Cache miss and offline")]
    CacheMissOffline,
}

/// DID Document structure (simplified).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidDocument {
    /// The DID.
    pub id: String,

    /// Verification methods.
    #[serde(default)]
    pub verification_method: Vec<VerificationMethod>,

    /// Authentication methods.
    #[serde(default)]
    pub authentication: Vec<String>,

    /// Assertion methods.
    #[serde(default, rename = "assertionMethod")]
    pub assertion_method: Vec<String>,

    /// Service endpoints.
    #[serde(default)]
    pub service: Vec<Service>,
}

/// Verification method in DID Document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    /// Method ID.
    pub id: String,

    /// Method type.
    #[serde(rename = "type")]
    pub method_type: String,

    /// Controller DID.
    pub controller: String,

    /// Public key in JWK format.
    #[serde(rename = "publicKeyJwk", skip_serializing_if = "Option::is_none")]
    pub public_key_jwk: Option<serde_json::Value>,

    /// Public key in multibase format.
    #[serde(rename = "publicKeyMultibase", skip_serializing_if = "Option::is_none")]
    pub public_key_multibase: Option<String>,
}

/// Service endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    /// Service ID.
    pub id: String,

    /// Service type.
    #[serde(rename = "type")]
    pub service_type: String,

    /// Service endpoint URL.
    #[serde(rename = "serviceEndpoint")]
    pub service_endpoint: String,
}

/// Cached DID document.
struct CachedDocument {
    document: DidDocument,
    cached_at: Instant,
}

/// DID resolver with caching.
pub struct DidResolver {
    /// Cache of resolved DIDs.
    cache: Arc<RwLock<HashMap<String, CachedDocument>>>,

    /// Cache TTL.
    cache_ttl: Duration,

    /// Whether offline mode is enabled.
    offline: Arc<RwLock<bool>>,
}

impl DidResolver {
    /// Create a new resolver.
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(14400), // 4 hours
            offline: Arc::new(RwLock::new(false)),
        }
    }

    /// Set cache TTL.
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    /// Set offline mode.
    pub async fn set_offline(&self, offline: bool) {
        let mut is_offline = self.offline.write().await;
        *is_offline = offline;
        if offline {
            warn!("DID resolver entering offline mode");
        } else {
            info!("DID resolver exiting offline mode");
        }
    }

    /// Resolve a DID to its document.
    pub async fn resolve(&self, did: &str) -> Result<DidDocument, ResolverError> {
        // Check cache first
        if let Some(doc) = self.get_cached(did).await {
            debug!("DID cache hit: {}", did);
            return Ok(doc);
        }

        let is_offline = *self.offline.read().await;
        if is_offline {
            return Err(ResolverError::CacheMissOffline);
        }

        // Parse DID method
        let method = Self::parse_method(did)?;

        // Resolve based on method
        let document = match method {
            "key" => self.resolve_did_key(did).await?,
            "web" => self.resolve_did_web(did).await?,
            "peer" => self.resolve_did_peer(did).await?,
            _ => return Err(ResolverError::UnsupportedMethod(method.to_string())),
        };

        // Cache the result
        self.cache_document(did, document.clone()).await;

        Ok(document)
    }

    fn parse_method(did: &str) -> Result<&str, ResolverError> {
        if !did.starts_with("did:") {
            return Err(ResolverError::InvalidFormat(did.to_string()));
        }

        let parts: Vec<&str> = did.split(':').collect();
        if parts.len() < 3 {
            return Err(ResolverError::InvalidFormat(did.to_string()));
        }

        Ok(parts[1])
    }

    async fn get_cached(&self, did: &str) -> Option<DidDocument> {
        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(did) {
            if cached.cached_at.elapsed() < self.cache_ttl {
                return Some(cached.document.clone());
            }
        }
        None
    }

    async fn cache_document(&self, did: &str, document: DidDocument) {
        let mut cache = self.cache.write().await;
        cache.insert(
            did.to_string(),
            CachedDocument {
                document,
                cached_at: Instant::now(),
            },
        );
    }

    /// Resolve did:key.
    async fn resolve_did_key(&self, did: &str) -> Result<DidDocument, ResolverError> {
        // did:key:<multibase-encoded-public-key>
        let parts: Vec<&str> = did.split(':').collect();
        if parts.len() != 3 {
            return Err(ResolverError::InvalidFormat(did.to_string()));
        }

        let multibase_key = parts[2];

        // Determine key type from multibase prefix
        // z6Mk... = Ed25519, zDn... = P-256
        let (method_type, public_key_multibase) = if multibase_key.starts_with("z6Mk") {
            ("Ed25519VerificationKey2020", multibase_key)
        } else if multibase_key.starts_with("zDn") {
            ("EcdsaSecp256r1VerificationKey2019", multibase_key)
        } else {
            ("JsonWebKey2020", multibase_key)
        };

        let verification_method_id = format!("{}#{}", did, multibase_key);

        Ok(DidDocument {
            id: did.to_string(),
            verification_method: vec![VerificationMethod {
                id: verification_method_id.clone(),
                method_type: method_type.to_string(),
                controller: did.to_string(),
                public_key_jwk: None,
                public_key_multibase: Some(public_key_multibase.to_string()),
            }],
            authentication: vec![verification_method_id.clone()],
            assertion_method: vec![verification_method_id],
            service: Vec::new(),
        })
    }

    /// Resolve did:web.
    async fn resolve_did_web(&self, did: &str) -> Result<DidDocument, ResolverError> {
        // did:web:<domain>[:path...]
        let parts: Vec<&str> = did.split(':').collect();
        if parts.len() < 3 {
            return Err(ResolverError::InvalidFormat(did.to_string()));
        }

        let domain = parts[2].replace("%3A", ":");
        let path = if parts.len() > 3 {
            parts[3..].join("/")
        } else {
            ".well-known".to_string()
        };

        let url = format!("https://{}/{}/did.json", domain, path);

        // In production, this would fetch from the URL
        // For now, return a stub document
        warn!("did:web resolution stubbed for: {}", url);

        Ok(DidDocument {
            id: did.to_string(),
            verification_method: vec![VerificationMethod {
                id: format!("{}#key-1", did),
                method_type: "JsonWebKey2020".to_string(),
                controller: did.to_string(),
                public_key_jwk: Some(serde_json::json!({
                    "kty": "EC",
                    "crv": "P-256",
                    "x": "stub",
                    "y": "stub"
                })),
                public_key_multibase: None,
            }],
            authentication: vec![format!("{}#key-1", did)],
            assertion_method: vec![format!("{}#key-1", did)],
            service: Vec::new(),
        })
    }

    /// Resolve did:peer.
    async fn resolve_did_peer(&self, did: &str) -> Result<DidDocument, ResolverError> {
        // did:peer:<numalgo><encoded-document>
        // Simplified implementation for numalgo 0 (inception key only)

        let parts: Vec<&str> = did.split(':').collect();
        if parts.len() != 3 {
            return Err(ResolverError::InvalidFormat(did.to_string()));
        }

        let peer_specific = parts[2];
        if peer_specific.is_empty() {
            return Err(ResolverError::InvalidFormat(did.to_string()));
        }

        let numalgo = peer_specific.chars().next().unwrap();

        match numalgo {
            '0' => {
                // numalgo 0: inception key only
                let multibase_key = &peer_specific[1..];
                Ok(DidDocument {
                    id: did.to_string(),
                    verification_method: vec![VerificationMethod {
                        id: format!("{}#key-1", did),
                        method_type: "Ed25519VerificationKey2020".to_string(),
                        controller: did.to_string(),
                        public_key_jwk: None,
                        public_key_multibase: Some(multibase_key.to_string()),
                    }],
                    authentication: vec![format!("{}#key-1", did)],
                    assertion_method: vec![format!("{}#key-1", did)],
                    service: Vec::new(),
                })
            }
            '2' => {
                // numalgo 2: multiple keys and services encoded
                // Stub for now
                warn!("did:peer numalgo 2 resolution stubbed");
                Ok(DidDocument {
                    id: did.to_string(),
                    verification_method: vec![],
                    authentication: vec![],
                    assertion_method: vec![],
                    service: vec![],
                })
            }
            _ => Err(ResolverError::UnsupportedMethod(format!(
                "did:peer numalgo {}",
                numalgo
            ))),
        }
    }

    /// Preload DIDs into cache (for offline preparation).
    pub async fn preload(&self, dids: &[&str]) -> Vec<Result<(), ResolverError>> {
        let mut results = Vec::new();
        for did in dids {
            let result = self.resolve(did).await.map(|_| ());
            results.push(result);
        }
        results
    }

    /// Clear the cache.
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("DID cache cleared");
    }

    /// Get cache size.
    pub async fn cache_size(&self) -> usize {
        self.cache.read().await.len()
    }
}

impl Default for DidResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resolve_did_key_ed25519() {
        let resolver = DidResolver::new();
        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";

        let doc = resolver.resolve(did).await.unwrap();
        assert_eq!(doc.id, did);
        assert_eq!(doc.verification_method.len(), 1);
        assert_eq!(
            doc.verification_method[0].method_type,
            "Ed25519VerificationKey2020"
        );
    }

    #[tokio::test]
    async fn test_resolve_did_web() {
        let resolver = DidResolver::new();
        let did = "did:web:sahi.my";

        let doc = resolver.resolve(did).await.unwrap();
        assert_eq!(doc.id, did);
    }

    #[tokio::test]
    async fn test_resolve_did_peer_numalgo0() {
        let resolver = DidResolver::new();
        let did = "did:peer:0z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";

        let doc = resolver.resolve(did).await.unwrap();
        assert_eq!(doc.id, did);
    }

    #[tokio::test]
    async fn test_invalid_did() {
        let resolver = DidResolver::new();

        let result = resolver.resolve("not-a-did").await;
        assert!(matches!(result, Err(ResolverError::InvalidFormat(_))));

        let result = resolver.resolve("did:").await;
        assert!(matches!(result, Err(ResolverError::InvalidFormat(_))));
    }

    #[tokio::test]
    async fn test_unsupported_method() {
        let resolver = DidResolver::new();
        let did = "did:unknown:123";

        let result = resolver.resolve(did).await;
        assert!(matches!(result, Err(ResolverError::UnsupportedMethod(_))));
    }

    #[tokio::test]
    async fn test_cache() {
        let resolver = DidResolver::new();
        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";

        // First resolve
        let _ = resolver.resolve(did).await.unwrap();
        assert_eq!(resolver.cache_size().await, 1);

        // Second resolve should hit cache
        let _ = resolver.resolve(did).await.unwrap();
        assert_eq!(resolver.cache_size().await, 1);

        // Clear cache
        resolver.clear_cache().await;
        assert_eq!(resolver.cache_size().await, 0);
    }

    #[tokio::test]
    async fn test_offline_mode() {
        let resolver = DidResolver::new();
        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";

        // Pre-cache
        let _ = resolver.resolve(did).await.unwrap();

        // Enable offline mode
        resolver.set_offline(true).await;

        // Cached DID should still resolve
        let _ = resolver.resolve(did).await.unwrap();

        // Uncached DID should fail
        let result = resolver.resolve("did:key:z6MkOther").await;
        assert!(matches!(result, Err(ResolverError::CacheMissOffline)));
    }
}
