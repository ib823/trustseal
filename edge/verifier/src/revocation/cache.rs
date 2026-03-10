//! Status list cache with TTL and stale handling.

use std::collections::HashMap;
use std::io::Read;
use std::time::{Duration, Instant};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use flate2::read::GzDecoder;
use thiserror::Error;
use tokio::sync::RwLock;

/// Revocation cache errors.
#[derive(Debug, Error)]
pub enum RevocationError {
    #[error("Status list not found: {0}")]
    NotFound(String),

    #[error("Status list is stale (age: {0:?}, threshold: {1:?})")]
    StaleCache(Duration, Duration),

    #[error("Invalid status list format: {0}")]
    InvalidFormat(String),

    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),

    #[error("Decompression error: {0}")]
    DecompressionError(#[from] std::io::Error),

    #[error("Index out of range: {0}")]
    IndexOutOfRange(usize),
}

/// Cached status list entry.
#[derive(Debug, Clone)]
struct CachedStatusList {
    /// Decompressed bitstring.
    bitstring: Vec<u8>,

    /// When this entry was fetched.
    fetched_at: Instant,

    /// Status list credential ID.
    credential_id: String,
}

/// Revocation status cache.
///
/// Caches decompressed status lists for fast revocation checking.
/// Implements TTL and stale thresholds per spec.
pub struct RevocationCache {
    /// Cached status lists by URL.
    cache: RwLock<HashMap<String, CachedStatusList>>,

    /// TTL for fresh entries.
    ttl: Duration,

    /// Maximum age before deny-all.
    stale_threshold: Duration,
}

impl RevocationCache {
    /// Create a new revocation cache.
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            ttl,
            stale_threshold: Duration::from_secs(4 * 3600), // 4 hours
        }
    }

    /// Create with custom stale threshold.
    pub fn with_stale_threshold(ttl: Duration, stale_threshold: Duration) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            ttl,
            stale_threshold,
        }
    }

    /// Check if a credential is revoked.
    ///
    /// Returns:
    /// - `Ok(true)` if revoked
    /// - `Ok(false)` if not revoked
    /// - `Err(RevocationError::StaleCache)` if cache is too old
    /// - `Err(RevocationError::NotFound)` if status list not cached
    pub async fn is_revoked(
        &self,
        status_list_url: &str,
        index: usize,
    ) -> Result<bool, RevocationError> {
        let cache = self.cache.read().await;

        let entry = cache
            .get(status_list_url)
            .ok_or_else(|| RevocationError::NotFound(status_list_url.to_string()))?;

        // Check staleness
        let age = entry.fetched_at.elapsed();
        if age > self.stale_threshold {
            return Err(RevocationError::StaleCache(age, self.stale_threshold));
        }

        // Check bit at index (MSB-first per W3C spec)
        let byte_index = index / 8;
        let bit_index = 7 - (index % 8);

        if byte_index >= entry.bitstring.len() {
            return Err(RevocationError::IndexOutOfRange(index));
        }

        let is_revoked = (entry.bitstring[byte_index] >> bit_index) & 1 == 1;
        Ok(is_revoked)
    }

    /// Check if cache entry is fresh (within TTL).
    pub async fn is_fresh(&self, status_list_url: &str) -> bool {
        let cache = self.cache.read().await;
        cache
            .get(status_list_url)
            .is_some_and(|entry| entry.fetched_at.elapsed() < self.ttl)
    }

    /// Check if cache entry exists (even if stale).
    pub async fn has_entry(&self, status_list_url: &str) -> bool {
        let cache = self.cache.read().await;
        cache.contains_key(status_list_url)
    }

    /// Get age of cached entry.
    pub async fn get_age(&self, status_list_url: &str) -> Option<Duration> {
        let cache = self.cache.read().await;
        cache
            .get(status_list_url)
            .map(|entry| entry.fetched_at.elapsed())
    }

    /// Update cache with new status list.
    ///
    /// Parses and decompresses the encoded list (gzip + base64).
    pub async fn update(
        &self,
        status_list_url: &str,
        credential_id: &str,
        encoded_list: &str,
    ) -> Result<(), RevocationError> {
        // Decode base64
        let compressed = BASE64.decode(encoded_list)?;

        // Decompress gzip
        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut bitstring = Vec::new();
        decoder.read_to_end(&mut bitstring)?;

        // Store in cache
        let entry = CachedStatusList {
            bitstring,
            fetched_at: Instant::now(),
            credential_id: credential_id.to_string(),
        };

        let mut cache = self.cache.write().await;
        cache.insert(status_list_url.to_string(), entry);

        Ok(())
    }

    /// Remove a cache entry.
    pub async fn invalidate(&self, status_list_url: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(status_list_url);
    }

    /// Clear all cache entries.
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get list of all cached URLs.
    pub async fn cached_urls(&self) -> Vec<String> {
        let cache = self.cache.read().await;
        cache.keys().cloned().collect()
    }

    /// Get statistics about the cache.
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let now = Instant::now();

        let mut fresh = 0;
        let mut stale = 0;
        let mut expired = 0;

        for entry in cache.values() {
            let age = now.duration_since(entry.fetched_at);
            if age < self.ttl {
                fresh += 1;
            } else if age < self.stale_threshold {
                stale += 1;
            } else {
                expired += 1;
            }
        }

        CacheStats {
            total: cache.len(),
            fresh,
            stale,
            expired,
        }
    }
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total: usize,
    pub fresh: usize,
    pub stale: usize,
    pub expired: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    fn create_test_status_list(size: usize, revoked_indices: &[usize]) -> String {
        let mut bitstring = vec![0u8; size.div_ceil(8)];

        for &index in revoked_indices {
            let byte_index = index / 8;
            let bit_index = 7 - (index % 8);
            if byte_index < bitstring.len() {
                bitstring[byte_index] |= 1 << bit_index;
            }
        }

        // Gzip compress
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&bitstring).unwrap();
        let compressed = encoder.finish().unwrap();

        // Base64 encode
        BASE64.encode(&compressed)
    }

    #[tokio::test]
    async fn test_cache_update_and_check() {
        let cache = RevocationCache::new(Duration::from_secs(900));

        let encoded = create_test_status_list(1000, &[5, 10, 100]);
        cache
            .update("https://example.com/status/1", "SLC_01HXK", &encoded)
            .await
            .unwrap();

        // Check not revoked
        assert!(!cache
            .is_revoked("https://example.com/status/1", 0)
            .await
            .unwrap());
        assert!(!cache
            .is_revoked("https://example.com/status/1", 50)
            .await
            .unwrap());

        // Check revoked
        assert!(cache
            .is_revoked("https://example.com/status/1", 5)
            .await
            .unwrap());
        assert!(cache
            .is_revoked("https://example.com/status/1", 10)
            .await
            .unwrap());
        assert!(cache
            .is_revoked("https://example.com/status/1", 100)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_cache_not_found() {
        let cache = RevocationCache::new(Duration::from_secs(900));

        let result = cache.is_revoked("https://example.com/unknown", 0).await;
        assert!(matches!(result, Err(RevocationError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_cache_freshness() {
        let cache = RevocationCache::new(Duration::from_millis(100));

        let encoded = create_test_status_list(100, &[]);
        cache
            .update("https://example.com/status/1", "SLC_01HXK", &encoded)
            .await
            .unwrap();

        assert!(cache.is_fresh("https://example.com/status/1").await);

        // Wait for TTL to expire
        tokio::time::sleep(Duration::from_millis(150)).await;

        assert!(!cache.is_fresh("https://example.com/status/1").await);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = RevocationCache::new(Duration::from_secs(900));

        let encoded = create_test_status_list(100, &[]);
        cache
            .update("https://example.com/status/1", "SLC_01HXK", &encoded)
            .await
            .unwrap();
        cache
            .update("https://example.com/status/2", "SLC_02HXK", &encoded)
            .await
            .unwrap();

        let stats = cache.stats().await;
        assert_eq!(stats.total, 2);
        assert_eq!(stats.fresh, 2);
        assert_eq!(stats.stale, 0);
    }
}
