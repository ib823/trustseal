//! VP-9: eKYC API routes.
//!
//! Endpoints for MyDigital ID integration and identity verification.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::services::ekyc::{
    AssuranceLevel, EkycError, EkycService, VerificationProvider, VerificationStatus,
};
use crate::state::AppState;

/// Build the eKYC router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/initiate", post(initiate_verification))
        .route("/callback", post(handle_callback))
        .route("/status/{verification_id}", get(get_status))
        .route("/bind-did", post(bind_did))
}

// ─── REQUEST/RESPONSE TYPES ───────────────────────────────────────────────

/// Request to initiate verification.
#[derive(Debug, Deserialize)]
pub struct InitiateRequest {
    /// Tenant ID (from JWT or header).
    pub tenant_id: String,
    /// Optional user ID if already registered.
    pub user_id: Option<String>,
    /// Redirect URI for mobile app callback.
    pub redirect_uri: Option<String>,
}

/// Response from initiating verification.
#[derive(Debug, Serialize, Deserialize)]
pub struct InitiateVerificationResponse {
    /// Verification ID to track this flow.
    pub verification_id: String,
    /// Authorization URL to redirect user to.
    pub authorization_url: String,
    /// State parameter for CSRF protection.
    pub state: String,
    /// Session expiration time (ISO 8601).
    pub expires_at: String,
}

/// Request to handle OAuth callback.
#[derive(Debug, Deserialize)]
pub struct CallbackRequest {
    /// Authorization code from OAuth provider.
    pub code: String,
    /// State parameter for CSRF verification.
    pub state: String,
    /// Verification ID from initiate step.
    pub verification_id: String,
}

/// Response from callback handling.
#[derive(Debug, Serialize)]
pub struct CallbackResponse {
    /// Verification ID.
    pub verification_id: String,
    /// Verification status.
    pub status: String,
    /// Assurance level achieved.
    pub assurance_level: String,
    /// When verification was completed (ISO 8601).
    pub verified_at: String,
    /// When verification expires (ISO 8601).
    pub expires_at: String,
}

/// Response for verification status.
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    /// Verification ID.
    pub verification_id: String,
    /// Current status.
    pub status: String,
    /// Verification provider.
    pub provider: String,
    /// Assurance level.
    pub assurance_level: String,
    /// Bound DID (if any).
    pub did: Option<String>,
    /// When verified (ISO 8601, if verified).
    pub verified_at: Option<String>,
    /// When expires (ISO 8601, if verified).
    pub expires_at: Option<String>,
    /// Created timestamp (ISO 8601).
    pub created_at: String,
}

/// Request to bind DID.
#[derive(Debug, Deserialize)]
pub struct BindDidRequest {
    /// Verification ID.
    pub verification_id: String,
    /// DID to bind (e.g., did:key:z6Mk...).
    pub did: String,
}

/// Response from binding DID.
#[derive(Debug, Serialize, Deserialize)]
pub struct BindDidResponse {
    /// Verification ID.
    pub verification_id: String,
    /// Bound DID.
    pub did: String,
    /// When bound (ISO 8601).
    pub bound_at: String,
}

/// Error response.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code.
    pub code: String,
    /// Error message.
    pub message: String,
}

// ─── HANDLERS ─────────────────────────────────────────────────────────────

/// POST /api/v1/ekyc/initiate
///
/// Initiate a new eKYC verification flow.
async fn initiate_verification(
    State(_state): State<AppState>,
    Json(request): Json<InitiateRequest>,
) -> impl IntoResponse {
    // In production, we'd get EkycService from state with proper config
    // For now, use mock configuration for development
    let service = match create_ekyc_service() {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "SAHI_5001".to_string(),
                    message: format!("eKYC service configuration error: {e}"),
                }),
            )
                .into_response();
        }
    };

    let response = match service.initiate_verification(&request.tenant_id, request.user_id.as_deref()) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: "SAHI_2300".to_string(),
                    message: format!("Failed to initiate verification: {e}"),
                }),
            )
                .into_response();
        }
    };

    // In production, persist verification and session to database here

    (
        StatusCode::OK,
        Json(InitiateVerificationResponse {
            verification_id: response.verification.id,
            authorization_url: response.authorization_url,
            state: response.session.state,
            expires_at: response.session.expires_at.to_rfc3339(),
        }),
    )
        .into_response()
}

/// POST /api/v1/ekyc/callback
///
/// Handle OAuth callback from MyDigital ID.
async fn handle_callback(
    State(_state): State<AppState>,
    Json(request): Json<CallbackRequest>,
) -> impl IntoResponse {
    // In production:
    // 1. Look up OAuth session by state
    // 2. Verify session belongs to verification_id
    // 3. Call service.handle_callback()
    // 4. Update verification record in database
    // 5. Delete OAuth session

    // For now, return a mock success response
    // The actual implementation requires database integration

    (
        StatusCode::OK,
        Json(CallbackResponse {
            verification_id: request.verification_id,
            status: VerificationStatus::Verified.to_string(),
            assurance_level: AssuranceLevel::P2.to_string(),
            verified_at: chrono::Utc::now().to_rfc3339(),
            expires_at: (chrono::Utc::now() + chrono::Duration::days(365)).to_rfc3339(),
        }),
    )
        .into_response()
}

/// GET /api/v1/ekyc/status/:verification_id
///
/// Get verification status.
async fn get_status(
    State(_state): State<AppState>,
    Path(verification_id): Path<String>,
) -> impl IntoResponse {
    // In production, look up verification from database

    // For now, return a mock response
    (
        StatusCode::OK,
        Json(StatusResponse {
            verification_id,
            status: VerificationStatus::Pending.to_string(),
            provider: VerificationProvider::MydigitalId.to_string(),
            assurance_level: AssuranceLevel::P1.to_string(),
            did: None,
            verified_at: None,
            expires_at: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        }),
    )
        .into_response()
}

/// POST /api/v1/ekyc/bind-did
///
/// Bind a DID to a verified identity.
async fn bind_did(
    State(_state): State<AppState>,
    Json(request): Json<BindDidRequest>,
) -> impl IntoResponse {
    // Validate DID format
    if !request.did.starts_with("did:") {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "SAHI_4001".to_string(),
                message: "Invalid DID format. Must start with 'did:'".to_string(),
            }),
        )
            .into_response();
    }

    // In production:
    // 1. Look up verification from database
    // 2. Verify status is 'verified'
    // 3. Call service.bind_did()
    // 4. Update verification record

    // For now, return a mock success response
    (
        StatusCode::OK,
        Json(BindDidResponse {
            verification_id: request.verification_id,
            did: request.did,
            bound_at: chrono::Utc::now().to_rfc3339(),
        }),
    )
        .into_response()
}

// ─── HELPERS ──────────────────────────────────────────────────────────────

/// Create eKYC service (uses mock config if env vars not set).
#[allow(clippy::unnecessary_wraps)]
fn create_ekyc_service() -> Result<EkycService, EkycError> {
    use crate::services::ekyc::MyDigitalIdConfig;

    // Try to create from environment, fall back to mock for development
    let config = MyDigitalIdConfig::from_env().unwrap_or_else(|_| {
        tracing::warn!("MyDigital ID not configured, using mock service");
        MyDigitalIdConfig {
            client_id: "mock_client_id".to_string(),
            client_secret: None,
            authorization_url: "https://mock.mydigital.gov.my/oauth2/authorize".to_string(),
            token_url: "https://mock.mydigital.gov.my/oauth2/token".to_string(),
            userinfo_url: "https://mock.mydigital.gov.my/oauth2/userinfo".to_string(),
            redirect_uri: "vaultpass://ekyc/callback".to_string(),
            scope: "openid profile ic_number".to_string(),
        }
    });

    Ok(EkycService::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn create_test_app() -> Router {
        use std::sync::Arc;
        use crypto_engine::kms::SoftwareKmsProvider;

        let state = AppState {
            kms: Arc::new(SoftwareKmsProvider::new()) as Arc<dyn crypto_engine::kms::KmsProvider>,
        };

        router().with_state(state)
    }

    #[tokio::test]
    async fn test_initiate_verification() {
        let app = create_test_app();

        let request = Request::builder()
            .method("POST")
            .uri("/initiate")
            .header("Content-Type", "application/json")
            .body(Body::from(
                r#"{"tenant_id": "TNT_test", "user_id": "USR_test"}"#,
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: InitiateVerificationResponse = serde_json::from_slice(&body).unwrap();

        assert!(json.verification_id.starts_with("IDV_"));
        assert!(!json.authorization_url.is_empty());
        assert!(!json.state.is_empty());
    }

    #[tokio::test]
    async fn test_get_status() {
        let app = create_test_app();

        let request = Request::builder()
            .method("GET")
            .uri("/status/VRF_01HXYZ")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: StatusResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(json.verification_id, "VRF_01HXYZ");
    }

    #[tokio::test]
    async fn test_bind_did_invalid_format() {
        let app = create_test_app();

        let request = Request::builder()
            .method("POST")
            .uri("/bind-did")
            .header("Content-Type", "application/json")
            .body(Body::from(
                r#"{"verification_id": "VRF_test", "did": "invalid"}"#,
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: ErrorResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(json.code, "SAHI_4001");
    }

    #[tokio::test]
    async fn test_bind_did_success() {
        let app = create_test_app();

        let request = Request::builder()
            .method("POST")
            .uri("/bind-did")
            .header("Content-Type", "application/json")
            .body(Body::from(
                r#"{"verification_id": "VRF_test", "did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"}"#,
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: BindDidResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(json.verification_id, "VRF_test");
        assert!(json.did.starts_with("did:key:"));
    }
}
