use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    extract::ConnectInfo,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use sahi_core::error::{ErrorCode, ErrorResponse, SahiError};
use tokio::sync::Mutex;

/// Maximum idle time before a bucket is evicted (10 minutes).
const BUCKET_IDLE_TTL: Duration = Duration::from_secs(600);

/// Maximum number of buckets before forced eviction of oldest entries.
const MAX_BUCKETS: usize = 10_000;

/// Rate limiter tier configuration.
#[derive(Debug, Clone, Copy)]
pub struct RateLimitTier {
    pub requests_per_minute: u32,
    pub burst: u32,
}

impl RateLimitTier {
    pub const FREE: Self = Self {
        requests_per_minute: 60,
        burst: 10,
    };
    pub const STANDARD: Self = Self {
        requests_per_minute: 600,
        burst: 50,
    };
    #[allow(dead_code)]
    pub const ENTERPRISE: Self = Self {
        requests_per_minute: 6000,
        burst: 200,
    };
}

/// Per-key token bucket state.
struct Bucket {
    tokens: f64,
    last_refill: Instant,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
}

impl Bucket {
    fn new(tier: RateLimitTier) -> Self {
        Self {
            tokens: f64::from(tier.burst),
            last_refill: Instant::now(),
            max_tokens: f64::from(tier.burst),
            refill_rate: f64::from(tier.requests_per_minute) / 60.0,
        }
    }

    fn try_consume(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    fn retry_after(&self) -> Duration {
        if self.refill_rate > 0.0 {
            Duration::from_secs_f64((1.0 - self.tokens) / self.refill_rate)
        } else {
            Duration::from_secs(60)
        }
    }

    fn is_idle(&self) -> bool {
        self.last_refill.elapsed() > BUCKET_IDLE_TTL
    }
}

/// Shared rate limiter state.
#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, Bucket>>>,
    tier: RateLimitTier,
    trust_proxy_headers: bool,
}

impl RateLimiter {
    pub fn new(tier: RateLimitTier) -> Self {
        Self::with_proxy_headers(tier, false)
    }

    pub fn with_proxy_headers(tier: RateLimitTier, trust_proxy_headers: bool) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            tier,
            trust_proxy_headers,
        }
    }

    async fn check(&self, key: &str) -> Result<(), Duration> {
        let mut buckets = self.buckets.lock().await;

        // Evict idle buckets when the map grows too large
        if buckets.len() >= MAX_BUCKETS {
            buckets.retain(|_, bucket| !bucket.is_idle());
        }

        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| Bucket::new(self.tier));

        if bucket.try_consume() {
            Ok(())
        } else {
            Err(bucket.retry_after())
        }
    }
}

/// Rate limiting middleware. Uses IP address as the rate limit key.
pub async fn rate_limit(request: Request, next: Next) -> Response {
    // Extract rate limiter from extensions (set by the router layer)
    let limiter = request.extensions().get::<RateLimiter>().cloned();

    let Some(limiter) = limiter else {
        // No rate limiter configured — pass through
        return next.run(request).await;
    };

    let key =
        client_ip(&request, limiter.trust_proxy_headers).unwrap_or_else(|| "unknown".to_string());

    match limiter.check(&key).await {
        Ok(()) => next.run(request).await,
        Err(retry_after) => {
            let retry_secs = retry_after.as_secs().max(1);
            let error = SahiError::new(
                ErrorCode::RateLimitExceeded,
                "Rate limit exceeded",
                format!("Retry after {retry_secs} seconds"),
            );
            (
                StatusCode::TOO_MANY_REQUESTS,
                [("retry-after", retry_secs.to_string())],
                Json(ErrorResponse::from(error)),
            )
                .into_response()
        }
    }
}

fn client_ip(request: &Request, trust_proxy_headers: bool) -> Option<String> {
    if trust_proxy_headers {
        if let Some(forwarded) = request
            .headers()
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(',').next())
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return Some(forwarded.to_string());
        }

        if let Some(real_ip) = request
            .headers()
            .get("x-real-ip")
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return Some(real_ip.to_string());
        }
    }

    request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|connect_info| connect_info.0.ip().to_string())
}
