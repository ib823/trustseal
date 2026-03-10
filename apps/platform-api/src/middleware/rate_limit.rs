use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tokio::sync::Mutex;

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
}

/// Shared rate limiter state.
#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, Bucket>>>,
    tier: RateLimitTier,
}

impl RateLimiter {
    pub fn new(tier: RateLimitTier) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            tier,
        }
    }

    async fn check(&self, key: &str) -> Result<(), Duration> {
        let mut buckets = self.buckets.lock().await;
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
pub async fn rate_limit(
    request: Request,
    next: Next,
) -> Response {
    // Extract rate limiter from extensions (set by the router layer)
    let limiter = request
        .extensions()
        .get::<RateLimiter>()
        .cloned();

    let Some(limiter) = limiter else {
        // No rate limiter configured — pass through
        return next.run(request).await;
    };

    // Use IP from ConnectInfo or forwarded headers
    let key = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .unwrap_or("unknown")
        .to_string();

    match limiter.check(&key).await {
        Ok(()) => next.run(request).await,
        Err(retry_after) => {
            let retry_secs = retry_after.as_secs().max(1);
            (
                StatusCode::TOO_MANY_REQUESTS,
                [("retry-after", retry_secs.to_string())],
                Json(json!({
                    "error": {
                        "code": "SAHI_1200",
                        "message": "Rate limit exceeded",
                        "action": format!("Retry after {} seconds", retry_secs)
                    }
                })),
            )
                .into_response()
        }
    }
}
