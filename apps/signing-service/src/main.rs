#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(unused_imports)]
// Allow dead_code for stub modules - will be filled in during TM-1/TM-2 implementation
#![allow(dead_code)]

mod auth;
mod domain;
mod orchestrator;
mod repository;
mod routes;
mod state;

use axum::{middleware as axum_middleware, routing::get, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use auth::{build_tenant_token_validator, extract_auth};
use state::{init_pool, AppState, SecurityConfig};

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "signing_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

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
            routes::ceremony_router()
                .layer(axum_middleware::from_fn(auth::require_tenant)),
        )
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            extract_auth,
        ))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .with_state(app_state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3003));
    info!("Signing Service starting on {}", addr);

    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}

async fn health() -> &'static str {
    "healthy"
}
