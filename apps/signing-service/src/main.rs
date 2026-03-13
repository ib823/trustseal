#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(unused_imports)]
// Allow dead_code for stub modules - will be filled in during TM-1/TM-2 implementation
#![allow(dead_code)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]

mod auth;
mod domain;
mod orchestrator;
mod repository;
mod routes;
mod state;

use std::time::Duration;

use axum::{
    http::{HeaderName, HeaderValue},
    middleware as axum_middleware,
    routing::get,
    Router,
};
use tower_http::{
    compression::CompressionLayer, limit::RequestBodyLimitLayer,
    set_header::SetResponseHeaderLayer, trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use auth::{build_tenant_token_validator, extract_auth};
use state::{init_pool, AppState, SecurityConfig};

/// Maximum request body size (2 MB).
const MAX_REQUEST_BODY_SIZE: usize = 2 * 1024 * 1024;

/// Request processing timeout.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    // Initialize tracing — JSON format when RUST_LOG_FORMAT=json (production)
    let use_json = std::env::var("RUST_LOG_FORMAT")
        .ok()
        .is_some_and(|v| v.eq_ignore_ascii_case("json"));

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "signing_service=debug,tower_http=debug".into());

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

    let security = SecurityConfig::from_env().expect("Invalid security configuration");
    let tenant_token_validator =
        build_tenant_token_validator(&security).expect("Invalid bearer auth configuration");
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be configured");
    let pool = init_pool(&database_url)
        .await
        .expect("Failed to initialize signing-service database connection");
    let app_state = AppState::new(pool, security, tenant_token_validator);

    // Build the router
    let app = Router::new()
        .route("/health", get(health))
        .nest(
            "/api/v1/ceremonies",
            routes::ceremony_router().layer(axum_middleware::from_fn(auth::require_tenant)),
        )
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            extract_auth,
        ))
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
        // Request body size limit
        .layer(RequestBodyLimitLayer::new(MAX_REQUEST_BODY_SIZE))
        // Request timeout
        .layer(tower_http::timeout::TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            REQUEST_TIMEOUT,
        ))
        .with_state(app_state);

    // Start the server
    let host = std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("API_PORT").unwrap_or_else(|_| "3003".to_string());
    let addr = format!("{host}:{port}");

    info!(
        version = env!("CARGO_PKG_VERSION"),
        "Signing Service listening on {}", addr,
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
    info!("Shutdown signal received, draining connections...");
}

async fn health() -> &'static str {
    "healthy"
}
