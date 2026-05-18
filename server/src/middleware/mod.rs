use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;

use crate::state::AppState;

const SESSION_COOKIE: &str = "pebble_session";

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

fn extract_cookie(header: &str, name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    header
        .split(';')
        .map(str::trim)
        .find(|part| part.starts_with(&prefix))
        .map(|part| part[prefix.len()..].to_string())
}
