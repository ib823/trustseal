#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::doc_markdown)]
#![allow(dead_code)] // Fields/types used by Axum extractors, not directly read

//! Sahi Platform API — main entry point

mod middleware;
mod routes;
mod state;

use std::sync::Arc;
use std::time::Duration;

use axum::{
    middleware as axum_middleware,
    routing::get,
    Router,
};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crypto_engine::kms::SoftwareKmsProvider;
use middleware::{
    rate_limit::{RateLimitTier, RateLimiter},
    request_id::inject_request_id,
    tenant::extract_tenant,
};
use state::AppState;

#[tokio::main]
async fn main() {
    // Load .env file (best-effort, not required)
    let _ = dotenvy::dotenv();

    // Initialize tracing (structured JSON in production)
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "platform_api=debug,tower_http=debug,crypto_engine=info".into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize KMS provider (SoftwareKmsProvider for dev)
    let kms = Arc::new(SoftwareKmsProvider::new());

    let app_state = AppState {
        kms: kms as Arc<dyn crypto_engine::kms::KmsProvider>,
    };

    // Rate limiter (Standard tier for dev)
    let rate_limiter = RateLimiter::new(RateLimitTier::STANDARD);

    // Build the middleware chain (order matters — per MASTER_PLAN §5.1)
    // Request flows: Compression → CORS → Trace → Rate Limit → Request ID → Tenant → Handler
    let app = Router::new()
        // Public routes (no tenant required)
        .route("/health", get(routes::health::health_check))
        // API v1 routes (tenant required)
        .nest(
            "/api/v1",
            Router::new()
                .route("/status", get(routes::health::health_check))
                .layer(axum_middleware::from_fn(
                    middleware::tenant::require_tenant,
                )),
        )
        // Global middleware (applied to ALL routes, bottom-up execution)
        .layer(axum_middleware::from_fn(extract_tenant))
        .layer(axum_middleware::from_fn(inject_request_id))
        .layer(axum_middleware::from_fn(
            middleware::rate_limit::rate_limit,
        ))
        .layer(axum::Extension(rate_limiter))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(
            CorsLayer::new()
                .allow_origin(Any) // TODO: restrict to known origins in production
                .allow_methods(Any)
                .allow_headers(Any)
                .max_age(Duration::from_secs(3600)),
        )
        .with_state(app_state);

    let host = std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("API_PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{host}:{port}");

    tracing::info!("Sahi Platform API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server error");
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    tracing::info!("Shutdown signal received, draining connections...");
}
