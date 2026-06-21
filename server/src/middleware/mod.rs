use axum::{
    extract::{Request, State},
    http::{header, HeaderName, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;

use crate::state::AppState;

const SESSION_COOKIE: &str = "pebble_session";
const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; img-src 'self' data: https:; script-src 'self'; style-src 'self'; style-src-elem 'self'; style-src-attr 'unsafe-inline'; connect-src 'self'; font-src 'self'; object-src 'none'; base-uri 'self'; form-action 'self'";

fn is_exempt(path: &str) -> bool {
    if path == "/events" {
        return false;
    }

    if !path.starts_with("/api/") {
        return true;
    }

    path == "/api/auth/login"
        || path == "/api/auth/logout"
        || path == "/api/auth/status"
        || path.starts_with("/api/docs")
}

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    if is_exempt(request.uri().path()) {
        return next.run(request).await;
    }

    let cookie_header = request
        .headers()
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let session_id = extract_cookie(cookie_header, SESSION_COOKIE);

    match session_id {
        Some(id) if state.session_store.validate_session(&id).await => next.run(request).await,
        _ => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Authentication required"})),
        )
            .into_response(),
    }
}

pub async fn security_headers_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(CONTENT_SECURITY_POLICY),
    );

    response
}

fn extract_cookie(header: &str, name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    header
        .split(';')
        .map(str::trim)
        .find(|part| part.starts_with(&prefix))
        .map(|part| part[prefix.len()..].to_string())
}
