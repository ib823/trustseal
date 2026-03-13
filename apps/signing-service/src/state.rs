use std::{sync::Arc, time::Duration};

use jsonwebtoken::Algorithm;
use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::{
    auth::SharedTenantTokenValidator,
    orchestrator::CeremonyOrchestrator,
    repository::{CeremonyRepository, SignerSlotRepository, TransitionRepository},
};

/// Shared application state for the signing service.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub security: SecurityConfig,
    pub tenant_token_validator: Option<SharedTenantTokenValidator>,
    pub ceremony_repository: Arc<CeremonyRepository>,
    pub signer_repository: Arc<SignerSlotRepository>,
    pub transition_repository: Arc<TransitionRepository>,
    pub orchestrator: Arc<CeremonyOrchestrator>,
}

impl AppState {
    #[must_use]
    pub fn new(
        pool: PgPool,
        security: SecurityConfig,
        tenant_token_validator: Option<SharedTenantTokenValidator>,
    ) -> Self {
        Self {
            ceremony_repository: Arc::new(CeremonyRepository::new(pool.clone())),
            signer_repository: Arc::new(SignerSlotRepository::new(pool.clone())),
            transition_repository: Arc::new(TransitionRepository::new(pool.clone())),
            orchestrator: Arc::new(CeremonyOrchestrator::new()),
            pool,
            security,
            tenant_token_validator,
        }
    }
}

/// Runtime security configuration loaded from environment variables.
#[derive(Debug, Clone, Default)]
pub struct SecurityConfig {
    pub allow_insecure_dev_tenant_header: bool,
    pub bearer_hs256_secret: Option<Arc<str>>,
    pub bearer_oidc: Option<OidcBearerAuthConfig>,
}

impl SecurityConfig {
    pub fn from_env() -> Result<Self, String> {
        let allow_insecure_dev_tenant_header = parse_bool_env("ALLOW_INSECURE_DEV_TENANT_HEADER");
        let bearer_hs256_secret = std::env::var("AUTH_JWT_HS256_SECRET")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(Arc::<str>::from);
        let bearer_oidc = OidcBearerAuthConfig::from_env()?;

        if bearer_hs256_secret.is_some() && bearer_oidc.is_some() {
            return Err(
                "AUTH_JWT_HS256_SECRET cannot be combined with AUTH_JWT_ISSUER-based bearer authentication"
                    .to_string(),
            );
        }

        Ok(Self {
            allow_insecure_dev_tenant_header,
            bearer_hs256_secret,
            bearer_oidc,
        })
    }
}

#[derive(Debug, Clone)]
pub struct OidcBearerAuthConfig {
    pub issuer: Arc<str>,
    pub audience: Arc<[Arc<str>]>,
    pub jwks_url: Option<Arc<str>>,
    pub discovery_url: Option<Arc<str>>,
    pub allowed_algs: Arc<[Algorithm]>,
    pub jwks_cache_ttl: Duration,
}

impl OidcBearerAuthConfig {
    pub fn from_env() -> Result<Option<Self>, String> {
        let issuer = non_empty_env("AUTH_JWT_ISSUER");
        let audience = parse_csv_env("AUTH_JWT_AUDIENCE");
        let jwks_url = non_empty_env("AUTH_JWT_JWKS_URL");
        let discovery_url = non_empty_env("AUTH_JWT_DISCOVERY_URL");

        let any_configured = issuer.is_some()
            || !audience.is_empty()
            || jwks_url.is_some()
            || discovery_url.is_some();
        if !any_configured {
            return Ok(None);
        }

        let issuer = issuer.ok_or_else(|| {
            "AUTH_JWT_ISSUER is required when OIDC bearer authentication is configured".to_string()
        })?;
        if audience.is_empty() {
            return Err(
                "AUTH_JWT_AUDIENCE must contain at least one audience when OIDC bearer authentication is configured"
                    .to_string(),
            );
        }
        if jwks_url.is_none() && discovery_url.is_none() {
            return Err(
                "Configure AUTH_JWT_JWKS_URL or AUTH_JWT_DISCOVERY_URL when OIDC bearer authentication is enabled"
                    .to_string(),
            );
        }

        Ok(Some(Self {
            issuer: Arc::<str>::from(issuer),
            audience: audience.into(),
            jwks_url: jwks_url.map(Arc::<str>::from),
            discovery_url: discovery_url.map(Arc::<str>::from),
            allowed_algs: parse_algorithms_env("AUTH_JWT_ALLOWED_ALGS")?,
            jwks_cache_ttl: Duration::from_secs(parse_u64_env(
                "AUTH_JWT_JWKS_CACHE_TTL_SECS",
                300,
            )?),
        }))
    }
}

pub async fn init_pool(database_url: &str) -> Result<PgPool, String> {
    let max_connections: u32 = std::env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);

    PgPoolOptions::new()
        .max_connections(max_connections)
        .idle_timeout(Duration::from_secs(300))
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
        .map_err(|err| format!("failed to connect to Postgres: {err}"))
}

fn parse_bool_env(name: &str) -> bool {
    std::env::var(name).ok().is_some_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

fn non_empty_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn parse_csv_env(name: &str) -> Vec<Arc<str>> {
    std::env::var(name)
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .map(Arc::<str>::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn parse_algorithms_env(name: &str) -> Result<Arc<[Algorithm]>, String> {
    let configured = std::env::var(name).ok().map_or_else(
        || vec!["RS256".to_string()],
        |value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        },
    );

    let mut algorithms = Vec::with_capacity(configured.len());
    for value in configured {
        let algorithm = match value.as_str() {
            "RS256" => Algorithm::RS256,
            "RS384" => Algorithm::RS384,
            "RS512" => Algorithm::RS512,
            "ES256" => Algorithm::ES256,
            "ES384" => Algorithm::ES384,
            "EdDSA" => Algorithm::EdDSA,
            "HS256" | "HS384" | "HS512" => {
                return Err(format!(
                    "{name} cannot contain symmetric algorithms; use AUTH_JWT_HS256_SECRET only for the explicit legacy fallback"
                ))
            }
            _ => return Err(format!("Unsupported JWT algorithm in {name}: {value}")),
        };
        algorithms.push(algorithm);
    }

    if algorithms.is_empty() {
        return Err(format!("{name} must contain at least one algorithm"));
    }

    Ok(algorithms.into())
}

fn parse_u64_env(name: &str, default: u64) -> Result<u64, String> {
    match std::env::var(name) {
        Ok(value) => value
            .trim()
            .parse::<u64>()
            .map_err(|_| format!("{name} must be an unsigned integer")),
        Err(_) => Ok(default),
    }
}
