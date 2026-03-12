use std::{
    collections::BTreeSet,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::Utc;
use jsonwebtoken::{
    decode, decode_header,
    jwk::{Jwk, JwkSet},
    Algorithm, DecodingKey, Header, Validation,
};
use ring::hmac;
use sahi_core::auth::{AuthContext, Role};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::RwLock;
use tracing::warn;

use crate::state::{AppState, OidcBearerAuthConfig, SecurityConfig};

pub type SharedTenantTokenValidator = Arc<TenantTokenValidator>;

/// Tenant context extracted from a validated bearer token or explicit dev header.
#[derive(Debug, Clone)]
pub struct TenantId(pub String);

/// Middleware: validate bearer token, extract tenant context, and attach auth context.
pub async fn extract_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(token) = bearer_token(&request) {
        let claims = if let Some(validator) = state.tenant_token_validator.as_ref() {
            validator.validate_bearer_token(token).await
        } else if let Some(secret) = state.security.bearer_hs256_secret.as_deref() {
            validate_legacy_bearer_token(token, secret.as_bytes())
                .map_err(|err| TenantTokenError::InvalidToken(err.to_string()))
        } else {
            Err(TenantTokenError::Configuration(
                "Bearer authentication is not configured".to_string(),
            ))
        };

        let claims = match claims {
            Ok(claims) => claims,
            Err(err) => {
                return unauthorized(
                    &format!("Invalid bearer token: {err}"),
                    auth_action(&state.security),
                );
            }
        };

        let auth_context = AuthContext::new(
            claims.tenant_id.clone(),
            claims.subject,
            claims.roles,
            claims.scopes,
            claims.client_id,
            claims.authorized_party,
        );

        request
            .extensions_mut()
            .insert(TenantId(claims.tenant_id.clone()));
        request.extensions_mut().insert(auth_context);
        return next.run(request).await;
    }

    if !state.security.allow_insecure_dev_tenant_header {
        return next.run(request).await;
    }

    let tenant_id = request
        .headers()
        .get("x-tenant-id")
        .and_then(|value| value.to_str().ok())
        .map(String::from);

    if let Some(ref tid) = tenant_id {
        if !tid.starts_with("TNT_") {
            return bad_request(
                "Invalid tenant ID format",
                "Tenant ID must start with TNT_ prefix",
            );
        }
        request.extensions_mut().insert(TenantId(tid.clone()));
    }

    next.run(request).await
}

pub fn build_tenant_token_validator(
    security: &SecurityConfig,
) -> Result<Option<SharedTenantTokenValidator>, String> {
    security
        .bearer_oidc
        .clone()
        .map(TenantTokenValidator::new)
        .transpose()
        .map(|validator| validator.map(Arc::new))
}

/// Require a tenant context for protected routes.
pub async fn require_tenant(request: Request, next: Next) -> Response {
    if request.extensions().get::<TenantId>().is_none() {
        return unauthorized(
            "Tenant context required",
            "Authenticate with a tenant-scoped bearer token or explicitly enable the dev tenant header",
        );
    }

    next.run(request).await
}

#[derive(Debug, Deserialize)]
struct LegacyJwtHeader {
    alg: String,
}

#[derive(Debug, Deserialize)]
struct LegacyJwtClaims {
    exp: Option<i64>,
    nbf: Option<i64>,
    tenant_id: Option<String>,
    tid: Option<String>,
    sub: Option<String>,
    scope: Option<String>,
    scp: Option<Vec<String>>,
    roles: Option<Vec<String>>,
    role: Option<String>,
    realm_access: Option<RealmAccess>,
    azp: Option<String>,
    client_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RealmAccess {
    #[serde(default)]
    roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct BearerClaims {
    #[serde(default)]
    tenant_id: Option<String>,
    #[serde(default)]
    tid: Option<String>,
    #[serde(default)]
    sub: Option<String>,
    #[serde(default)]
    iat: Option<i64>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    scp: Option<Vec<String>>,
    #[serde(default)]
    roles: Option<Vec<String>>,
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    realm_access: Option<RealmAccess>,
    #[serde(default)]
    azp: Option<String>,
    #[serde(default)]
    client_id: Option<String>,
}

#[derive(Debug)]
struct ValidatedClaims {
    tenant_id: String,
    subject: Option<String>,
    roles: BTreeSet<Role>,
    scopes: BTreeSet<String>,
    client_id: Option<String>,
    authorized_party: Option<String>,
}

pub struct TenantTokenValidator {
    config: OidcBearerAuthConfig,
    http_client: reqwest::Client,
    jwks_cache: RwLock<Option<CachedJwks>>,
}

struct CachedJwks {
    fetched_at: Instant,
    jwks: Arc<JwkSet>,
}

#[derive(Debug, Deserialize)]
struct OpenIdProviderMetadata {
    issuer: String,
    jwks_uri: String,
}

#[derive(Debug, thiserror::Error)]
enum TenantTokenError {
    #[error("{0}")]
    Configuration(String),
    #[error("{0}")]
    InvalidToken(String),
    #[error("{0}")]
    Discovery(String),
    #[error("{0}")]
    Jwks(String),
}

impl TenantTokenValidator {
    fn new(config: OidcBearerAuthConfig) -> Result<Self, String> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|err| format!("failed to build auth HTTP client: {err}"))?;

        Ok(Self {
            config,
            http_client,
            jwks_cache: RwLock::new(None),
        })
    }

    async fn validate_bearer_token(
        &self,
        token: &str,
    ) -> Result<ValidatedClaims, TenantTokenError> {
        let header = decode_header(token).map_err(|err| {
            TenantTokenError::InvalidToken(format!("malformed JWT header: {err}"))
        })?;
        self.ensure_allowed_algorithm(header.alg)?;
        let decoding_key = self.decoding_key_for_token(&header).await?;

        let mut validation = Validation::new(header.alg);
        validation.set_required_spec_claims(&["exp", "iss", "aud"]);
        validation.set_issuer(&[self.config.issuer.as_ref()]);
        let audiences = self
            .config
            .audience
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>();
        validation.set_audience(&audiences);
        validation.validate_nbf = true;
        validation.leeway = 60;

        let token_data =
            decode::<BearerClaims>(token, &decoding_key, &validation).map_err(|err| {
                TenantTokenError::InvalidToken(format!("token validation failed: {err}"))
            })?;
        let claims = token_data.claims;

        if let Some(iat) = claims.iat {
            let now = Utc::now().timestamp();
            if iat > now + 60 {
                return Err(TenantTokenError::InvalidToken(
                    "token issued-at time is in the future".to_string(),
                ));
            }
        }

        validated_claims(
            claims.tenant_id.or(claims.tid),
            claims.sub,
            claims.scope,
            claims.scp,
            claims.roles,
            claims.role,
            claims.realm_access,
            claims.client_id,
            claims.azp,
        )
        .map_err(|err| TenantTokenError::InvalidToken(err.to_string()))
    }

    fn ensure_allowed_algorithm(&self, algorithm: Algorithm) -> Result<(), TenantTokenError> {
        if !self.config.allowed_algs.contains(&algorithm) {
            return Err(TenantTokenError::InvalidToken(format!(
                "JWT algorithm {algorithm:?} is not allowed by configuration"
            )));
        }

        match algorithm {
            Algorithm::RS256
            | Algorithm::RS384
            | Algorithm::RS512
            | Algorithm::ES256
            | Algorithm::ES384
            | Algorithm::EdDSA => Ok(()),
            unsupported => Err(TenantTokenError::InvalidToken(format!(
                "JWT algorithm {unsupported:?} is not supported for JWKS-based validation"
            ))),
        }
    }

    async fn decoding_key_for_token(
        &self,
        header: &Header,
    ) -> Result<DecodingKey, TenantTokenError> {
        let jwks = self.jwks_for_kid(header.kid.as_deref()).await?;
        let jwk = select_jwk(&jwks, header.kid.as_deref()).map_err(TenantTokenError::Jwks)?;
        DecodingKey::try_from(jwk).map_err(|err| TenantTokenError::Jwks(err.to_string()))
    }

    async fn jwks_for_kid(&self, kid: Option<&str>) -> Result<Arc<JwkSet>, TenantTokenError> {
        if let Some(jwks) = self.cached_jwks(kid).await {
            return Ok(jwks);
        }

        let jwks = self.fetch_jwks().await?;
        if let Err(err) = select_jwk(&jwks, kid) {
            return Err(TenantTokenError::Jwks(err));
        }

        let mut cache = self.jwks_cache.write().await;
        *cache = Some(CachedJwks {
            fetched_at: Instant::now(),
            jwks: Arc::clone(&jwks),
        });

        Ok(jwks)
    }

    async fn cached_jwks(&self, kid: Option<&str>) -> Option<Arc<JwkSet>> {
        let cache = self.jwks_cache.read().await;
        let cached = cache.as_ref()?;
        if cached.fetched_at.elapsed() > self.config.jwks_cache_ttl {
            return None;
        }
        if select_jwk(&cached.jwks, kid).is_err() {
            return None;
        }
        Some(Arc::clone(&cached.jwks))
    }

    async fn fetch_jwks(&self) -> Result<Arc<JwkSet>, TenantTokenError> {
        let jwks_url = self.resolve_jwks_url().await?;
        let response = self
            .http_client
            .get(&jwks_url)
            .send()
            .await
            .map_err(|err| TenantTokenError::Jwks(format!("JWKS fetch failed: {err}")))?;

        if !response.status().is_success() {
            return Err(TenantTokenError::Jwks(format!(
                "JWKS fetch failed with HTTP {}",
                response.status()
            )));
        }

        response
            .json::<JwkSet>()
            .await
            .map(Arc::new)
            .map_err(|err| TenantTokenError::Jwks(format!("invalid JWKS response: {err}")))
    }

    async fn resolve_jwks_url(&self) -> Result<String, TenantTokenError> {
        let Some(discovery_url) = self.config.discovery_url.as_deref() else {
            return self
                .config
                .jwks_url
                .as_ref()
                .map(std::string::ToString::to_string)
                .ok_or_else(|| {
                    TenantTokenError::Configuration(
                        "OIDC bearer auth requires AUTH_JWT_JWKS_URL or AUTH_JWT_DISCOVERY_URL"
                            .to_string(),
                    )
                });
        };

        let metadata = self.fetch_provider_metadata(discovery_url).await?;
        if metadata.issuer != self.config.issuer.as_ref() {
            return Err(TenantTokenError::Discovery(format!(
                "discovery issuer {} does not match configured issuer {}",
                metadata.issuer, self.config.issuer
            )));
        }

        if let Some(expected_jwks_url) = self.config.jwks_url.as_deref() {
            if metadata.jwks_uri != expected_jwks_url {
                return Err(TenantTokenError::Discovery(format!(
                    "discovery jwks_uri {} does not match configured AUTH_JWT_JWKS_URL {}",
                    metadata.jwks_uri, expected_jwks_url
                )));
            }
        }

        Ok(metadata.jwks_uri)
    }

    async fn fetch_provider_metadata(
        &self,
        discovery_url: &str,
    ) -> Result<OpenIdProviderMetadata, TenantTokenError> {
        let response = self
            .http_client
            .get(discovery_url)
            .send()
            .await
            .map_err(|err| TenantTokenError::Discovery(format!("discovery fetch failed: {err}")))?;

        if !response.status().is_success() {
            return Err(TenantTokenError::Discovery(format!(
                "discovery fetch failed with HTTP {}",
                response.status()
            )));
        }

        response
            .json::<OpenIdProviderMetadata>()
            .await
            .map_err(|err| {
                TenantTokenError::Discovery(format!("invalid discovery document: {err}"))
            })
    }
}

fn bearer_token(request: &Request) -> Option<&str> {
    request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn validate_legacy_bearer_token(
    token: &str,
    secret: &[u8],
) -> Result<ValidatedClaims, &'static str> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("expected 3 JWT segments");
    }

    let header: LegacyJwtHeader = decode_part(parts[0]).map_err(|()| "malformed JWT header")?;
    if header.alg != "HS256" {
        return Err("unsupported JWT algorithm");
    }

    let signature = URL_SAFE_NO_PAD
        .decode(parts[2])
        .map_err(|_| "malformed JWT signature")?;
    let signing_input = format!("{}.{}", parts[0], parts[1]);
    let key = hmac::Key::new(hmac::HMAC_SHA256, secret);
    hmac::verify(&key, signing_input.as_bytes(), &signature)
        .map_err(|_| "signature verification failed")?;

    let claims: LegacyJwtClaims = decode_part(parts[1]).map_err(|()| "malformed JWT claims")?;
    let now = Utc::now().timestamp();

    let exp = claims.exp.ok_or("missing exp claim")?;
    if now >= exp {
        return Err("token expired");
    }

    if let Some(nbf) = claims.nbf {
        if now < nbf {
            return Err("token not yet valid");
        }
    }

    validated_claims(
        claims.tenant_id.or(claims.tid),
        claims.sub,
        claims.scope,
        claims.scp,
        claims.roles,
        claims.role,
        claims.realm_access,
        claims.client_id,
        claims.azp,
    )
}

#[allow(clippy::too_many_arguments)]
fn validated_claims(
    tenant_id: Option<String>,
    subject: Option<String>,
    scope: Option<String>,
    scp: Option<Vec<String>>,
    roles: Option<Vec<String>>,
    role: Option<String>,
    realm_access: Option<RealmAccess>,
    client_id: Option<String>,
    authorized_party: Option<String>,
) -> Result<ValidatedClaims, &'static str> {
    let tenant_id = tenant_id.ok_or("missing tenant claim")?;
    if !tenant_id.starts_with("TNT_") {
        return Err("invalid tenant claim");
    }

    let scopes = parse_scopes(scope, scp);
    let roles = parse_roles(roles, role, realm_access);

    Ok(ValidatedClaims {
        tenant_id,
        subject,
        roles,
        scopes,
        client_id,
        authorized_party,
    })
}

fn parse_scopes(scope: Option<String>, scp: Option<Vec<String>>) -> BTreeSet<String> {
    let mut parsed = BTreeSet::new();

    if let Some(scope) = scope {
        for item in scope.split_whitespace() {
            if !item.is_empty() {
                parsed.insert(item.to_string());
            }
        }
    }

    if let Some(scp) = scp {
        for item in scp {
            if !item.trim().is_empty() {
                parsed.insert(item);
            }
        }
    }

    parsed
}

fn parse_roles(
    roles: Option<Vec<String>>,
    role: Option<String>,
    realm_access: Option<RealmAccess>,
) -> BTreeSet<Role> {
    let mut parsed = BTreeSet::new();

    let mut role_values = Vec::new();
    if let Some(roles) = roles {
        role_values.extend(roles);
    }
    if let Some(role) = role {
        role_values.push(role);
    }
    if let Some(realm_access) = realm_access {
        role_values.extend(realm_access.roles);
    }

    for value in role_values {
        if let Some(role) = Role::from_claim(&value) {
            parsed.insert(role);
        } else {
            warn!("Ignoring unrecognized auth role claim: {}", value);
        }
    }

    parsed
}

fn decode_part<T: for<'de> Deserialize<'de>>(part: &str) -> Result<T, ()> {
    let bytes = URL_SAFE_NO_PAD.decode(part).map_err(|_| ())?;
    serde_json::from_slice(&bytes).map_err(|_| ())
}

fn select_jwk<'a>(jwks: &'a JwkSet, kid: Option<&str>) -> Result<&'a Jwk, String> {
    if let Some(kid) = kid {
        return jwks
            .find(kid)
            .ok_or_else(|| format!("no JWK found for kid {kid}"));
    }

    match jwks.keys.as_slice() {
        [single] => Ok(single),
        [] => Err("JWKS does not contain any keys".to_string()),
        _ => Err("JWKS contains multiple keys but the JWT header has no kid".to_string()),
    }
}

fn auth_action(security: &SecurityConfig) -> &'static str {
    if security.bearer_oidc.is_some() {
        "Present a valid tenant-scoped bearer token issued by the configured identity provider"
    } else if security.bearer_hs256_secret.is_some() {
        "Present a valid tenant-scoped HS256 bearer token or migrate to issuer/JWKS-backed bearer auth"
    } else {
        "Configure AUTH_JWT_ISSUER/AUTH_JWT_AUDIENCE plus AUTH_JWT_JWKS_URL (or AUTH_JWT_DISCOVERY_URL), set AUTH_JWT_HS256_SECRET for the explicit legacy fallback, or enable the dev tenant header explicitly"
    }
}

fn bad_request(message: &str, action: &str) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "error": {
                "code": "SAHI_1100",
                "message": message,
                "action": action
            }
        })),
    )
        .into_response()
}

fn unauthorized(message: &str, action: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": {
                "code": "SAHI_1100",
                "message": message,
                "action": action
            }
        })),
    )
        .into_response()
}
