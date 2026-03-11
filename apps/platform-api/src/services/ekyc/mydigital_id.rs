//! MyDigital ID OAuth 2.0 client for eKYC.
//!
//! Implements OAuth 2.0 with PKCE for Malaysian government identity verification.
//! Reference: MyDigital ID API documentation (Malaysian government identity platform).

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use sha2::{Digest, Sha256};

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
            authorization_url: std::env::var("MYDIGITAL_AUTH_URL").unwrap_or_else(|_| {
                "https://api.mydigital.gov.my/oauth2/authorize".to_string()
            }),
            token_url: std::env::var("MYDIGITAL_TOKEN_URL")
                .unwrap_or_else(|_| "https://api.mydigital.gov.my/oauth2/token".to_string()),
            userinfo_url: std::env::var("MYDIGITAL_USERINFO_URL")
                .unwrap_or_else(|_| "https://api.mydigital.gov.my/oauth2/userinfo".to_string()),
            redirect_uri: std::env::var("MYDIGITAL_REDIRECT_URI")
                .map_err(|_| MyDigitalIdError::MissingConfig("MYDIGITAL_REDIRECT_URI"))?,
            scope: std::env::var("MYDIGITAL_SCOPE")
                .unwrap_or_else(|_| "openid profile ic_number".to_string()),
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
    /// Returns the URL and PKCE parameters that must be stored for the callback.
    pub fn build_authorization_url(&self) -> Result<(String, PkceParams), MyDigitalIdError> {
        let pkce = PkceParams::generate().map_err(|_| MyDigitalIdError::PkceGenerationFailed)?;

        let url = format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method={}",
            self.config.authorization_url,
            urlencoding::encode(&self.config.client_id),
            urlencoding::encode(&self.config.redirect_uri),
            urlencoding::encode(&self.config.scope),
            urlencoding::encode(&pkce.state),
            urlencoding::encode(&pkce.code_challenge),
            pkce.code_challenge_method,
        );

        Ok((url, pkce))
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
    pub fn process_claims(&self, user_info: &UserInfoResponse) -> Result<VerifiedClaims, MyDigitalIdError> {
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
}

/// Hash a claim value using SHA-256.
fn hash_claim(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let hash = hasher.finalize();
    URL_SAFE_NO_PAD.encode(hash)
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
    #[error("PKCE generation failed")]
    PkceGenerationFailed,
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Token exchange failed: {0}")]
    TokenExchangeFailed(String),
    #[error("User info fetch failed: {0}")]
    UserInfoFailed(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("Missing claim: {0}")]
    MissingClaim(&'static str),
    #[error("State mismatch")]
    StateMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let (url, pkce) = client
            .build_authorization_url()
            .expect("Should build URL");

        assert!(url.contains("response_type=code"));
        assert!(url.contains("client_id=test_client_id"));
        assert!(url.contains(&format!("state={}", pkce.state)));
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
        assert!(matches!(result, Err(MyDigitalIdError::MissingClaim("name"))));
    }
}
