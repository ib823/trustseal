//! TM-1: Signing service ceremony routes.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ring::rand::{SecureRandom, SystemRandom};
use sahi_core::{
    auth::{AuthContext, Role},
    ulid::{generate, UlidPrefix},
};
use serde::{Deserialize, Serialize};

use crate::{
    auth::TenantId,
    domain::{
        Ceremony, CeremonyConfig, CeremonyDocument, CeremonyId, CeremonyMetadata,
        CeremonyTransition, SignatureField, SignerRole, SignerSlot, SignerSlotId, SignerStatus,
    },
    orchestrator::OrchestratorError,
    state::AppState,
};

const CEREMONIES_READ_SCOPE: &str = "ceremonies:read";
const CEREMONIES_WRITE_SCOPE: &str = "ceremonies:write";
const CEREMONIES_SIGN_SCOPE: &str = "ceremonies:sign";
const MANAGEMENT_ROLES: &[Role] = &[
    Role::PlatformAdmin,
    Role::TenantAdmin,
    Role::ServiceInternal,
];
const SIGNER_ROLES: &[Role] = &[Role::ResidentUser, Role::GuestUser, Role::ServiceInternal];

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse {
                code: self.code.to_string(),
                message: self.message,
            }),
        )
            .into_response()
    }
}

/// Create the ceremony router.
pub fn ceremony_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_ceremony))
        .route("/:id", get(get_ceremony))
        .route("/:id/prepare", post(prepare_ceremony))
        .route("/:id/ready", post(ready_for_signatures))
        .route("/:id/abort", post(abort_ceremony))
        .route("/:id/resume", post(resume_ceremony))
        .route(
            "/:id/signers/:signer_id/authenticate",
            post(authenticate_signer),
        )
        .route("/:id/signers/:signer_id/sign", post(sign))
}

#[derive(Debug, Deserialize)]
pub struct CreateCeremonyRequest {
    pub title: String,
    pub description: Option<String>,
    pub reference: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub document: DocumentUpload,
    pub signers: Vec<SignerInput>,
    #[serde(default)]
    pub config: Option<CeremonyConfig>,
}

#[derive(Debug, Deserialize)]
pub struct DocumentUpload {
    pub filename: String,
    pub content_type: String,
    pub content_hash: String,
    pub size_bytes: u64,
    pub storage_key: String,
    #[serde(default)]
    pub signature_fields: Vec<SignatureField>,
}

#[derive(Debug, Deserialize)]
pub struct SignerInput {
    pub name: String,
    pub email: String,
    pub role: String,
    pub is_required: bool,
    pub order: u32,
}

#[derive(Debug, Serialize)]
pub struct CeremonyResponse {
    pub id: String,
    pub state: String,
    pub title: String,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub expires_at: String,
    pub signers: Vec<SignerResponse>,
}

#[derive(Debug, Serialize)]
pub struct SignerResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub signed_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct AbortRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct ResumeRequest {
    pub document_hash: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct AuthenticateRequest {
    pub webauthn_credential_id: String,
    pub assurance_level: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct SignRequest {
    pub cms_signature: String,
    pub timestamp_token: Option<String>,
    pub certificate_chain: Vec<String>,
    pub content_hash: String,
    pub algorithm: String,
}

#[derive(Debug, Serialize)]
pub struct TransitionResponse {
    pub from_state: String,
    pub to_state: String,
    pub timestamp: String,
}

async fn create_ceremony(
    State(state): State<AppState>,
    Extension(TenantId(tenant_id)): Extension<TenantId>,
    auth: Option<Extension<AuthContext>>,
    Json(request): Json<CreateCeremonyRequest>,
) -> Response {
    let auth = match require_auth_context(auth, CEREMONIES_WRITE_SCOPE, MANAGEMENT_ROLES) {
        Ok(auth) => auth,
        Err(error) => return error.into_response(),
    };
    let actor = match actor_id(&auth) {
        Ok(actor) => actor,
        Err(error) => return error.into_response(),
    };

    if request.signers.is_empty() {
        return error_response(
            StatusCode::BAD_REQUEST,
            "SAHI_3019",
            "A ceremony requires at least one signer",
        );
    }

    let signers = match build_signers(request.signers) {
        Ok(signers) => signers,
        Err(error) => return error.into_response(),
    };

    let document = CeremonyDocument {
        id: generate(UlidPrefix::CeremonyDocument),
        filename: request.document.filename,
        content_type: request.document.content_type,
        content_hash: request.document.content_hash,
        size_bytes: request.document.size_bytes,
        storage_key: request.document.storage_key,
        signature_fields: request.document.signature_fields,
    };
    let metadata = CeremonyMetadata {
        title: request.title,
        description: request.description,
        reference: request.reference,
        tags: request.tags,
    };

    let ceremony = match state
        .orchestrator
        .create_ceremony(
            &tenant_id,
            &actor,
            document,
            signers,
            request.config.unwrap_or_default(),
            metadata,
        )
        .await
    {
        Ok(ceremony) => ceremony,
        Err(err) => return orchestrator_error_response(&err),
    };

    if let Err(err) = state.ceremony_repository.create(&ceremony).await {
        return orchestrator_error_response(&err);
    }

    (StatusCode::CREATED, Json(map_ceremony(&ceremony))).into_response()
}

async fn get_ceremony(
    State(state): State<AppState>,
    Extension(TenantId(_tenant_id)): Extension<TenantId>,
    auth: Option<Extension<AuthContext>>,
    Path(id): Path<String>,
) -> Response {
    if let Err(response) = require_auth_context(auth, CEREMONIES_READ_SCOPE, MANAGEMENT_ROLES) {
        return response.into_response();
    }

    let ceremony = match state.ceremony_repository.get_by_id(&CeremonyId(id)).await {
        Ok(ceremony) => ceremony,
        Err(err) => return orchestrator_error_response(&err),
    };

    Json(map_ceremony(&ceremony)).into_response()
}

async fn prepare_ceremony(
    State(state): State<AppState>,
    auth: Option<Extension<AuthContext>>,
    Path(id): Path<String>,
) -> Response {
    let auth = match require_auth_context(auth, CEREMONIES_WRITE_SCOPE, MANAGEMENT_ROLES) {
        Ok(auth) => auth,
        Err(error) => return error.into_response(),
    };
    let actor = match actor_id(&auth) {
        Ok(actor) => actor,
        Err(error) => return error.into_response(),
    };

    let mut ceremony = match state.ceremony_repository.get_by_id(&CeremonyId(id)).await {
        Ok(ceremony) => ceremony,
        Err(err) => return orchestrator_error_response(&err),
    };
    let tenant_id = ceremony.tenant_id.clone();

    let transition = match state
        .orchestrator
        .prepare_ceremony(&mut ceremony, &actor)
        .await
    {
        Ok(transition) => transition,
        Err(err) => return orchestrator_error_response(&err),
    };

    if let Err(err) = persist_transition(&state, &ceremony, &transition).await {
        return orchestrator_error_response(&err);
    }

    for signer in ceremony
        .signers
        .iter()
        .filter(|signer| signer.status == SignerStatus::Pending)
    {
        let token = match generate_invitation_token() {
            Ok(token) => token,
            Err(error) => return error.into_response(),
        };
        if let Err(err) = state
            .signer_repository
            .send_invitation(&signer.id, &token, &ceremony.expires_at)
            .await
        {
            return orchestrator_error_response(&err);
        }
    }

    let _ = tenant_id;
    Json(map_transition(&transition)).into_response()
}

async fn ready_for_signatures(
    State(state): State<AppState>,
    auth: Option<Extension<AuthContext>>,
    Path(id): Path<String>,
) -> Response {
    let auth = match require_auth_context(auth, CEREMONIES_WRITE_SCOPE, MANAGEMENT_ROLES) {
        Ok(auth) => auth,
        Err(error) => return error.into_response(),
    };
    let actor = match actor_id(&auth) {
        Ok(actor) => actor,
        Err(error) => return error.into_response(),
    };

    let mut ceremony = match state.ceremony_repository.get_by_id(&CeremonyId(id)).await {
        Ok(ceremony) => ceremony,
        Err(err) => return orchestrator_error_response(&err),
    };

    let transition = match state
        .orchestrator
        .ready_for_signatures(&mut ceremony, &actor)
        .await
    {
        Ok(transition) => transition,
        Err(err) => return orchestrator_error_response(&err),
    };

    if let Err(err) = persist_transition(&state, &ceremony, &transition).await {
        return orchestrator_error_response(&err);
    }

    Json(map_transition(&transition)).into_response()
}

async fn abort_ceremony(
    State(state): State<AppState>,
    auth: Option<Extension<AuthContext>>,
    Path(id): Path<String>,
    Json(request): Json<AbortRequest>,
) -> Response {
    let auth = match require_auth_context(auth, CEREMONIES_WRITE_SCOPE, MANAGEMENT_ROLES) {
        Ok(auth) => auth,
        Err(error) => return error.into_response(),
    };
    let actor = match actor_id(&auth) {
        Ok(actor) => actor,
        Err(error) => return error.into_response(),
    };

    let mut ceremony = match state.ceremony_repository.get_by_id(&CeremonyId(id)).await {
        Ok(ceremony) => ceremony,
        Err(err) => return orchestrator_error_response(&err),
    };

    let transition = match state
        .orchestrator
        .abort_ceremony(&mut ceremony, &request.reason, &actor)
        .await
    {
        Ok(transition) => transition,
        Err(err) => return orchestrator_error_response(&err),
    };

    if let Err(err) = persist_transition(&state, &ceremony, &transition).await {
        return orchestrator_error_response(&err);
    }

    Json(map_transition(&transition)).into_response()
}

async fn resume_ceremony(
    State(state): State<AppState>,
    auth: Option<Extension<AuthContext>>,
    Path(id): Path<String>,
    Json(request): Json<ResumeRequest>,
) -> Response {
    let auth = match require_auth_context(auth, CEREMONIES_WRITE_SCOPE, MANAGEMENT_ROLES) {
        Ok(auth) => auth,
        Err(error) => return error.into_response(),
    };
    let actor = match actor_id(&auth) {
        Ok(actor) => actor,
        Err(error) => return error.into_response(),
    };

    let mut ceremony = match state.ceremony_repository.get_by_id(&CeremonyId(id)).await {
        Ok(ceremony) => ceremony,
        Err(err) => return orchestrator_error_response(&err),
    };

    let transition = match state
        .orchestrator
        .resume_ceremony(&mut ceremony, &request.document_hash, &actor)
        .await
    {
        Ok(transition) => transition,
        Err(err) => return orchestrator_error_response(&err),
    };

    if let Err(err) = persist_transition(&state, &ceremony, &transition).await {
        return orchestrator_error_response(&err);
    }

    Json(map_transition(&transition)).into_response()
}

async fn authenticate_signer(
    auth: Option<Extension<AuthContext>>,
    Path((_ceremony_id, _signer_id)): Path<(String, String)>,
    Json(_request): Json<AuthenticateRequest>,
) -> Response {
    let _ = require_auth_context(auth, CEREMONIES_SIGN_SCOPE, SIGNER_ROLES);
    error_response(
        StatusCode::NOT_IMPLEMENTED,
        "SAHI_3199",
        "Signer authentication is pending the explicit signer identity model",
    )
}

async fn sign(
    auth: Option<Extension<AuthContext>>,
    Path((_ceremony_id, _signer_id)): Path<(String, String)>,
    Json(_request): Json<SignRequest>,
) -> Response {
    let _ = require_auth_context(auth, CEREMONIES_SIGN_SCOPE, SIGNER_ROLES);
    error_response(
        StatusCode::NOT_IMPLEMENTED,
        "SAHI_3199",
        "Signer submission is pending the explicit signer identity model and TSA/LTV provider selection",
    )
}

fn build_signers(signers: Vec<SignerInput>) -> Result<Vec<SignerSlot>, ApiError> {
    let mut built = Vec::with_capacity(signers.len());
    for signer in signers {
        let role = match parse_signer_role(&signer.role) {
            Ok(role) => role,
            Err(message) => {
                return Err(ApiError::new(StatusCode::BAD_REQUEST, "SAHI_3019", message));
            }
        };

        built.push(SignerSlot {
            id: SignerSlotId(generate(UlidPrefix::SignerSlot)),
            order: signer.order,
            name: signer.name,
            email: signer.email,
            role,
            is_required: signer.is_required,
            status: SignerStatus::Pending,
            invitation: None,
            webauthn_credential_id: None,
            assurance_level: None,
            signed_at: None,
            signature_data: None,
            decline_reason: None,
        });
    }

    Ok(built)
}

fn parse_signer_role(role: &str) -> Result<SignerRole, String> {
    match role {
        "signatory" => Ok(SignerRole::Signatory),
        "witness" => Ok(SignerRole::Witness),
        "approver" => Ok(SignerRole::Approver),
        "notary" => Ok(SignerRole::Notary),
        _ => Err(format!("Unsupported signer role: {role}")),
    }
}

fn require_auth_context(
    auth: Option<Extension<AuthContext>>,
    scope: &str,
    allowed_roles: &[Role],
) -> Result<AuthContext, ApiError> {
    let Some(Extension(auth)) = auth else {
        return Err(ApiError::new(
            StatusCode::UNAUTHORIZED,
            "SAHI_1100",
            "Authenticated bearer token required",
        ));
    };

    if auth.has_scope(scope) || auth.has_any_role(allowed_roles) {
        Ok(auth)
    } else {
        Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "SAHI_1101",
            "Token does not have the required role or scope",
        ))
    }
}

fn actor_id(auth: &AuthContext) -> Result<String, ApiError> {
    auth.actor_id().map(str::to_string).ok_or_else(|| {
        ApiError::new(
            StatusCode::UNAUTHORIZED,
            "SAHI_1100",
            "Authenticated token must include sub, client_id, or azp",
        )
    })
}

async fn persist_transition(
    state: &AppState,
    ceremony: &Ceremony,
    transition: &CeremonyTransition,
) -> Result<(), OrchestratorError> {
    state.ceremony_repository.update(ceremony).await?;
    state
        .transition_repository
        .record(&ceremony.id.0, &ceremony.tenant_id, transition)
        .await?;
    Ok(())
}

fn map_ceremony(ceremony: &Ceremony) -> CeremonyResponse {
    CeremonyResponse {
        id: ceremony.id.0.clone(),
        state: ceremony.state.to_string(),
        title: ceremony.metadata.title.clone(),
        description: ceremony.metadata.description.clone(),
        reference: ceremony.metadata.reference.clone(),
        tags: ceremony.metadata.tags.clone(),
        created_at: ceremony.created_at.clone(),
        expires_at: ceremony.expires_at.clone(),
        signers: ceremony
            .signers
            .iter()
            .map(|signer| SignerResponse {
                id: signer.id.0.clone(),
                name: signer.name.clone(),
                email: signer.email.clone(),
                role: signer.role.as_str().to_string(),
                status: signer.status.to_string(),
                signed_at: signer.signed_at.clone(),
            })
            .collect(),
    }
}

fn map_transition(transition: &CeremonyTransition) -> TransitionResponse {
    TransitionResponse {
        from_state: transition.from_state.to_string(),
        to_state: transition.to_state.to_string(),
        timestamp: transition.timestamp.clone(),
    }
}

fn orchestrator_error_response(error: &OrchestratorError) -> Response {
    let status = match error {
        OrchestratorError::InvalidTransition(_)
        | OrchestratorError::DocumentHashMismatch { .. }
        | OrchestratorError::SignersIncomplete { .. }
        | OrchestratorError::AssuranceLevelTooLow { .. }
        | OrchestratorError::AbortFailed(_)
        | OrchestratorError::ResumeFailed(_)
        | OrchestratorError::DocumentError(_) => StatusCode::BAD_REQUEST,
        OrchestratorError::SignerNotReady | OrchestratorError::AlreadySigned => StatusCode::CONFLICT,
        OrchestratorError::CeremonyExpiredError => StatusCode::GONE,
        OrchestratorError::CeremonyNotFound(_) | OrchestratorError::SignerNotFound(_) => {
            StatusCode::NOT_FOUND
        }
        OrchestratorError::WebAuthnFailed(_) => StatusCode::UNAUTHORIZED,
        OrchestratorError::DatabaseError(_)
        | OrchestratorError::MerkleLogError(_)
        | OrchestratorError::KmsError(_)
        | OrchestratorError::TimestampError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    error_response(status, error.code(), &error.to_string())
}

fn error_response(status: StatusCode, code: &str, message: &str) -> Response {
    (
        status,
        Json(ErrorResponse {
            code: code.to_string(),
            message: message.to_string(),
        }),
    )
        .into_response()
}

fn generate_invitation_token() -> Result<String, ApiError> {
    let rng = SystemRandom::new();
    let mut token = [0u8; 32];
    rng.fill(&mut token).map_err(|_| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SAHI_3016",
            "Failed to generate signer invitation token",
        )
    })?;
    Ok(URL_SAFE_NO_PAD.encode(token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_deserialization() {
        let json = r#"{
            "title": "Test Ceremony",
            "document": {
                "filename": "test.pdf",
                "content_type": "application/pdf",
                "content_hash": "abc123",
                "size_bytes": 1024,
                "storage_key": "docs/test.pdf"
            },
            "signers": [
                {
                    "name": "Alice",
                    "email": "alice@example.com",
                    "role": "signatory",
                    "is_required": true,
                    "order": 1
                }
            ]
        }"#;

        let request: CreateCeremonyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.title, "Test Ceremony");
        assert_eq!(request.signers.len(), 1);
        assert_eq!(request.signers[0].name, "Alice");
    }

    #[test]
    fn test_parse_signer_role_rejects_invalid_values() {
        assert!(parse_signer_role("signatory").is_ok());
        assert!(parse_signer_role("invalid").is_err());
    }

    #[test]
    fn test_invitation_token_is_url_safe() {
        let token = generate_invitation_token().unwrap().to_string();
        assert!(!token.contains('='));
        assert!(token.len() >= 43);
    }
}
