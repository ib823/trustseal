//! Background sync for status lists via MQTT and HTTP polling.

use std::sync::Arc;
use std::time::Duration;

use reqwest::Url;
use serde_json::Value;
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info};

use super::RevocationCache;

/// Sync errors.
#[derive(Debug, Error)]
pub enum SyncError {
    #[error("HTTP fetch error: {0}")]
    HttpError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Cache update error: {0}")]
    CacheError(#[from] super::cache::RevocationError),
}

/// Background sync configuration.
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Poll interval for HTTP refresh.
    pub poll_interval: Duration,

    /// Base URL for status list API.
    pub api_base_url: String,

    /// Whether to enable background polling.
    pub enabled: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(300), // 5 minutes
            api_base_url: "https://api.sahi.my".to_string(),
            enabled: true,
        }
    }
}

/// Status list sync service.
pub struct RevocationSync {
    cache: Arc<RevocationCache>,
    config: SyncConfig,
    /// URLs to periodically refresh.
    watch_urls: RwLock<Vec<String>>,
}

impl RevocationSync {
    /// Create a new sync service.
    pub fn new(cache: Arc<RevocationCache>, config: SyncConfig) -> Self {
        Self {
            cache,
            config,
            watch_urls: RwLock::new(Vec::new()),
        }
    }

    /// Add a URL to the watch list.
    pub async fn watch(&self, url: &str) {
        let mut urls = self.watch_urls.write().await;
        if !urls.contains(&url.to_string()) {
            urls.push(url.to_string());
            debug!("Added status list to watch: {}", url);
        }
    }

    /// Remove a URL from the watch list.
    pub async fn unwatch(&self, url: &str) {
        let mut urls = self.watch_urls.write().await;
        urls.retain(|u| u != url);
    }

    /// Fetch and update a status list from HTTP.
    pub async fn fetch_and_update(&self, url: &str) -> Result<(), SyncError> {
        info!("Fetching status list: {}", url);

        let target = resolve_status_list_url(url, &self.config.api_base_url)?;
        let response = reqwest::get(target)
            .await
            .map_err(|e| SyncError::HttpError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            return Err(SyncError::HttpError(format!(
                "status list fetch returned HTTP {status}"
            )));
        }

        let body = response
            .json::<Value>()
            .await
            .map_err(|e| SyncError::ParseError(e.to_string()))?;
        let encoded_list = body
            .get("credentialSubject")
            .and_then(|subject| subject.get("encodedList"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                SyncError::ParseError("Missing credentialSubject.encodedList".to_string())
            })?;
        let credential_id = body
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| SyncError::ParseError("Missing id".to_string()))?;

        self.cache.update(url, credential_id, encoded_list).await?;
        Ok(())
    }

    /// Handle MQTT status list update message.
    pub async fn handle_mqtt_update(
        &self,
        url: &str,
        credential_id: &str,
        encoded_list: &str,
    ) -> Result<(), SyncError> {
        info!("Received MQTT status list update: {}", url);
        self.cache.update(url, credential_id, encoded_list).await?;
        Ok(())
    }

    /// Run background polling loop.
    pub async fn run_polling(&self) {
        if !self.config.enabled {
            info!("Background polling disabled");
            return;
        }

        info!(
            "Starting background polling (interval: {:?})",
            self.config.poll_interval
        );

        let mut ticker = interval(self.config.poll_interval);

        loop {
            ticker.tick().await;

            let urls = self.watch_urls.read().await.clone();

            for url in urls {
                // Only refresh if not fresh
                if !self.cache.is_fresh(&url).await {
                    if let Err(e) = self.fetch_and_update(&url).await {
                        error!("Failed to refresh status list {}: {}", url, e);
                    }
                }
            }
        }
    }
}

fn resolve_status_list_url(url: &str, api_base_url: &str) -> Result<Url, SyncError> {
    if let Ok(parsed) = Url::parse(url) {
        return Ok(parsed);
    }

    let base = Url::parse(api_base_url).map_err(|e| SyncError::HttpError(e.to_string()))?;
    base.join(url)
        .map_err(|e| SyncError::HttpError(format!("Invalid status list URL: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_watch_urls() {
        let cache = Arc::new(RevocationCache::new(Duration::from_secs(900)));
        let sync = RevocationSync::new(cache, SyncConfig::default());

        sync.watch("https://example.com/status/1").await;
        sync.watch("https://example.com/status/2").await;

        let urls = sync.watch_urls.read().await;
        assert_eq!(urls.len(), 2);
    }

    #[tokio::test]
    async fn test_unwatch() {
        let cache = Arc::new(RevocationCache::new(Duration::from_secs(900)));
        let sync = RevocationSync::new(cache, SyncConfig::default());

        sync.watch("https://example.com/status/1").await;
        sync.watch("https://example.com/status/2").await;
        sync.unwatch("https://example.com/status/1").await;

        let urls = sync.watch_urls.read().await;
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "https://example.com/status/2");
    }
}
