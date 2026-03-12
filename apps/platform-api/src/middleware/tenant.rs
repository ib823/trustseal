use std::{
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
use serde::Deserialize;
use serde_json::json;
use tokio::sync::RwLock;

use crate::state::{AppState, OidcBearerAuthConfig, SecurityConfig};

pub type SharedTenantTokenValidator = Arc<TenantTokenValidator>;

/// Middleware: extract tenant ID from JWT claims (X-Tenant-Id header for dev).
///
/// In production, this reads from validated JWT claims.
/// For development, it accepts X-Tenant-Id header directly.
///
/// Sets the tenant context for downstream RLS enforcement.
pub async fn extract_tenant(
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

        request
            .extensions_mut()
            .insert(TenantId(claims.tenant_id.clone()));
        request.extensions_mut().insert(AuthSubject(claims.subject));
        return next.run(request).await;
    }

    if !state.security.allow_insecure_dev_tenant_header {
        return next.run(request).await;
    }

    let tenant_id = request
        .headers()
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
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

/// Middleware: require tenant ID on protected routes.
pub async fn require_tenant(request: Request, next: Next) -> Response {
    if request.extensions().get::<TenantId>().is_none() {
        return unauthorized(
            "Tenant context required",
            "Authenticate with a tenant-scoped bearer token or explicitly enable the dev tenant header",
        );
    }

    next.run(request).await
}

/// Extracted from request extensions to access the current tenant.
#[derive(Debug, Clone)]
pub struct TenantId(pub String);

/// Subject extracted from a validated bearer token, if present.
#[derive(Debug, Clone)]
pub struct AuthSubject(pub Option<String>);

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
}

#[derive(Debug)]
struct ValidatedClaims {
    tenant_id: String,
    subject: Option<String>,
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

        validated_claims(claims.tenant_id.or(claims.tid), claims.sub)
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

    validated_claims(claims.tenant_id.or(claims.tid), claims.sub)
}

fn validated_claims(
    tenant_id: Option<String>,
    subject: Option<String>,
) -> Result<ValidatedClaims, &'static str> {
    let tenant_id = tenant_id.ok_or("missing tenant claim")?;
    if !tenant_id.starts_with("TNT_") {
        return Err("invalid tenant claim");
    }

    Ok(ValidatedClaims { tenant_id, subject })
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};
    use jsonwebtoken::{encode, EncodingKey};
    use serde::Serialize;
    use tokio::net::TcpListener;

    const TEST_RSA_PRIVATE_KEY: &str = r"-----BEGIN PRIVATE KEY-----
MIIEvwIBADANBgkqhkiG9w0BAQEFAASCBKkwggSlAgEAAoIBAQDrT6BDDaOtHL3m
04Ju6Xs2hX3E0il/vsLnMtxIBV4KIo3Q7dDy1cE2JJLgvS81bku2mg14PltnTwmQ
RLfwi5KSAjvN8jc/b2Qir+ZaWr1PaqHshL4lfXEvLg0qC1Ob4JHOMPq86Kxd8+5C
QuCvE6bWZZQkvtxsleXo21qpdMcwNEXAw4xmaKej8tGwxLACoE3AG+LQtIKd/xCt
Y6OMdiJNPh/itrVIXJOuur5FL7/+PhftajehWSAGI0ncmoWt7yce/OdNC9LYzdJZ
HBLOOyd7WRoPI6O9YCkxJ0sBzz9Od7EtY/Play2atZYZ+pTx52regA8f4y9NZaX5
SCh7skE3AgMBAAECggEARm2Dmu4XIfXrRI3jiQyqiwvzM5hvQUO4E/ieA5RPrBrh
dTnogvXFKU5TA5675XMIiDOdenK3ark2NI7MutsbWEYA3kfzjzHot5UMDdkAtidF
JYQpYRElciiHWiEfuhTBrwPr1+SPymL62awokV4BXkPyzfiuAnXu5P3aKcPA5kT7
7v+788wsS5RbBKM4cZZxz8R9K3mG189jHk/nthoe1a3g18SDWOhGuGCHY6t3Wwqw
qBomWRdyturL97bzO4ECdszEnbQCs6Nwe8qiNeZv4hQOwyO+EHVnBgdO4hGg9yfU
M63hDcOE9/oVeOj5g3h7RjjdsAf/vhehlW6tWA0p+QKBgQD8BNxhHoD+AZAcnUZt
F4kDhDUe4atjlkDefHZ53I2HjEjRBS18q8bPc+D+Iex1pUQ0KvlQAB/n4VyvgKGL
FJXZWJszFpNO/t3zM9vJEOMnqMkFo15eaCgNnIHzrN+4MUFR2u5vxH0fFk4ISc9u
poEnz92/PDxL7GviDmEZQEvllQKBgQDvBzMuGyaOKimCTpKy1c5NYGbdM47/JEUb
jRgMhH1Fn4ZTm4tY1GWjb8h8nwDmAMZ4xKmKkkGjz8Qz+ovRrW55LDsAaaz2chzR
iwlDleT7SXkwopr0wH6GomJtuQguXlc7DyPERF6dMyxE9hOZgUm/keAcYJvI2C5G
T8kEurVAmwKBgQDYEGbMkPVwT/C4x5IIl4PtUtykFD/3SmtlE/oTMibYzknjgffk
ifUSCLwdxQHQPxeBTlKe5uxzxb/L65EUB1sNkyzEGRfEQzgQeSZ2dJb7enaV8eFH
OS8VtFepjU3kwb3JqtR/WEsZausNqhJAQFo8wrbPbJoZUaGQllli57/qHQKBgQC9
UBSHFdXpjxclL5ocrh4hRpLx6138UfuyIIPFlkGpnPlEytMI3eBKG8TMfxq1EDQh
fpFRQRlf5rRc/rkyrovqyM9KOmhVIHgWtmn174hWRhEIJiFYbAVKGN6gTIZgQzQP
gKQxVH0jQF51l/haAf4pDh5UG2gHIME1ywdJCZ94tQKBgQCRse2FoqttupDL6c85
YFPcUL0oiOxF7CZMNVkkVj3NIWi4U/vOBGCxlWdKX51J2KH52TOBX91rJkGKHDPb
qiPQCg3fkgKIZMQAlViyuf3u2JENBN1btSgTeURSm8iWaMvvkHeddjlaXj8WNTyv
Wi7dkXvR/oAS6t9kJDwXKfCJ+A==
-----END PRIVATE KEY-----";

    #[derive(Debug, Serialize)]
    struct TestBearerClaims<'a> {
        iss: &'a str,
        aud: &'a str,
        exp: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        nbf: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        iat: Option<i64>,
        tenant_id: &'a str,
        sub: &'a str,
    }

    fn sign_hs256(payload: &serde_json::Value, secret: &[u8]) -> String {
        let header = json!({"alg": "HS256", "typ": "JWT"});
        let header_b64 = URL_SAFE_NO_PAD.encode(header.to_string());
        let payload_b64 = URL_SAFE_NO_PAD.encode(payload.to_string());
        let signing_input = format!("{header_b64}.{payload_b64}");
        let key = hmac::Key::new(hmac::HMAC_SHA256, secret);
        let signature = hmac::sign(&key, signing_input.as_bytes());
        let signature_b64 = URL_SAFE_NO_PAD.encode(signature.as_ref());
        format!("{signing_input}.{signature_b64}")
    }

    fn sign_rs256(claims: &TestBearerClaims<'_>, issuer_kid: &str) -> String {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(issuer_kid.to_string());

        encode(
            &header,
            claims,
            &EncodingKey::from_rsa_pem(TEST_RSA_PRIVATE_KEY.as_bytes())
                .expect("test RSA key should parse"),
        )
        .expect("should encode RS256 JWT")
    }

    async fn spawn_oidc_server(issuer: &str) -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test listener");
        let addr = listener.local_addr().expect("listener addr");
        let base_url = format!("http://{addr}");
        let discovery = json!({
            "issuer": issuer,
            "jwks_uri": format!("{base_url}/jwks.json"),
        });
        let jwks = json!({
            "keys": [{
                "kty": "RSA",
                "n": "60-gQw2jrRy95tOCbul7NoV9xNIpf77C5zLcSAVeCiKN0O3Q8tXBNiSS4L0vNW5LtpoNeD5bZ08JkES38IuSkgI7zfI3P29kIq_mWlq9T2qh7IS-JX1xLy4NKgtTm-CRzjD6vOisXfPuQkLgrxOm1mWUJL7cbJXl6NtaqXTHMDRFwMOMZmino_LRsMSwAqBNwBvi0LSCnf8QrWOjjHYiTT4f4ra1SFyTrrq-RS-__j4X7Wo3oVkgBiNJ3JqFre8nHvznTQvS2M3SWRwSzjsne1kaDyOjvWApMSdLAc8_TnexLWPz5WstmrWWGfqU8edq3oAPH-MvTWWl-Ugoe7JBNw",
                "e": "AQAB",
                "use": "sig",
                "alg": "RS256",
                "kid": "test-kid"
            }]
        });

        let app = Router::new()
            .route(
                "/.well-known/openid-configuration",
                get({
                    let discovery = discovery.clone();
                    move || {
                        let discovery = discovery.clone();
                        async move { Json(discovery) }
                    }
                }),
            )
            .route(
                "/jwks.json",
                get({
                    let jwks = jwks.clone();
                    move || {
                        let jwks = jwks.clone();
                        async move { Json(jwks) }
                    }
                }),
            );

        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("test OIDC server");
        });

        (base_url, handle)
    }

    #[test]
    fn validates_legacy_hs256_bearer_token() {
        let now = Utc::now().timestamp();
        let payload = json!({
            "tenant_id": "TNT_test",
            "sub": "USR_123",
            "exp": now + 300,
            "nbf": now - 10
        });
        let token = sign_hs256(&payload, b"super-secret");

        let claims = validate_legacy_bearer_token(&token, b"super-secret").unwrap();
        assert_eq!(claims.tenant_id, "TNT_test");
        assert_eq!(claims.subject.as_deref(), Some("USR_123"));
    }

    #[test]
    fn rejects_expired_legacy_hs256_bearer_token() {
        let now = Utc::now().timestamp();
        let payload = json!({
            "tenant_id": "TNT_test",
            "exp": now - 1
        });
        let token = sign_hs256(&payload, b"super-secret");

        assert_eq!(
            validate_legacy_bearer_token(&token, b"super-secret").unwrap_err(),
            "token expired"
        );
    }

    #[test]
    fn rejects_wrong_legacy_hs256_signature() {
        let now = Utc::now().timestamp();
        let payload = json!({
            "tenant_id": "TNT_test",
            "exp": now + 300
        });
        let token = sign_hs256(&payload, b"secret-a");

        assert_eq!(
            validate_legacy_bearer_token(&token, b"secret-b").unwrap_err(),
            "signature verification failed"
        );
    }

    #[tokio::test]
    async fn validates_rs256_bearer_token_via_discovery_and_jwks() {
        let issuer = "https://issuer.example";
        let (base_url, handle) = spawn_oidc_server(issuer).await;
        let now = Utc::now().timestamp();
        let validator = TenantTokenValidator::new(OidcBearerAuthConfig {
            issuer: Arc::<str>::from(issuer),
            audience: vec![Arc::<str>::from("platform-api")].into(),
            jwks_url: None,
            discovery_url: Some(format!("{base_url}/.well-known/openid-configuration").into()),
            allowed_algs: vec![Algorithm::RS256].into(),
            jwks_cache_ttl: Duration::from_secs(300),
        })
        .expect("validator");

        let token = sign_rs256(
            &TestBearerClaims {
                iss: issuer,
                aud: "platform-api",
                exp: now + 300,
                nbf: Some(now - 10),
                iat: Some(now),
                tenant_id: "TNT_test",
                sub: "USR_123",
            },
            "test-kid",
        );

        let claims = validator.validate_bearer_token(&token).await.unwrap();
        assert_eq!(claims.tenant_id, "TNT_test");
        assert_eq!(claims.subject.as_deref(), Some("USR_123"));

        handle.abort();
    }

    #[tokio::test]
    async fn reuses_cached_jwks_between_validations() {
        let issuer = "https://issuer.example";
        let (base_url, handle) = spawn_oidc_server(issuer).await;
        let now = Utc::now().timestamp();
        let validator = TenantTokenValidator::new(OidcBearerAuthConfig {
            issuer: Arc::<str>::from(issuer),
            audience: vec![Arc::<str>::from("platform-api")].into(),
            jwks_url: None,
            discovery_url: Some(format!("{base_url}/.well-known/openid-configuration").into()),
            allowed_algs: vec![Algorithm::RS256].into(),
            jwks_cache_ttl: Duration::from_secs(300),
        })
        .expect("validator");

        let token = sign_rs256(
            &TestBearerClaims {
                iss: issuer,
                aud: "platform-api",
                exp: now + 300,
                nbf: Some(now - 10),
                iat: Some(now),
                tenant_id: "TNT_test",
                sub: "USR_123",
            },
            "test-kid",
        );

        validator.validate_bearer_token(&token).await.unwrap();
        handle.abort();
        validator.validate_bearer_token(&token).await.unwrap();
    }

    #[tokio::test]
    async fn rejects_wrong_audience_for_rs256_bearer_token() {
        let issuer = "https://issuer.example";
        let (base_url, handle) = spawn_oidc_server(issuer).await;
        let now = Utc::now().timestamp();
        let validator = TenantTokenValidator::new(OidcBearerAuthConfig {
            issuer: Arc::<str>::from(issuer),
            audience: vec![Arc::<str>::from("platform-api")].into(),
            jwks_url: None,
            discovery_url: Some(format!("{base_url}/.well-known/openid-configuration").into()),
            allowed_algs: vec![Algorithm::RS256].into(),
            jwks_cache_ttl: Duration::from_secs(300),
        })
        .expect("validator");

        let token = sign_rs256(
            &TestBearerClaims {
                iss: issuer,
                aud: "another-audience",
                exp: now + 300,
                nbf: Some(now - 10),
                iat: Some(now),
                tenant_id: "TNT_test",
                sub: "USR_123",
            },
            "test-kid",
        );

        let err = validator.validate_bearer_token(&token).await.unwrap_err();
        assert!(err.to_string().contains("Audience"));

        handle.abort();
    }
}
