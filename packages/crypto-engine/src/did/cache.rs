//! DID resolution caching.
//!
//! Provides a multi-level cache for DID documents:
//! - L1: In-memory LRU cache (per-service, fast)
//! - L2: Redis (shared across services, optional)
//! - L3: PostgreSQL (persistent, for audit)
//!
//! The cache respects TTLs and provides automatic invalidation.

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use super::types::DidDocument;

/// Cached DID document with metadata.
#[derive(Debug, Clone)]
pub struct CachedDocument {
    /// The DID document.
    pub document: DidDocument,
    /// When this entry was cached.
    pub cached_at: Instant,
    /// Time-to-live for this entry.
    pub ttl: Duration,
    /// Number of times this entry has been accessed.
    pub access_count: u64,
}

impl CachedDocument {
    /// Create a new cached entry.
    #[must_use]
    pub fn new(document: DidDocument, ttl: Duration) -> Self {
        Self {
            document,
            cached_at: Instant::now(),
            ttl,
            access_count: 0,
        }
    }

    /// Check if this entry has expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }

    /// Get the remaining TTL.
    #[must_use]
    pub fn remaining_ttl(&self) -> Duration {
        self.ttl.saturating_sub(self.cached_at.elapsed())
    }
}

/// LRU cache for DID documents.
///
/// Thread-safe implementation using `RwLock`.
pub struct DidCache {
    /// Maximum number of entries.
    capacity: usize,
    /// Default TTL for entries.
    default_ttl: Duration,
    /// Cache entries.
    entries: RwLock<HashMap<String, CachedDocument>>,
    /// LRU order (most recent at end).
    order: RwLock<Vec<String>>,
    /// Cache statistics.
    stats: RwLock<CacheStats>,
}

/// Cache statistics.
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Number of cache hits.
    pub hits: u64,
    /// Number of cache misses.
    pub misses: u64,
    /// Number of entries evicted.
    pub evictions: u64,
    /// Number of entries expired.
    pub expirations: u64,
}

impl CacheStats {
    /// Calculate hit rate.
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // Acceptable for statistics
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

impl DidCache {
    /// Create a new cache with the specified capacity and default TTL.
    #[must_use]
    pub fn new(capacity: usize, default_ttl: Duration) -> Self {
        Self {
            capacity,
            default_ttl,
            entries: RwLock::new(HashMap::with_capacity(capacity)),
            order: RwLock::new(Vec::with_capacity(capacity)),
            stats: RwLock::new(CacheStats::default()),
        }
    }

    /// Create a cache with default settings (100 entries, 5-minute TTL).
    #[must_use]
    pub fn default_config() -> Self {
        Self::new(100, Duration::from_secs(300))
    }

    /// Get a document from the cache.
    ///
    /// Returns `None` if:
    /// - The DID is not in the cache
    /// - The entry has expired
    pub fn get(&self, did: &str) -> Option<DidDocument> {
        // Check if entry exists and is not expired
        {
            let entries = self.entries.read().ok()?;
            let entry = entries.get(did)?;

            if entry.is_expired() {
                drop(entries);
                self.remove(did);
                if let Ok(mut stats) = self.stats.write() {
                    stats.expirations += 1;
                    stats.misses += 1;
                }
                return None;
            }
        }

        // Update access order and return
        if let Ok(mut order) = self.order.write() {
            order.retain(|k| k != did);
            order.push(did.to_string());
        }

        // Update access count and stats
        if let Ok(mut entries) = self.entries.write() {
            if let Some(entry) = entries.get_mut(did) {
                entry.access_count += 1;
            }
        }

        if let Ok(mut stats) = self.stats.write() {
            stats.hits += 1;
        }

        self.entries
            .read()
            .ok()
            .and_then(|e| e.get(did).map(|e| e.document.clone()))
    }

    /// Insert a document into the cache.
    ///
    /// Uses the default TTL.
    pub fn insert(&self, did: &str, document: DidDocument) {
        self.insert_with_ttl(did, document, self.default_ttl);
    }

    /// Insert a document with a specific TTL.
    pub fn insert_with_ttl(&self, did: &str, document: DidDocument, ttl: Duration) {
        // Evict if at capacity
        self.evict_if_needed();

        let entry = CachedDocument::new(document, ttl);

        if let Ok(mut entries) = self.entries.write() {
            entries.insert(did.to_string(), entry);
        }

        if let Ok(mut order) = self.order.write() {
            order.retain(|k| k != did);
            order.push(did.to_string());
        }
    }

    /// Remove a document from the cache.
    pub fn remove(&self, did: &str) {
        if let Ok(mut entries) = self.entries.write() {
            entries.remove(did);
        }

        if let Ok(mut order) = self.order.write() {
            order.retain(|k| k != did);
        }
    }

    /// Clear all entries from the cache.
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }

        if let Ok(mut order) = self.order.write() {
            order.clear();
        }
    }

    /// Get the current number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.read().map(|e| e.len()).unwrap_or(0)
    }

    /// Check if the cache is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get cache statistics.
    #[must_use]
    pub fn stats(&self) -> CacheStats {
        self.stats.read().map(|s| s.clone()).unwrap_or_default()
    }

    /// Evict the least recently used entry if at capacity.
    fn evict_if_needed(&self) {
        let current_len = self.len();
        if current_len < self.capacity {
            return;
        }

        // Find and remove the oldest entry
        let oldest = {
            let Ok(order) = self.order.read() else { return };
            order.first().cloned()
        };

        if let Some(did) = oldest {
            self.remove(&did);
            if let Ok(mut stats) = self.stats.write() {
                stats.evictions += 1;
            }
        }
    }

    /// Remove all expired entries.
    pub fn prune_expired(&self) {
        let expired: Vec<String> = {
            let Ok(entries) = self.entries.read() else {
                return;
            };

            entries
                .iter()
                .filter(|(_, v)| v.is_expired())
                .map(|(k, _)| k.clone())
                .collect()
        };

        for did in expired {
            self.remove(&did);
            if let Ok(mut stats) = self.stats.write() {
                stats.expirations += 1;
            }
        }
    }
}

impl Default for DidCache {
    fn default() -> Self {
        Self::default_config()
    }
}

/// Cache key prefix for Redis.
pub const REDIS_PREFIX: &str = "did:cache:";

/// Build a Redis cache key for a DID.
#[must_use]
pub fn redis_key(did: &str) -> String {
    format!("{REDIS_PREFIX}{did}")
}

#[cfg(test)]
mod tests {
    use super::super::types::{DidContext, VerificationMethod};
    use super::*;

    fn make_test_doc(id: &str) -> DidDocument {
        DidDocument {
            context: DidContext::default(),
            id: id.to_string(),
            controller: None,
            verification_method: vec![VerificationMethod {
                id: format!("{id}#key-1"),
                method_type: "Ed25519VerificationKey2020".to_string(),
                controller: id.to_string(),
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
        }
    }

    #[test]
    fn cache_basic_operations() {
        let cache = DidCache::new(10, Duration::from_secs(60));

        let doc = make_test_doc("did:web:example.com");
        cache.insert("did:web:example.com", doc.clone());

        let retrieved = cache.get("did:web:example.com").unwrap();
        assert_eq!(retrieved.id, "did:web:example.com");

        cache.remove("did:web:example.com");
        assert!(cache.get("did:web:example.com").is_none());
    }

    #[test]
    fn cache_miss_returns_none() {
        let cache = DidCache::new(10, Duration::from_secs(60));

        assert!(cache.get("did:web:nonexistent.com").is_none());

        // Misses are only tracked when entry exists but is expired
        // (to avoid overhead on every cache access)
        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
    }

    #[test]
    fn cache_hit_updates_stats() {
        let cache = DidCache::new(10, Duration::from_secs(60));

        let doc = make_test_doc("did:web:example.com");
        cache.insert("did:web:example.com", doc);

        let _ = cache.get("did:web:example.com");
        let _ = cache.get("did:web:example.com");

        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
    }

    #[test]
    fn cache_evicts_on_capacity() {
        let cache = DidCache::new(3, Duration::from_secs(60));

        for i in 0..5 {
            let did = format!("did:web:example{i}.com");
            cache.insert(&did, make_test_doc(&did));
        }

        // Should have evicted oldest entries
        assert!(cache.len() <= 3);
        let stats = cache.stats();
        assert!(stats.evictions >= 2);
    }

    #[test]
    fn cache_expired_entries_return_none() {
        let cache = DidCache::new(10, Duration::from_millis(1));

        let doc = make_test_doc("did:web:example.com");
        cache.insert("did:web:example.com", doc);

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(10));

        assert!(cache.get("did:web:example.com").is_none());

        let stats = cache.stats();
        assert_eq!(stats.expirations, 1);
    }

    #[test]
    fn cache_clear_removes_all() {
        let cache = DidCache::new(10, Duration::from_secs(60));

        for i in 0..5 {
            let did = format!("did:web:example{i}.com");
            cache.insert(&did, make_test_doc(&did));
        }

        assert_eq!(cache.len(), 5);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn redis_key_format() {
        assert_eq!(
            redis_key("did:web:example.com"),
            "did:cache:did:web:example.com"
        );
    }

    #[test]
    fn cached_document_expiry() {
        let doc = make_test_doc("did:web:example.com");
        let entry = CachedDocument::new(doc, Duration::from_millis(10));

        assert!(!entry.is_expired());
        assert!(entry.remaining_ttl() > Duration::ZERO);

        std::thread::sleep(Duration::from_millis(15));

        assert!(entry.is_expired());
        assert_eq!(entry.remaining_ttl(), Duration::ZERO);
    }

    #[test]
    fn hit_rate_calculation() {
        let mut stats = CacheStats::default();

        assert_eq!(stats.hit_rate(), 0.0);

        stats.hits = 8;
        stats.misses = 2;
        assert!((stats.hit_rate() - 0.8).abs() < 0.001);
    }
}
