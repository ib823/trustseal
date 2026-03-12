//! MyDigital ID OAuth 2.0 client for eKYC.
//!
//! Implements OAuth 2.0 with PKCE for Malaysian government identity verification.
//! Reference: MyDigital ID API documentation (Malaysian government identity platform).

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use jsonwebtoken::{
    decode, decode_header, jwk::JwkSet, Algorithm, DecodingKey, Header, Validation,
};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha384, Sha512};

use super::pkce::PkceParams;
use super::types::{AssuranceLevel, TokenResponse, UserInfoResponse, VerifiedClaims};

/// MyDigital ID OAuth configuration.
#[derive(Debug, Clone)]
pub struct MyDigitalIdConfig {
    /// OAuth client ID.
    pub client_id: String,
    /// OAuth client secret (optional for public clients).
    pub client_secret: Option<String>,
    /// Authorization endpoint URL.
    pub authorization_url: String,
    /// Token endpoint URL.
    pub token_url: String,
    /// User info endpoint URL.
    pub userinfo_url: String,
    /// Expected OpenID Provider issuer identifier.
    pub issuer: String,
    /// JWKS URL for ID-token signature verification.
    pub jwks_url: Option<String>,
    /// Redirect URI (callback URL).
    pub redirect_uri: String,
    /// OAuth scopes to request.
    pub scope: String,
}

impl MyDigitalIdConfig {
    /// Create configuration from environment variables.
    pub fn from_env() -> Result<Self, MyDigitalIdError> {
        Ok(Self {
            client_id: std::env::var("MYDIGITAL_CLIENT_ID")
                .map_err(|_| MyDigitalIdError::MissingConfig("MYDIGITAL_CLIENT_ID"))?,
            client_secret: std::env::var("MYDIGITAL_CLIENT_SECRET").ok(),
            authorization_url: std::env::var("MYDIGITAL_AUTH_URL")
                .unwrap_or_else(|_| "https://api.mydigital.gov.my/oauth2/authorize".to_string()),
            token_url: std::env::var("MYDIGITAL_TOKEN_URL")
                .unwrap_or_else(|_| "https://api.mydigital.gov.my/oauth2/token".to_string()),
            userinfo_url: std::env::var("MYDIGITAL_USERINFO_URL")
                .unwrap_or_else(|_| "https://api.mydigital.gov.my/oauth2/userinfo".to_string()),
            issuer: std::env::var("MYDIGITAL_ISSUER")
                .map_err(|_| MyDigitalIdError::MissingConfig("MYDIGITAL_ISSUER"))?,
            jwks_url: std::env::var("MYDIGITAL_JWKS_URL").ok(),
            redirect_uri: std::env::var("MYDIGITAL_REDIRECT_URI")
                .map_err(|_| MyDigitalIdError::MissingConfig("MYDIGITAL_REDIRECT_URI"))?,
            scope: ensure_openid_scope(
                &std::env::var("MYDIGITAL_SCOPE")
                    .unwrap_or_else(|_| "openid profile ic_number".to_string()),
            ),
        })
    }

    /// Create mock configuration for testing.
    #[cfg(test)]
    pub fn mock() -> Self {
        Self {
            client_id: "test_client_id".to_string(),
            client_secret: Some("test_client_secret".to_string()),
            authorization_url: "https://mock.mydigital.gov.my/oauth2/authorize".to_string(),
            token_url: "https://mock.mydigital.gov.my/oauth2/token".to_string(),
            userinfo_url: "https://mock.mydigital.gov.my/oauth2/userinfo".to_string(),
            issuer: "https://mock.mydigital.gov.my".to_string(),
            jwks_url: None,
            redirect_uri: "vaultpass://callback".to_string(),
            scope: "openid profile ic_number".to_string(),
        }
    }
}

/// MyDigital ID OAuth client.
pub struct MyDigitalIdClient {
    config: MyDigitalIdConfig,
    #[allow(dead_code)]
    http_client: reqwest::Client,
}

impl MyDigitalIdClient {
    /// Create a new client with the given configuration.
    pub fn new(config: MyDigitalIdConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
        }
    }

    /// Build the authorization URL for the OAuth flow.
    ///
    /// Returns the URL, PKCE parameters, and OIDC nonce that must be stored for the callback.
    pub fn build_authorization_url(
        &self,
    ) -> Result<(String, PkceParams, String), MyDigitalIdError> {
        let pkce = PkceParams::generate().map_err(|_| MyDigitalIdError::PkceGenerationFailed)?;
        let nonce = generate_nonce()?;

        let url = format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&nonce={}&code_challenge={}&code_challenge_method={}",
            self.config.authorization_url,
            urlencoding::encode(&self.config.client_id),
            urlencoding::encode(&self.config.redirect_uri),
            urlencoding::encode(&self.config.scope),
            urlencoding::encode(&pkce.state),
            urlencoding::encode(&nonce),
            urlencoding::encode(&pkce.code_challenge),
            pkce.code_challenge_method,
        );

        Ok((url, pkce, nonce))
    }

    /// Exchange authorization code for tokens.
    ///
    /// # Arguments
    ///
    /// * `code` - Authorization code from callback.
    /// * `code_verifier` - PKCE code verifier from initial request.
    pub async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<TokenResponse, MyDigitalIdError> {
        let mut params = vec![
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.config.redirect_uri),
            ("client_id", &self.config.client_id),
            ("code_verifier", code_verifier),
        ];

        // Add client secret if configured (confidential client)
        let secret_ref;
        if let Some(ref secret) = self.config.client_secret {
            secret_ref = secret.as_str();
            params.push(("client_secret", secret_ref));
        }

        let response = self
            .http_client
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| MyDigitalIdError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(MyDigitalIdError::TokenExchangeFailed(format!(
                "{status}: {body}"
            )));
        }

        response
            .json::<TokenResponse>()
            .await
            .map_err(|e| MyDigitalIdError::InvalidResponse(e.to_string()))
    }

    /// Validate the OIDC ID token returned from the token endpoint.
    pub async fn validate_id_token(
        &self,
        id_token: &str,
        access_token: &str,
        expected_nonce: &str,
    ) -> Result<IdTokenClaims, MyDigitalIdError> {
        let header =
            decode_header(id_token).map_err(|e| MyDigitalIdError::InvalidIdToken(e.to_string()))?;
        let decoding_key = self.decoding_key_for_id_token(&header).await?;

        let mut validation = Validation::new(header.alg);
        validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
        validation.set_issuer(&[self.config.issuer.as_str()]);
        validation.set_audience(&[self.config.client_id.as_str()]);
        validation.leeway = 60;

        let token_data = decode::<IdTokenClaims>(id_token, &decoding_key, &validation)
            .map_err(|e| MyDigitalIdError::InvalidIdToken(e.to_string()))?;
        let claims = token_data.claims;

        if claims.nonce.as_deref() != Some(expected_nonce) {
            return Err(MyDigitalIdError::NonceMismatch);
        }

        claims.validate_authorized_party(&self.config.client_id)?;

        if let Some(expected_at_hash) = claims.at_hash.as_deref() {
            validate_at_hash(header.alg, access_token, expected_at_hash)?;
        }

        Ok(claims)
    }

    /// Fetch user info using access token.
    pub async fn fetch_user_info(
        &self,
        access_token: &str,
    ) -> Result<UserInfoResponse, MyDigitalIdError> {
        let response = self
            .http_client
            .get(&self.config.userinfo_url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| MyDigitalIdError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(MyDigitalIdError::UserInfoFailed(format!(
                "{status}: {body}"
            )));
        }

        response
            .json::<UserInfoResponse>()
            .await
            .map_err(|e| MyDigitalIdError::InvalidResponse(e.to_string()))
    }

    /// Process user info into verified claims.
    ///
    /// IMPORTANT: This hashes all PII before returning. Raw values are never stored.
    #[allow(clippy::unused_self)]
    pub fn process_claims(
        &self,
        user_info: &UserInfoResponse,
    ) -> Result<VerifiedClaims, MyDigitalIdError> {
        let name = user_info
            .name
            .as_ref()
            .ok_or(MyDigitalIdError::MissingClaim("name"))?;

        let ic_number = user_info
            .ic_number
            .as_ref()
            .ok_or(MyDigitalIdError::MissingClaim("ic_number"))?;

        // Hash PII (never store raw values)
        let name_hash = hash_claim(&normalize_name(name));
        let ic_hash = hash_claim(&normalize_ic(ic_number));

        // Determine assurance level based on verification status
        let assurance_level = if user_info.verified.unwrap_or(false) {
            AssuranceLevel::P2 // eKYC-verified
        } else {
            AssuranceLevel::P1 // Self-asserted
        };

        Ok(VerifiedClaims {
            name_hash,
            ic_hash,
            assurance_level,
        })
    }

    #[must_use]
    pub fn redirect_uri(&self) -> &str {
        &self.config.redirect_uri
    }

    #[must_use]
    pub fn scope(&self) -> &str {
        &self.config.scope
    }

    pub(crate) fn ensure_matching_subject(
        id_token_claims: &IdTokenClaims,
        user_info: &UserInfoResponse,
    ) -> Result<(), MyDigitalIdError> {
        if id_token_claims.sub != user_info.sub {
            return Err(MyDigitalIdError::SubjectMismatch);
        }

        Ok(())
    }

    async fn decoding_key_for_id_token(
        &self,
        header: &Header,
    ) -> Result<DecodingKey, MyDigitalIdError> {
        match header.alg {
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => self
                .config
                .client_secret
                .as_deref()
                .map(|secret| DecodingKey::from_secret(secret.as_bytes()))
                .ok_or(MyDigitalIdError::MissingClientSecret),
            Algorithm::RS256
            | Algorithm::RS384
            | Algorithm::RS512
            | Algorithm::ES256
            | Algorithm::ES384 => {
                let jwks = self.fetch_jwks().await?;
                let jwk = select_jwk(&jwks, header.kid.as_deref())?;
                DecodingKey::try_from(jwk).map_err(|e| MyDigitalIdError::InvalidJwks(e.to_string()))
            }
            unsupported => Err(MyDigitalIdError::UnsupportedIdTokenAlgorithm(format!(
                "{unsupported:?}"
            ))),
        }
    }

    async fn fetch_jwks(&self) -> Result<JwkSet, MyDigitalIdError> {
        let jwks_url = self
            .config
            .jwks_url
            .as_deref()
            .ok_or(MyDigitalIdError::MissingJwksUrl)?;

        let response = self
            .http_client
            .get(jwks_url)
            .send()
            .await
            .map_err(|e| MyDigitalIdError::JwksFetchFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(MyDigitalIdError::JwksFetchFailed(format!(
                "HTTP {}",
                response.status()
            )));
        }

        response
            .json::<JwkSet>()
            .await
            .map_err(|e| MyDigitalIdError::InvalidJwks(e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdTokenClaims {
    pub iss: String,
    pub sub: String,
    pub aud: AudienceClaim,
    pub exp: i64,
    #[serde(default)]
    pub iat: Option<i64>,
    #[serde(default)]
    pub nonce: Option<String>,
    #[serde(default)]
    pub azp: Option<String>,
    #[serde(default, rename = "at_hash")]
    pub at_hash: Option<String>,
}

impl IdTokenClaims {
    fn validate_authorized_party(&self, client_id: &str) -> Result<(), MyDigitalIdError> {
        match &self.aud {
            AudienceClaim::One(_) => {
                if let Some(azp) = self.azp.as_deref() {
                    if azp != client_id {
                        return Err(MyDigitalIdError::InvalidIdToken(
                            "azp does not match client_id".to_string(),
                        ));
                    }
                }
            }
            AudienceClaim::Many(values) => {
                if values.len() > 1 && self.azp.as_deref() != Some(client_id) {
                    return Err(MyDigitalIdError::InvalidIdToken(
                        "azp is required and must match client_id when aud has multiple values"
                            .to_string(),
                    ));
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AudienceClaim {
    One(String),
    Many(Vec<String>),
}

/// Hash a claim value using SHA-256.
fn hash_claim(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let hash = hasher.finalize();
    URL_SAFE_NO_PAD.encode(hash)
}

fn generate_nonce() -> Result<String, MyDigitalIdError> {
    let rng = SystemRandom::new();
    let mut nonce = [0u8; 32];
    rng.fill(&mut nonce)
        .map_err(|_| MyDigitalIdError::NonceGenerationFailed)?;
    Ok(URL_SAFE_NO_PAD.encode(nonce))
}

fn ensure_openid_scope(scope: &str) -> String {
    let mut scopes: Vec<String> = scope
        .split_whitespace()
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect();
    if !scopes.iter().any(|value| value == "openid") {
        scopes.insert(0, "openid".to_string());
    }
    scopes.join(" ")
}

fn select_jwk<'a>(
    jwks: &'a JwkSet,
    kid: Option<&str>,
) -> Result<&'a jsonwebtoken::jwk::Jwk, MyDigitalIdError> {
    if let Some(kid) = kid {
        return jwks
            .find(kid)
            .ok_or_else(|| MyDigitalIdError::InvalidJwks(format!("No JWK found for kid {kid}")));
    }

    match jwks.keys.as_slice() {
        [single] => Ok(single),
        [] => Err(MyDigitalIdError::InvalidJwks(
            "JWKS does not contain any keys".to_string(),
        )),
        _ => Err(MyDigitalIdError::InvalidJwks(
            "JWKS contains multiple keys but the ID token header has no kid".to_string(),
        )),
    }
}

fn validate_at_hash(
    algorithm: Algorithm,
    access_token: &str,
    expected_at_hash: &str,
) -> Result<(), MyDigitalIdError> {
    let digest = match algorithm {
        Algorithm::HS256 | Algorithm::RS256 | Algorithm::ES256 => {
            Sha256::digest(access_token).to_vec()
        }
        Algorithm::HS384 | Algorithm::RS384 | Algorithm::ES384 => {
            Sha384::digest(access_token).to_vec()
        }
        Algorithm::HS512 | Algorithm::RS512 => Sha512::digest(access_token).to_vec(),
        unsupported => {
            return Err(MyDigitalIdError::UnsupportedIdTokenAlgorithm(format!(
                "{unsupported:?}"
            )))
        }
    };

    let expected = URL_SAFE_NO_PAD.encode(&digest[..digest.len() / 2]);
    if expected != expected_at_hash {
        return Err(MyDigitalIdError::InvalidIdToken(
            "at_hash does not match access token".to_string(),
        ));
    }

    Ok(())
}

/// Normalize a name for hashing (lowercase, trim, remove extra spaces).
fn normalize_name(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Normalize an IC number for hashing (digits only).
fn normalize_ic(ic: &str) -> String {
    ic.chars().filter(char::is_ascii_digit).collect()
}

/// MyDigital ID errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum MyDigitalIdError {
    #[error("Missing configuration: {0}")]
    MissingConfig(&'static str),
    #[error("Client secret is required for HS-based ID token validation")]
    MissingClientSecret,
    #[error("JWKS URL is required for asymmetric ID token validation")]
    MissingJwksUrl,
    #[error("PKCE generation failed")]
    PkceGenerationFailed,
    #[error("Nonce generation failed")]
    NonceGenerationFailed,
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Token exchange failed: {0}")]
    TokenExchangeFailed(String),
    #[error("User info fetch failed: {0}")]
    UserInfoFailed(String),
    #[error("JWKS fetch failed: {0}")]
    JwksFetchFailed(String),
    #[error("Invalid JWKS: {0}")]
    InvalidJwks(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("Invalid ID token: {0}")]
    InvalidIdToken(String),
    #[error("Unsupported ID token algorithm: {0}")]
    UnsupportedIdTokenAlgorithm(String),
    #[error("Missing claim: {0}")]
    MissingClaim(&'static str),
    #[error("State mismatch")]
    StateMismatch,
    #[error("Token endpoint did not return a bearer token: {0}")]
    UnsupportedTokenType(String),
    #[error("ID token is missing from token response")]
    MissingIdToken,
    #[error("ID token nonce mismatch")]
    NonceMismatch,
    #[error("UserInfo subject does not match ID token subject")]
    SubjectMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use jsonwebtoken::{encode, EncodingKey};

    fn make_test_id_token(config: &MyDigitalIdConfig, nonce: &str, access_token: &str) -> String {
        let claims = IdTokenClaims {
            iss: config.issuer.clone(),
            sub: "user123".to_string(),
            aud: AudienceClaim::One(config.client_id.clone()),
            exp: Utc::now().timestamp() + 300,
            iat: Some(Utc::now().timestamp()),
            nonce: Some(nonce.to_string()),
            azp: None,
            at_hash: Some(
                URL_SAFE_NO_PAD.encode(&Sha256::digest(access_token)[..Sha256::output_size() / 2]),
            ),
        };

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(config.client_secret.as_deref().unwrap().as_bytes()),
        )
        .expect("Should encode HS256 ID token")
    }

    #[test]
    fn test_normalize_name() {
        assert_eq!(normalize_name("  John   Doe  "), "john doe");
        assert_eq!(normalize_name("AHMAD BIN ALI"), "ahmad bin ali");
        assert_eq!(normalize_name("Mary Jane"), "mary jane");
    }

    #[test]
    fn test_normalize_ic() {
        assert_eq!(normalize_ic("900101-14-5678"), "900101145678");
        assert_eq!(normalize_ic("900101 14 5678"), "900101145678");
        assert_eq!(normalize_ic("900101145678"), "900101145678");
    }

    #[test]
    fn test_hash_claim() {
        let hash1 = hash_claim("test");
        let hash2 = hash_claim("test");
        let hash3 = hash_claim("different");

        // Same input produces same hash
        assert_eq!(hash1, hash2);
        // Different input produces different hash
        assert_ne!(hash1, hash3);
        // Hash is 43 chars (32-byte SHA256 -> Base64URL)
        assert_eq!(hash1.len(), 43);
    }

    #[test]
    fn test_build_authorization_url() {
        let config = MyDigitalIdConfig::mock();
        let client = MyDigitalIdClient::new(config);

        let (url, pkce, nonce) = client.build_authorization_url().expect("Should build URL");

        assert!(url.contains("response_type=code"));
        assert!(url.contains("client_id=test_client_id"));
        assert!(url.contains(&format!("state={}", pkce.state)));
        assert!(url.contains(&format!("nonce={nonce}")));
        assert!(url.contains(&format!("code_challenge={}", pkce.code_challenge)));
        assert!(url.contains("code_challenge_method=S256"));
    }

    #[test]
    fn test_process_claims() {
        let config = MyDigitalIdConfig::mock();
        let client = MyDigitalIdClient::new(config);

        let user_info = UserInfoResponse {
            sub: "user123".to_string(),
            name: Some("Ahmad Bin Ali".to_string()),
            ic_number: Some("900101-14-5678".to_string()),
            verified: Some(true),
        };

        let claims = client
            .process_claims(&user_info)
            .expect("Should process claims");

        // Name is hashed
        assert_eq!(claims.name_hash.len(), 43);
        // IC is hashed
        assert_eq!(claims.ic_hash.len(), 43);
        // Verified user gets P2
        assert_eq!(claims.assurance_level, AssuranceLevel::P2);
    }

    #[test]
    fn test_process_claims_missing_name() {
        let config = MyDigitalIdConfig::mock();
        let client = MyDigitalIdClient::new(config);

        let user_info = UserInfoResponse {
            sub: "user123".to_string(),
            name: None,
            ic_number: Some("900101145678".to_string()),
            verified: Some(true),
        };

        let result = client.process_claims(&user_info);
        assert!(matches!(
            result,
            Err(MyDigitalIdError::MissingClaim("name"))
        ));
    }

    #[tokio::test]
    async fn test_validate_id_token_hs256() {
        let config = MyDigitalIdConfig::mock();
        let client = MyDigitalIdClient::new(config.clone());
        let id_token = make_test_id_token(&config, "nonce-123", "access-token");

        let claims = client
            .validate_id_token(&id_token, "access-token", "nonce-123")
            .await
            .expect("Should validate ID token");

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.nonce.as_deref(), Some("nonce-123"));
    }

    #[tokio::test]
    async fn test_validate_id_token_rejects_nonce_mismatch() {
        let config = MyDigitalIdConfig::mock();
        let client = MyDigitalIdClient::new(config.clone());
        let id_token = make_test_id_token(&config, "nonce-123", "access-token");

        let result = client
            .validate_id_token(&id_token, "access-token", "wrong-nonce")
            .await;

        assert!(matches!(result, Err(MyDigitalIdError::NonceMismatch)));
    }

    #[test]
    fn test_ensure_matching_subject_rejects_mismatch() {
        let id_token_claims = IdTokenClaims {
            iss: "https://mock.mydigital.gov.my".to_string(),
            sub: "user123".to_string(),
            aud: AudienceClaim::One("test_client_id".to_string()),
            exp: Utc::now().timestamp() + 300,
            iat: None,
            nonce: Some("nonce-123".to_string()),
            azp: None,
            at_hash: None,
        };
        let user_info = UserInfoResponse {
            sub: "different-user".to_string(),
            name: Some("Ahmad Bin Ali".to_string()),
            ic_number: Some("900101-14-5678".to_string()),
            verified: Some(true),
        };

        let result = MyDigitalIdClient::ensure_matching_subject(&id_token_claims, &user_info);
        assert!(matches!(result, Err(MyDigitalIdError::SubjectMismatch)));
    }
}
