//! VP-9: eKYC API routes.
//!
//! Endpoints for MyDigital ID integration and identity verification.

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use crypto_engine::did::Did;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::middleware::tenant::TenantId;
use crate::services::ekyc::EkycService;
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
    /// Tenant ID is derived from authenticated context.
    pub tenant_id: Option<String>,
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
    State(state): State<AppState>,
    Extension(TenantId(tenant_id)): Extension<TenantId>,
    Json(request): Json<InitiateRequest>,
) -> impl IntoResponse {
    if request
        .tenant_id
        .as_deref()
        .is_some_and(|supplied| supplied != tenant_id)
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "SAHI_2301".to_string(),
                message: "Tenant ID must not override the authenticated tenant context".to_string(),
            }),
        )
            .into_response();
    }

    let (service, store) = match configured_ekyc_runtime(&state) {
        Ok(runtime) => runtime,
        Err(response) => return *response,
    };

    let response = match service.initiate_verification(&tenant_id, request.user_id.as_deref()) {
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

    if let Err(e) = store
        .create_flow(&response.verification, &response.session)
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "SAHI_5002".to_string(),
                message: format!("Failed to persist verification flow: {e}"),
            }),
        )
            .into_response();
    }

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
#[allow(clippy::too_many_lines)]
async fn handle_callback(
    State(state): State<AppState>,
    Extension(TenantId(tenant_id)): Extension<TenantId>,
    Json(request): Json<CallbackRequest>,
) -> impl IntoResponse {
    let (service, store) = match configured_ekyc_runtime(&state) {
        Ok(runtime) => runtime,
        Err(response) => return *response,
    };

    let Some(session) = (match store
        .find_session_by_state(&tenant_id, &request.state)
        .await
    {
        Ok(session) => session,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "SAHI_5003".to_string(),
                    message: format!("Failed to read OAuth session: {e}"),
                }),
            )
                .into_response();
        }
    }) else {
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                code: "SAHI_2304".to_string(),
                message: "OAuth session not found or already consumed".to_string(),
            }),
        )
            .into_response();
    };

    if session.verification_id != request.verification_id {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "SAHI_2305".to_string(),
                message: "Callback verification ID does not match the stored OAuth session"
                    .to_string(),
            }),
        )
            .into_response();
    }

    let callback = match service
        .handle_callback(&request.code, &request.state, &session)
        .await
    {
        Ok(result) => result,
        Err(e) => {
            let _ = store
                .mark_failed(&tenant_id, &session.verification_id, &e.to_string())
                .await;
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    code: "SAHI_2306".to_string(),
                    message: format!("Verification callback failed: {e}"),
                }),
            )
                .into_response();
        }
    };

    let verification = match store
        .mark_verified(
            &tenant_id,
            &callback.verification_id,
            &callback.claims,
            callback.verified_at,
            callback.expires_at,
        )
        .await
    {
        Ok(verification) => verification,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "SAHI_5004".to_string(),
                    message: format!("Failed to persist verified identity: {e}"),
                }),
            )
                .into_response();
        }
    };

    if let Err(e) = store.delete_session(&tenant_id, &session.id).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                code: "SAHI_5005".to_string(),
                message: format!("Verification succeeded but session cleanup failed: {e}"),
            }),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(CallbackResponse {
            verification_id: verification.id,
            status: verification.status.to_string(),
            assurance_level: verification.assurance_level.to_string(),
            verified_at: verification
                .verified_at
                .expect("verified_at set by mark_verified")
                .to_rfc3339(),
            expires_at: verification
                .expires_at
                .expect("expires_at set by mark_verified")
                .to_rfc3339(),
        }),
    )
        .into_response()
}

/// GET /api/v1/ekyc/status/:verification_id
///
/// Get verification status.
async fn get_status(
    State(state): State<AppState>,
    Extension(TenantId(tenant_id)): Extension<TenantId>,
    Path(verification_id): Path<String>,
) -> impl IntoResponse {
    let (_, store) = match configured_ekyc_runtime(&state) {
        Ok(runtime) => runtime,
        Err(response) => return *response,
    };

    let verification = match store.get_verification(&tenant_id, &verification_id).await {
        Ok(Some(verification)) => verification,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: "SAHI_2307".to_string(),
                    message: "Verification record not found".to_string(),
                }),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "SAHI_5006".to_string(),
                    message: format!("Failed to load verification status: {e}"),
                }),
            )
                .into_response()
        }
    };

    (
        StatusCode::OK,
        Json(StatusResponse {
            verification_id: verification.id,
            status: verification.status.to_string(),
            provider: verification.provider.to_string(),
            assurance_level: verification.assurance_level.to_string(),
            did: verification.did,
            verified_at: verification.verified_at.map(|value| value.to_rfc3339()),
            expires_at: verification.expires_at.map(|value| value.to_rfc3339()),
            created_at: verification.created_at.to_rfc3339(),
        }),
    )
        .into_response()
}

/// POST /api/v1/ekyc/bind-did
///
/// Bind a DID to a verified identity.
async fn bind_did(
    State(state): State<AppState>,
    Extension(TenantId(tenant_id)): Extension<TenantId>,
    Json(request): Json<BindDidRequest>,
) -> impl IntoResponse {
    // Validate DID format using crypto-engine parser
    if Did::parse(&request.did).is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "SAHI_2308".to_string(),
                message: "Invalid DID format. Expected format: did:method:identifier".to_string(),
            }),
        )
            .into_response();
    }

    let (service, store) = match configured_ekyc_runtime(&state) {
        Ok(runtime) => runtime,
        Err(response) => return *response,
    };

    let mut verification = match store
        .get_verification(&tenant_id, &request.verification_id)
        .await
    {
        Ok(Some(verification)) => verification,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    code: "SAHI_2307".to_string(),
                    message: "Verification record not found".to_string(),
                }),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "SAHI_5007".to_string(),
                    message: format!("Failed to load verification record: {e}"),
                }),
            )
                .into_response()
        }
    };

    if let Err(e) = service.bind_did(&mut verification, &request.did) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "SAHI_2309".to_string(),
                message: format!("Failed to bind DID: {e}"),
            }),
        )
            .into_response();
    }

    let verification = match store
        .bind_did(
            &tenant_id,
            &request.verification_id,
            &request.did,
            verification
                .did_bound_at
                .expect("did_bound_at set by bind_did"),
        )
        .await
    {
        Ok(verification) => verification,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    code: "SAHI_5008".to_string(),
                    message: format!("Failed to persist DID binding: {e}"),
                }),
            )
                .into_response()
        }
    };

    (
        StatusCode::OK,
        Json(BindDidResponse {
            verification_id: verification.id,
            did: verification.did.expect("did set by bind_did"),
            bound_at: verification
                .did_bound_at
                .expect("did_bound_at set by bind_did")
                .to_rfc3339(),
        }),
    )
        .into_response()
}

// ─── HELPERS ──────────────────────────────────────────────────────────────

fn configured_ekyc_runtime(
    state: &AppState,
) -> Result<(Arc<EkycService>, crate::services::ekyc::SharedEkycStore), Box<axum::response::Response>>
{
    let Some(service) = state.ekyc_service.clone() else {
        return Err(Box::new(
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    code: "SAHI_5001".to_string(),
                    message: "eKYC service is not configured".to_string(),
                }),
            )
                .into_response(),
        ));
    };
    let Some(store) = state.ekyc_store.clone() else {
        return Err(Box::new(
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    code: "SAHI_5009".to_string(),
                    message: "eKYC persistence store is not configured".to_string(),
                }),
            )
                .into_response(),
        ));
    };
    Ok((service, store))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn create_test_app() -> Router {
        create_test_app_with_store().0
    }

    fn create_test_app_with_store() -> (Router, Arc<crate::services::ekyc::InMemoryEkycStore>) {
        use crate::services::ekyc::InMemoryEkycStore;
        use crypto_engine::kms::SoftwareKmsProvider;

        let store = Arc::new(InMemoryEkycStore::new());
        let state = AppState {
            kms: Arc::new(SoftwareKmsProvider::new()) as Arc<dyn crypto_engine::kms::KmsProvider>,
            security: crate::state::SecurityConfig::default(),
            tenant_token_validator: None,
            ekyc_service: Some(Arc::new(EkycService::new(
                crate::services::ekyc::MyDigitalIdConfig::mock(),
            ))),
            ekyc_store: Some(store.clone()),
        };

        (
            router()
                .layer(axum::Extension(TenantId("TNT_test".to_string())))
                .with_state(state),
            store,
        )
    }

    #[tokio::test]
    async fn test_initiate_verification() {
        let app = create_test_app();

        let request = Request::builder()
            .method("POST")
            .uri("/initiate")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"user_id": "USR_test"}"#))
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
    async fn test_get_status_not_found() {
        let app = create_test_app();

        let request = Request::builder()
            .method("GET")
            .uri("/status/VRF_01HXYZ")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: ErrorResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(json.code, "SAHI_2307");
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

        assert_eq!(json.code, "SAHI_2308");
    }

    #[tokio::test]
    async fn test_bind_did_requires_verified_record() {
        let app = create_test_app();

        let initiate_request = Request::builder()
            .method("POST")
            .uri("/initiate")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"user_id": "USR_test"}"#))
            .unwrap();
        let initiate_response = app.clone().oneshot(initiate_request).await.unwrap();
        let body = axum::body::to_bytes(initiate_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: InitiateVerificationResponse = serde_json::from_slice(&body).unwrap();

        let request = Request::builder()
            .method("POST")
            .uri("/bind-did")
            .header("Content-Type", "application/json")
            .body(Body::from(format!(
                r#"{{"verification_id": "{}", "did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"}}"#,
                json.verification_id
            )))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: ErrorResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(json.code, "SAHI_2309");
    }

    #[tokio::test]
    async fn test_callback_requires_existing_session() {
        let app = create_test_app();

        let request = Request::builder()
            .method("POST")
            .uri("/callback")
            .header("Content-Type", "application/json")
            .body(Body::from(
                r#"{"code":"auth-code","state":"missing","verification_id":"IDV_missing"}"#,
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: ErrorResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(json.code, "SAHI_2304");
    }

    #[tokio::test]
    async fn test_get_status_after_initiate() {
        let app = create_test_app();

        let initiate_request = Request::builder()
            .method("POST")
            .uri("/initiate")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"user_id": "USR_test"}"#))
            .unwrap();
        let initiate_response = app.clone().oneshot(initiate_request).await.unwrap();
        let body = axum::body::to_bytes(initiate_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let initiated: InitiateVerificationResponse = serde_json::from_slice(&body).unwrap();

        let request = Request::builder()
            .method("GET")
            .uri(format!("/status/{}", initiated.verification_id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: StatusResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(json.status, "in_progress");
        assert_eq!(json.assurance_level, "P1");
        assert_eq!(json.provider, "mydigital_id");
    }

    #[tokio::test]
    async fn test_bind_did_success_for_verified_record() {
        use crate::services::ekyc::EkycStore;
        use crate::services::ekyc::{
            AssuranceLevel, IdentityVerification, OAuthSession, VerificationProvider,
            VerificationStatus,
        };
        use chrono::{Duration, Utc};

        let (app, store) = create_test_app_with_store();
        let now = Utc::now();
        let verification = IdentityVerification {
            id: "IDV_test_verified".to_string(),
            tenant_id: "TNT_test".to_string(),
            user_id: Some("USR_test".to_string()),
            status: VerificationStatus::Verified,
            provider: VerificationProvider::MydigitalId,
            assurance_level: AssuranceLevel::P2,
            name_hash: Some("name_hash".to_string()),
            ic_hash: Some("ic_hash".to_string()),
            did: None,
            did_bound_at: None,
            verified_at: Some(now),
            expires_at: Some(now + Duration::days(365)),
            failure_reason: None,
            created_at: now,
            updated_at: now,
        };
        let session = OAuthSession {
            id: "OAS_test_verified".to_string(),
            tenant_id: "TNT_test".to_string(),
            verification_id: verification.id.clone(),
            state: "state".to_string(),
            nonce: "nonce".to_string(),
            code_verifier: "verifier".to_string(),
            code_challenge: "challenge".to_string(),
            redirect_uri: "vaultpass://callback".to_string(),
            scope: "openid profile ic_number".to_string(),
            expires_at: now + Duration::minutes(10),
            created_at: now,
        };
        store.create_flow(&verification, &session).await.unwrap();

        let request = Request::builder()
            .method("POST")
            .uri("/bind-did")
            .header("Content-Type", "application/json")
            .body(Body::from(
                r#"{"verification_id":"IDV_test_verified","did":"did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"}"#,
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: BindDidResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(json.verification_id, "IDV_test_verified");
        assert_eq!(
            json.did,
            "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
        );
    }
}
