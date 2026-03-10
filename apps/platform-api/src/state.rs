use std::sync::Arc;

use crypto_engine::kms::KmsProvider;

/// Shared application state injected into all handlers via Axum's State extractor.
#[derive(Clone)]
pub struct AppState {
    pub kms: Arc<dyn KmsProvider>,
    // Future: db pool, redis, merkle log, trust registry
}
