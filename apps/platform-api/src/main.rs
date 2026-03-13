#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
// Allow dead_code for stub modules under development (eKYC, rate limit tiers, etc.)
#![allow(dead_code)]

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
    limit::RequestBodyLimitLayer,
    set_header::SetResponseHeaderLayer,
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

/// Maximum request body size (2 MB).
const MAX_REQUEST_BODY_SIZE: usize = 2 * 1024 * 1024;

/// Request processing timeout.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() {
    // Load .env file (best-effort, not required)
    let _ = dotenvy::dotenv();

    // Initialize tracing — JSON format when RUST_LOG_FORMAT=json (production)
    let use_json = std::env::var("RUST_LOG_FORMAT")
        .ok()
        .is_some_and(|v| v.eq_ignore_ascii_case("json"));

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "platform_api=debug,tower_http=debug,crypto_engine=info".into());

    if use_json {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

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
    // Request flows: BodyLimit → Timeout → Compression → SecurityHeaders → CORS → Trace → Rate Limit → Request ID → Tenant → Handler
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
        // Security headers
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("cache-control"),
            HeaderValue::from_static("no-store"),
        ))
        .layer(CompressionLayer::new())
        .layer(build_cors_layer(&security))
        // Request body size limit (S-03)
        .layer(RequestBodyLimitLayer::new(MAX_REQUEST_BODY_SIZE))
        // Request timeout (S-04)
        .layer(tower_http::timeout::TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            REQUEST_TIMEOUT,
        ))
        .with_state(app_state);

    let host = std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("API_PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{host}:{port}");

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "Sahi Platform API listening on {}",
        addr,
    );

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
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
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
        // Default: deny all cross-origin requests in production.
        // For development, set CORS_ALLOWED_ORIGINS explicitly.
        layer.allow_origin(AllowOrigin::exact(HeaderValue::from_static(
            "http://localhost:3001",
        )))
    } else {
        layer.allow_origin(AllowOrigin::list(origins))
    }
}

async fn init_ekyc_store(
    database_url: &str,
) -> Result<services::ekyc::SharedEkycStore, sqlx::Error> {
    let max_connections: u32 = std::env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .idle_timeout(Duration::from_secs(300))
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await?;
    Ok(Arc::new(PostgresEkycStore::new(pool)))
}
