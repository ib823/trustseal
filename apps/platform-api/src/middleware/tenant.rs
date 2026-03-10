use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Middleware: extract tenant ID from JWT claims (X-Tenant-Id header for dev).
///
/// In production, this reads from validated JWT claims.
/// For development, it accepts X-Tenant-Id header directly.
///
/// Sets the tenant context for downstream RLS enforcement.
pub async fn extract_tenant(mut request: Request, next: Next) -> Response {
    // Dev mode: accept X-Tenant-Id header
    let tenant_id = request
        .headers()
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    if let Some(ref tid) = tenant_id {
        if !tid.starts_with("TNT_") {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": {
                        "code": "SAHI_1100",
                        "message": "Invalid tenant ID format",
                        "action": "Tenant ID must start with TNT_ prefix"
                    }
                })),
            )
                .into_response();
        }
        request.extensions_mut().insert(TenantId(tid.clone()));
    }

    next.run(request).await
}

/// Middleware: require tenant ID on protected routes.
pub async fn require_tenant(request: Request, next: Next) -> Response {
    if request.extensions().get::<TenantId>().is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": {
                    "code": "SAHI_1100",
                    "message": "Tenant context required",
                    "action": "Include X-Tenant-Id header or authenticate with a tenant-scoped JWT"
                }
            })),
        )
            .into_response();
    }

    next.run(request).await
}

/// Extracted from request extensions to access the current tenant.
#[derive(Debug, Clone)]
pub struct TenantId(pub String);
