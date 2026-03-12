//! VP-9: Onboarding & KYC Service
//!
//! Provides eKYC integration via MyDigital ID (Malaysian government identity platform).
//!
//! ## Flow
//!
//! 1. Client initiates verification via `POST /api/v1/ekyc/initiate`
//! 2. Service creates OAuth session with PKCE and returns authorization URL
//! 3. User completes verification in MyDigital ID (in-app browser or redirect)
//! 4. Callback received at `POST /api/v1/ekyc/callback` with authorization code
//! 5. Service exchanges code for tokens, fetches user info
//! 6. Claims are hashed (never stored raw) and verification status updated
//! 7. Client binds verified identity to wallet DID via `POST /api/v1/ekyc/bind-did`
//!
//! ## Security
//!
//! - PKCE required for all OAuth flows (RFC 7636)
//! - No raw PII stored - only SHA-256 hashes of name and IC
//! - OAuth sessions expire after 10 minutes
//! - Verification records expire based on assurance level

pub mod mydigital_id;
pub mod pkce;
pub mod store;
pub mod types;

use chrono::{Duration, Utc};
use sahi_core::ulid::{TypedUlid, UlidPrefix};

pub use mydigital_id::{MyDigitalIdClient, MyDigitalIdConfig, MyDigitalIdError};
#[allow(unused_imports)]
pub use pkce::{constant_time_eq, PkceParams};
pub use store::{EkycStore, InMemoryEkycStore, PostgresEkycStore, SharedEkycStore};
pub use types::{
    AssuranceLevel, IdentityVerification, OAuthSession, VerificationProvider, VerificationStatus,
    VerifiedClaims,
};

/// eKYC service for managing identity verifications.
pub struct EkycService {
    client: MyDigitalIdClient,
}

impl EkycService {
    /// Create a new eKYC service.
    pub fn new(config: MyDigitalIdConfig) -> Self {
        Self {
            client: MyDigitalIdClient::new(config),
        }
    }

    /// Create a new eKYC service from environment configuration.
    pub fn from_env() -> Result<Self, EkycError> {
        let config = MyDigitalIdConfig::from_env().map_err(EkycError::Config)?;
        Ok(Self::new(config))
    }

    /// Initiate a new verification flow.
    ///
    /// Returns the authorization URL and session details.
    pub fn initiate_verification(
        &self,
        tenant_id: &str,
        user_id: Option<&str>,
    ) -> Result<InitiateResponse, EkycError> {
        // Generate verification ID
        let verification_id = TypedUlid::new(UlidPrefix::IdentityVerification);

        // Generate OAuth session
        let session_id = TypedUlid::new(UlidPrefix::OAuthSession);
        let (authorization_url, pkce, nonce) = self
            .client
            .build_authorization_url()
            .map_err(EkycError::MyDigitalId)?;

        let now = Utc::now();

        // Create verification record
        let verification = IdentityVerification {
            id: verification_id.to_string(),
            tenant_id: tenant_id.to_string(),
            user_id: user_id.map(ToString::to_string),
            status: VerificationStatus::InProgress,
            provider: VerificationProvider::MydigitalId,
            assurance_level: AssuranceLevel::P1, // Will be updated after verification
            name_hash: None,
            ic_hash: None,
            did: None,
            did_bound_at: None,
            verified_at: None,
            expires_at: None,
            failure_reason: None,
            created_at: now,
            updated_at: now,
        };

        // Create OAuth session
        let session = OAuthSession {
            id: session_id.to_string(),
            tenant_id: tenant_id.to_string(),
            verification_id: verification_id.to_string(),
            state: pkce.state.clone(),
            nonce,
            code_verifier: pkce.code_verifier.clone(),
            code_challenge: pkce.code_challenge.clone(),
            redirect_uri: self.client.redirect_uri().to_string(),
            scope: self.client.scope().to_string(),
            expires_at: now + Duration::minutes(10),
            created_at: now,
        };

        Ok(InitiateResponse {
            verification,
            session,
            authorization_url,
        })
    }

    /// Handle OAuth callback.
    ///
    /// Exchanges authorization code for tokens and processes claims.
    pub async fn handle_callback(
        &self,
        code: &str,
        state: &str,
        session: &OAuthSession,
    ) -> Result<CallbackResponse, EkycError> {
        // Verify state matches (constant-time comparison prevents timing attacks)
        if !constant_time_eq(session.state.as_bytes(), state.as_bytes()) {
            return Err(EkycError::StateMismatch);
        }

        // Check session expiration
        if Utc::now() > session.expires_at {
            return Err(EkycError::SessionExpired);
        }

        // Exchange code for tokens
        let tokens = self
            .client
            .exchange_code(code, &session.code_verifier)
            .await
            .map_err(EkycError::MyDigitalId)?;

        if !tokens.token_type.eq_ignore_ascii_case("Bearer") {
            return Err(EkycError::MyDigitalId(
                MyDigitalIdError::UnsupportedTokenType(tokens.token_type),
            ));
        }

        let id_token = tokens
            .id_token
            .as_deref()
            .ok_or(EkycError::MyDigitalId(MyDigitalIdError::MissingIdToken))?;
        let id_token_claims = self
            .client
            .validate_id_token(id_token, &tokens.access_token, &session.nonce)
            .await
            .map_err(EkycError::MyDigitalId)?;

        // Fetch user info
        let user_info = self
            .client
            .fetch_user_info(&tokens.access_token)
            .await
            .map_err(EkycError::MyDigitalId)?;
        MyDigitalIdClient::ensure_matching_subject(&id_token_claims, &user_info)
            .map_err(EkycError::MyDigitalId)?;

        // Process claims (hash PII)
        let claims = self
            .client
            .process_claims(&user_info)
            .map_err(EkycError::MyDigitalId)?;

        // Calculate verification expiration based on assurance level
        let expires_at = match claims.assurance_level {
            AssuranceLevel::P1 => Utc::now() + Duration::days(30),
            AssuranceLevel::P2 => Utc::now() + Duration::days(365),
            AssuranceLevel::P3 => Utc::now() + Duration::days(730), // 2 years
        };

        Ok(CallbackResponse {
            verification_id: session.verification_id.clone(),
            claims,
            verified_at: Utc::now(),
            expires_at,
        })
    }

    /// Bind a DID to a verified identity.
    #[allow(clippy::unused_self)]
    pub fn bind_did(
        &self,
        verification: &mut IdentityVerification,
        did: &str,
    ) -> Result<(), EkycError> {
        // Verify status is verified
        if verification.status != VerificationStatus::Verified {
            return Err(EkycError::NotVerified);
        }

        // Verify DID format
        if !did.starts_with("did:") {
            return Err(EkycError::InvalidDid);
        }

        // Bind DID
        verification.did = Some(did.to_string());
        verification.did_bound_at = Some(Utc::now());
        verification.updated_at = Utc::now();

        Ok(())
    }
}

/// Response from initiating verification.
#[derive(Debug)]
pub struct InitiateResponse {
    pub verification: IdentityVerification,
    pub session: OAuthSession,
    pub authorization_url: String,
}

/// Response from handling callback.
#[derive(Debug)]
pub struct CallbackResponse {
    pub verification_id: String,
    pub claims: VerifiedClaims,
    pub verified_at: chrono::DateTime<Utc>,
    pub expires_at: chrono::DateTime<Utc>,
}

/// eKYC service errors.
#[derive(Debug, thiserror::Error)]
pub enum EkycError {
    #[error("Configuration error: {0}")]
    Config(#[from] MyDigitalIdError),
    #[error("MyDigital ID error: {0}")]
    MyDigitalId(MyDigitalIdError),
    #[error("State mismatch - possible CSRF attack")]
    StateMismatch,
    #[error("OAuth session expired")]
    SessionExpired,
    #[error("Identity not verified")]
    NotVerified,
    #[error("Invalid DID format")]
    InvalidDid,
    #[error("Verification not found")]
    NotFound,
    #[error("Database error: {0}")]
    Database(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_service() -> EkycService {
        let config = MyDigitalIdConfig::mock();
        EkycService::new(config)
    }

    #[test]
    fn test_initiate_verification() {
        let service = create_test_service();

        let response = service
            .initiate_verification("TNT_test", Some("USR_test"))
            .expect("Should initiate verification");

        // Verification should be created
        assert!(response.verification.id.starts_with("IDV_"));
        assert_eq!(response.verification.tenant_id, "TNT_test");
        assert_eq!(response.verification.user_id, Some("USR_test".to_string()));
        assert_eq!(response.verification.status, VerificationStatus::InProgress);

        // Session should be created
        assert!(response.session.id.starts_with("OAS_"));
        assert!(!response.session.state.is_empty());
        assert!(!response.session.nonce.is_empty());
        assert!(!response.session.code_verifier.is_empty());
        assert_eq!(response.session.redirect_uri, "vaultpass://callback");

        // Authorization URL should be valid
        assert!(response.authorization_url.contains("response_type=code"));
        assert!(response.authorization_url.contains("nonce="));
    }

    #[test]
    fn test_bind_did() {
        let service = create_test_service();

        let mut verification = IdentityVerification {
            id: "VRF_test".to_string(),
            tenant_id: "TNT_test".to_string(),
            user_id: None,
            status: VerificationStatus::Verified,
            provider: VerificationProvider::MydigitalId,
            assurance_level: AssuranceLevel::P2,
            name_hash: Some("hash".to_string()),
            ic_hash: Some("hash".to_string()),
            did: None,
            did_bound_at: None,
            verified_at: Some(Utc::now()),
            expires_at: Some(Utc::now() + Duration::days(365)),
            failure_reason: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        service
            .bind_did(
                &mut verification,
                "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
            )
            .expect("Should bind DID");

        assert_eq!(
            verification.did,
            Some("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string())
        );
        assert!(verification.did_bound_at.is_some());
    }

    #[test]
    fn test_bind_did_not_verified() {
        let service = create_test_service();

        let mut verification = IdentityVerification {
            id: "VRF_test".to_string(),
            tenant_id: "TNT_test".to_string(),
            user_id: None,
            status: VerificationStatus::Pending,
            provider: VerificationProvider::MydigitalId,
            assurance_level: AssuranceLevel::P1,
            name_hash: None,
            ic_hash: None,
            did: None,
            did_bound_at: None,
            verified_at: None,
            expires_at: None,
            failure_reason: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = service.bind_did(&mut verification, "did:key:z6Mk...");
        assert!(matches!(result, Err(EkycError::NotVerified)));
    }

    #[test]
    fn test_bind_did_invalid_format() {
        let service = create_test_service();

        let mut verification = IdentityVerification {
            id: "VRF_test".to_string(),
            tenant_id: "TNT_test".to_string(),
            user_id: None,
            status: VerificationStatus::Verified,
            provider: VerificationProvider::MydigitalId,
            assurance_level: AssuranceLevel::P2,
            name_hash: Some("hash".to_string()),
            ic_hash: Some("hash".to_string()),
            did: None,
            did_bound_at: None,
            verified_at: Some(Utc::now()),
            expires_at: Some(Utc::now() + Duration::days(365)),
            failure_reason: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = service.bind_did(&mut verification, "invalid_did");
        assert!(matches!(result, Err(EkycError::InvalidDid)));
    }
}
