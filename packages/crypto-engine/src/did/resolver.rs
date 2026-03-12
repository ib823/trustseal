//! Universal DID resolver.
//!
//! Resolves DIDs across multiple methods with caching.
//!
//! Supported methods:
//! - `did:key` — Pure cryptographic, no network (Phase 1)
//! - `did:web` — HTTP-based resolution (Phase 1)
//! - `did:peer` — Pairwise DIDs (Phase 2 stub)

use std::sync::Arc;
use std::time::{Duration, Instant};

use tracing::{debug, instrument};

/// Convert duration to milliseconds, clamping to u64::MAX.
#[allow(clippy::cast_possible_truncation)]
fn duration_to_millis(duration: Duration) -> u64 {
    let millis = duration.as_millis();
    if millis > u128::from(u64::MAX) {
        u64::MAX
    } else {
        millis as u64
    }
}

use super::cache::DidCache;
use super::did_key;
#[cfg(feature = "http")]
use super::did_web;
use super::did_web::DidWebConfig;
use super::types::{Did, DidDocument, ResolutionResult, VerificationPurpose};
use crate::error::CryptoError;

/// Configuration for the DID resolver.
#[derive(Debug, Clone)]
pub struct ResolverConfig {
    /// Configuration for did:web resolution.
    pub did_web: DidWebConfig,
    /// Cache TTL for did:key (short since it's deterministic).
    pub did_key_ttl: Duration,
    /// Cache TTL for did:web.
    pub did_web_ttl: Duration,
    /// Whether to use caching.
    pub enable_cache: bool,
    /// Cache capacity.
    pub cache_capacity: usize,
}

impl Default for ResolverConfig {
    fn default() -> Self {
        Self {
            did_web: DidWebConfig::default(),
            did_key_ttl: Duration::from_secs(300), // 5 min per spec (L1 TTL)
            did_web_ttl: Duration::from_secs(3600), // 1 hour per spec
            enable_cache: true,
            cache_capacity: 100,
        }
    }
}

/// Universal DID resolver.
///
/// Resolves DIDs across multiple methods with automatic caching.
///
/// # Example
///
/// ```ignore
/// let resolver = DidResolver::new(ResolverConfig::default());
///
/// // Resolve a did:key (no network call)
/// let result = resolver.resolve("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK")?;
///
/// // Resolve a did:web (HTTP call, cached)
/// let result = resolver.resolve("did:web:issuer.sahi.my")?;
/// ```
pub struct DidResolver {
    config: ResolverConfig,
    cache: Arc<DidCache>,
}

impl DidResolver {
    /// Create a new resolver with the given configuration.
    #[must_use]
    pub fn new(config: ResolverConfig) -> Self {
        let cache_ttl = config.did_web_ttl; // Use did:web TTL as default
        let cache = Arc::new(DidCache::new(config.cache_capacity, cache_ttl));

        Self { config, cache }
    }

    /// Create a resolver with default configuration.
    #[must_use]
    pub fn default_config() -> Self {
        Self::new(ResolverConfig::default())
    }

    /// Resolve a DID to its document.
    ///
    /// # Arguments
    /// * `did` - The DID to resolve (e.g., "did:key:z6Mkh..." or "did:web:example.com")
    ///
    /// # Returns
    /// A `ResolutionResult` containing the document and metadata.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The DID format is invalid
    /// - The DID method is unsupported
    /// - Resolution fails (network error for did:web, etc.)
    #[instrument(skip(self), fields(did = %did))]
    pub fn resolve(&self, did: &str) -> Result<ResolutionResult, CryptoError> {
        let start = Instant::now();

        // Normalize DID (remove fragment for caching)
        let cache_key = did.split('#').next().unwrap_or(did);

        // Check cache first
        if self.config.enable_cache {
            if let Some(doc) = self.cache.get(cache_key) {
                debug!("Cache hit for {}", cache_key);
                return Ok(ResolutionResult::success(
                    doc,
                    duration_to_millis(start.elapsed()),
                    true,
                ));
            }
        }

        // Parse DID to determine method
        let parsed = Did::from(did);
        let method = parsed
            .method()
            .ok_or_else(|| CryptoError::Internal(format!("Invalid DID format: {did}")))?;

        // Resolve based on method
        let (doc, ttl) = match method {
            "key" => {
                let doc = did_key::resolve(did).map_err(|e| {
                    CryptoError::Internal(format!("did:key resolution failed: {e}"))
                })?;
                (doc, self.config.did_key_ttl)
            }
            "web" => {
                // For did:web, we need async HTTP. In sync context, this won't work.
                // Provide a stub that returns an error, or use validate_document with pre-fetched data.
                return Err(CryptoError::Internal(
                    "did:web resolution requires async context. Use resolve_web_async or provide pre-fetched document.".to_string()
                ));
            }
            "peer" => {
                return Err(CryptoError::Internal(
                    "did:peer resolution not implemented (Phase 2)".to_string(),
                ));
            }
            _ => {
                return Err(CryptoError::Internal(format!(
                    "Unsupported DID method: {method}"
                )));
            }
        };

        // Cache the result
        if self.config.enable_cache {
            self.cache.insert_with_ttl(cache_key, doc.clone(), ttl);
        }

        let duration_ms = duration_to_millis(start.elapsed());
        debug!(duration_ms, "Resolved {}", cache_key);

        Ok(ResolutionResult::success(doc, duration_ms, false))
    }

    /// Resolve a did:key synchronously.
    ///
    /// This is a convenience method that doesn't require async.
    ///
    /// # Errors
    /// Returns an error if the DID is not a valid did:key.
    pub fn resolve_key(&self, did: &str) -> Result<ResolutionResult, CryptoError> {
        if !did.starts_with("did:key:") {
            return Err(CryptoError::Internal(format!(
                "Expected did:key, got: {did}"
            )));
        }
        self.resolve(did)
    }

    /// Register a pre-resolved DID document.
    ///
    /// Use this to add documents that were fetched externally (e.g., by a background job).
    pub fn register_document(&self, did: &str, document: DidDocument, ttl: Duration) {
        let cache_key = did.split('#').next().unwrap_or(did);
        self.cache.insert_with_ttl(cache_key, document, ttl);
    }

    /// Invalidate a cached DID document.
    pub fn invalidate(&self, did: &str) {
        let cache_key = did.split('#').next().unwrap_or(did);
        self.cache.remove(cache_key);
    }

    /// Clear the entire cache.
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Get cache statistics.
    #[must_use]
    pub fn cache_stats(&self) -> super::cache::CacheStats {
        self.cache.stats()
    }

    /// Extract a public key for a specific purpose from a resolved document.
    ///
    /// # Arguments
    /// * `did` - The DID to resolve
    /// * `purpose` - The verification purpose (authentication, assertion, etc.)
    ///
    /// # Returns
    /// The raw public key bytes.
    ///
    /// # Errors
    /// Returns an error if resolution fails or no key is found for the purpose.
    pub fn resolve_key_for_purpose(
        &self,
        did: &str,
        purpose: VerificationPurpose,
    ) -> Result<Vec<u8>, CryptoError> {
        let result = self.resolve(did)?;
        let doc = result
            .document
            .ok_or_else(|| CryptoError::Internal("Resolution returned no document".to_string()))?;

        let methods = doc.verification_methods_for_purpose(purpose);
        let method = methods.first().ok_or_else(|| {
            CryptoError::Internal(format!("No verification method for {purpose:?}"))
        })?;

        method
            .public_key_bytes()
            .map_err(|e| CryptoError::Internal(format!("Failed to extract public key: {e}")))
    }

    /// Resolve issuer public key for SD-JWT verification.
    ///
    /// This is a convenience method for the common case of verifying a credential.
    ///
    /// # Errors
    /// Returns an error if resolution fails or no assertion key is found.
    pub fn resolve_issuer_key(&self, issuer_did: &str) -> Result<Vec<u8>, CryptoError> {
        self.resolve_key_for_purpose(issuer_did, VerificationPurpose::AssertionMethod)
    }
}

impl Default for DidResolver {
    fn default() -> Self {
        Self::default_config()
    }
}

/// Async DID resolver for contexts where async is available.
///
/// This is a thin wrapper that provides async methods.
pub struct AsyncDidResolver {
    inner: DidResolver,
}

impl AsyncDidResolver {
    /// Create a new async resolver.
    #[must_use]
    pub fn new(config: ResolverConfig) -> Self {
        Self {
            inner: DidResolver::new(config),
        }
    }

    /// Resolve a DID asynchronously.
    ///
    /// For did:key, this is synchronous under the hood.
    /// For did:web, this makes an HTTP request.
    ///
    /// # Errors
    /// Returns an error if resolution fails.
    #[instrument(skip(self), fields(did = %did))]
    pub async fn resolve(&self, did: &str) -> Result<ResolutionResult, CryptoError> {
        let start = Instant::now();
        let cache_key = did.split('#').next().unwrap_or(did);

        // Check cache
        if self.inner.config.enable_cache {
            if let Some(doc) = self.inner.cache.get(cache_key) {
                debug!("Cache hit for {}", cache_key);
                return Ok(ResolutionResult::success(
                    doc,
                    duration_to_millis(start.elapsed()),
                    true,
                ));
            }
        }

        let parsed = Did::from(did);
        let method = parsed
            .method()
            .ok_or_else(|| CryptoError::Internal(format!("Invalid DID format: {did}")))?;

        let (doc, ttl) = match method {
            "key" => {
                let doc = did_key::resolve(did).map_err(|e| {
                    CryptoError::Internal(format!("did:key resolution failed: {e}"))
                })?;
                (doc, self.inner.config.did_key_ttl)
            }
            "web" => {
                #[cfg(feature = "http")]
                {
                    let doc = did_web::resolve(did, &self.inner.config.did_web)
                        .await
                        .map_err(|e| {
                            CryptoError::Internal(format!("did:web resolution failed: {e}"))
                        })?;
                    (doc, self.inner.config.did_web_ttl)
                }
                #[cfg(not(feature = "http"))]
                {
                    return Err(CryptoError::Internal(
                        "did:web resolution requires 'http' feature".to_string(),
                    ));
                }
            }
            "peer" => {
                return Err(CryptoError::Internal(
                    "did:peer resolution not implemented (Phase 2)".to_string(),
                ));
            }
            _ => {
                return Err(CryptoError::Internal(format!(
                    "Unsupported DID method: {method}"
                )));
            }
        };

        // Cache the result
        if self.inner.config.enable_cache {
            self.inner
                .cache
                .insert_with_ttl(cache_key, doc.clone(), ttl);
        }

        let duration_ms = duration_to_millis(start.elapsed());
        debug!(duration_ms, "Resolved {}", cache_key);

        Ok(ResolutionResult::success(doc, duration_ms, false))
    }

    /// Get the underlying sync resolver.
    #[must_use]
    pub fn inner(&self) -> &DidResolver {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_did_key() {
        let resolver = DidResolver::default();

        let result = resolver
            .resolve("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK")
            .unwrap();

        let doc = result.document.unwrap();
        assert!(doc.id.starts_with("did:key:z6Mkh"));
        assert!(!result.metadata.cached.unwrap_or(true));
    }

    #[test]
    fn resolve_did_key_cached() {
        let resolver = DidResolver::default();
        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";

        // First call - not cached
        let result1 = resolver.resolve(did).unwrap();
        assert!(!result1.metadata.cached.unwrap_or(true));

        // Second call - should be cached
        let result2 = resolver.resolve(did).unwrap();
        assert!(result2.metadata.cached.unwrap_or(false));
    }

    #[test]
    fn resolve_did_key_for_assertion() {
        let resolver = DidResolver::default();
        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";

        let key = resolver
            .resolve_key_for_purpose(did, VerificationPurpose::AssertionMethod)
            .unwrap();

        assert!(!key.is_empty());
    }

    #[test]
    fn resolve_issuer_key() {
        let resolver = DidResolver::default();
        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";

        let key = resolver.resolve_issuer_key(did).unwrap();
        assert_eq!(key.len(), 32); // Ed25519 public key
    }

    #[test]
    fn resolve_did_web_sync_returns_error() {
        let resolver = DidResolver::default();

        let result = resolver.resolve("did:web:example.com");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("async"));
    }

    #[test]
    fn resolve_unsupported_method() {
        let resolver = DidResolver::default();

        let result = resolver.resolve("did:unknown:something");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported"));
    }

    #[test]
    fn resolve_invalid_did() {
        let resolver = DidResolver::default();

        let result = resolver.resolve("not-a-did");
        assert!(result.is_err());
    }

    #[test]
    fn register_and_retrieve_document() {
        let resolver = DidResolver::default();

        let doc =
            did_key::resolve("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK").unwrap();

        // Register under a different DID (simulating did:web)
        resolver.register_document("did:web:example.com", doc.clone(), Duration::from_secs(60));

        // Should be retrievable
        let cached = resolver.cache.get("did:web:example.com").unwrap();
        assert_eq!(cached.id, doc.id);
    }

    #[test]
    fn invalidate_removes_from_cache() {
        let resolver = DidResolver::default();
        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";

        // Populate cache
        let _ = resolver.resolve(did).unwrap();
        assert!(resolver.cache.get(did).is_some());

        // Invalidate
        resolver.invalidate(did);
        assert!(resolver.cache.get(did).is_none());
    }

    #[test]
    fn cache_stats_tracked() {
        let resolver = DidResolver::default();
        let did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";

        // Miss
        let _ = resolver.resolve(did).unwrap();

        // Hit
        let _ = resolver.resolve(did).unwrap();
        let _ = resolver.resolve(did).unwrap();

        let stats = resolver.cache_stats();
        assert_eq!(stats.hits, 2);
        // Misses tracked by cache, not resolver
    }
}
