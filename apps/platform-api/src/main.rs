#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::doc_markdown)]
#![allow(dead_code)] // Fields/types used by Axum extractors, not directly read

//! Sahi Platform API — main entry point

mod middleware;
mod routes;
mod services;
mod state;

use std::sync::Arc;
use std::time::Duration;

use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderName, HeaderValue, Method,
    },
    middleware as axum_middleware,
    routing::get,
    Router,
};
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crypto_engine::kms::SoftwareKmsProvider;
use middleware::{
    rate_limit::{RateLimitTier, RateLimiter},
    request_id::inject_request_id,
    tenant::{build_tenant_token_validator, extract_tenant},
};
use services::ekyc::{EkycService, PostgresEkycStore};
use state::{AppState, SecurityConfig};

#[tokio::main]
async fn main() {
    // Load .env file (best-effort, not required)
    let _ = dotenvy::dotenv();

    // Initialize tracing (structured JSON in production)
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "platform_api=debug,tower_http=debug,crypto_engine=info".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize KMS provider (SoftwareKmsProvider for dev)
    let kms = Arc::new(SoftwareKmsProvider::new());
    let security = SecurityConfig::from_env().expect("Invalid security configuration");
    let tenant_token_validator =
        build_tenant_token_validator(&security).expect("Invalid bearer auth configuration");
    let ekyc_service = EkycService::from_env().ok().map(Arc::new);
    let ekyc_store = match std::env::var("DATABASE_URL") {
        Ok(database_url) => match init_ekyc_store(&database_url).await {
            Ok(store) => Some(store),
            Err(err) => {
                tracing::warn!("Failed to initialize eKYC store: {err}");
                None
            }
        },
        Err(_) => None,
    };

    let app_state = AppState {
        kms: kms as Arc<dyn crypto_engine::kms::KmsProvider>,
        security: security.clone(),
        tenant_token_validator,
        ekyc_service,
        ekyc_store,
    };

    // Rate limiter (Standard tier for dev)
    let rate_limiter =
        RateLimiter::with_proxy_headers(RateLimitTier::STANDARD, security.trust_proxy_headers);

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
                .nest("/ekyc", routes::ekyc::router())
                .layer(axum_middleware::from_fn(middleware::tenant::require_tenant)),
        )
        // Global middleware (applied to ALL routes, bottom-up execution)
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            extract_tenant,
        ))
        .layer(axum_middleware::from_fn(inject_request_id))
        .layer(axum_middleware::from_fn(middleware::rate_limit::rate_limit))
        .layer(axum::Extension(rate_limiter))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(build_cors_layer(&security))
        .with_state(app_state);

    let host = std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("API_PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{host}:{port}");

    tracing::info!("Sahi Platform API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
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

fn build_cors_layer(security: &SecurityConfig) -> CorsLayer {
    let mut allowed_headers = vec![
        AUTHORIZATION,
        CONTENT_TYPE,
        HeaderName::from_static("x-request-id"),
    ];

    if security.allow_insecure_dev_tenant_header {
        allowed_headers.push(HeaderName::from_static("x-tenant-id"));
    }

    let layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(allowed_headers)
        .max_age(Duration::from_secs(3600));

    let origins = std::env::var("CORS_ALLOWED_ORIGINS")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|origin| !origin.is_empty())
                .filter_map(|origin| HeaderValue::from_str(origin).ok())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if origins.is_empty() {
        layer
    } else {
        layer.allow_origin(AllowOrigin::list(origins))
    }
}

async fn init_ekyc_store(
    database_url: &str,
) -> Result<services::ekyc::SharedEkycStore, sqlx::Error> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    Ok(Arc::new(PostgresEkycStore::new(pool)))
}
