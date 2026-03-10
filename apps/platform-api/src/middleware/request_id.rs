use axum::{extract::Request, middleware::Next, response::Response};

/// Middleware: inject a ULID request ID (REQ_ prefix) into every request.
/// Propagated to logs, audit events, and error responses.
pub async fn inject_request_id(mut request: Request, next: Next) -> Response {
    let request_id = format!("REQ_{}", ulid::Ulid::new());

    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));

    let mut response = next.run(request).await;

    response.headers_mut().insert(
        "x-request-id",
        request_id.parse().expect("ULID is always valid ASCII"),
    );

    response
}

/// Extracted from request extensions to access the current request ID.
#[derive(Debug, Clone)]
pub struct RequestId(pub String);
