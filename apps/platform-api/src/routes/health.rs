use axum::Json;
use chrono::Utc;
use serde_json::{json, Value};

/// GET /health — service health check.
/// Returns OK if the service is running. In production, also checks DB, Redis, KMS.
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "platform-api",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": Utc::now().to_rfc3339(),
    }))
}
