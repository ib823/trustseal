use axum::{extract::State, Json};
use chrono::Utc;
use serde_json::{json, Value};

use crate::state::AppState;

/// GET /health — liveness probe.
/// Returns OK if the service process is running. Does not check dependencies.
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "platform-api",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": Utc::now().to_rfc3339(),
    }))
}

/// GET /health/ready — readiness probe.
/// Checks that the service can reach its critical dependencies (database).
pub async fn readiness_check(
    State(state): State<AppState>,
) -> (axum::http::StatusCode, Json<Value>) {
    let mut checks: Vec<Value> = Vec::new();
    let mut all_ok = true;

    // Check database connectivity via the eKYC store's pool (if available)
    let db_ok = if let Some(store) = &state.ekyc_store {
        match store.health_check().await {
            Ok(()) => true,
            Err(e) => {
                tracing::warn!("Readiness: database check failed: {e}");
                false
            }
        }
    } else {
        // No database configured — report as degraded but not fatal for dev
        true
    };

    checks.push(json!({
        "name": "database",
        "status": if db_ok { "ok" } else { "fail" },
    }));
    if !db_ok {
        all_ok = false;
    }

    let status_code = if all_ok {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(json!({
            "status": if all_ok { "ready" } else { "not_ready" },
            "service": "platform-api",
            "version": env!("CARGO_PKG_VERSION"),
            "timestamp": Utc::now().to_rfc3339(),
            "checks": checks,
        })),
    )
}
