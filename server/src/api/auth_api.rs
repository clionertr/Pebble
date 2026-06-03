// POST /api/auth/login, POST /api/auth/logout, GET /api/auth/status

use axum::{
    extract::{ConnectInfo, State},
    response::Json,
    routing::{get, post},
    Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::{api::error::ApiError, state::AppState};

const SESSION_COOKIE: &str = "pebble_session";

#[derive(Deserialize)]
pub struct LoginRequest {
    password: String,
}

#[derive(Serialize)]
pub struct AuthStatus {
    authenticated: bool,
}

/// Build auth routes: login, logout, status.
pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/auth/login", post(login_handler))
        .route("/api/auth/logout", post(logout_handler))
        .route("/api/auth/status", get(status_handler))
}

async fn login_handler(
    State(state): State<Arc<AppState>>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    jar: CookieJar,
    Json(body): Json<LoginRequest>,
) -> Result<(CookieJar, Json<AuthStatus>), ApiError> {
    let source_ip = connect_info
        .map(|ci| ci.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let session_store = &state.session_store;

    if !session_store.check_rate_limit(&source_ip).await {
        return Err(ApiError::too_many_requests(
            "Too many login attempts. Try again later.",
        ));
    }

    if body.password.is_empty() {
        return Err(ApiError::bad_request("Password is required"));
    }

    if session_store.check_password(&body.password) {
        let session_id = session_store.create_session().await;
        let cookie = Cookie::build((SESSION_COOKIE, session_id))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(axum_extra::extract::cookie::SameSite::Strict)
            .max_age(cookie::time::Duration::days(7))
            .build();
        Ok((
            jar.add(cookie),
            Json(AuthStatus {
                authenticated: true,
            }),
        ))
    } else {
        session_store.record_failure(&source_ip).await;
        Err(ApiError::unauthorized("Invalid password"))
    }
}

async fn logout_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> impl axum::response::IntoResponse {
    if let Some(session_cookie) = jar.get(SESSION_COOKIE) {
        if let Err(error) = state
            .store
            .delete_notification_devices_by_session(session_cookie.value())
        {
            tracing::warn!("Failed to delete notification devices for logout session: {error}");
        }
        state
            .session_store
            .remove_session(session_cookie.value())
            .await;
    }

    let cookie = Cookie::build((SESSION_COOKIE, ""))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(axum_extra::extract::cookie::SameSite::Strict)
        .max_age(cookie::time::Duration::seconds(0))
        .build();
    (
        jar.add(cookie),
        Json(AuthStatus {
            authenticated: false,
        }),
    )
}

async fn status_handler(jar: CookieJar, State(state): State<Arc<AppState>>) -> Json<AuthStatus> {
    if let Some(session_cookie) = jar.get(SESSION_COOKIE) {
        let authenticated = state
            .session_store
            .validate_session(session_cookie.value())
            .await;
        Json(AuthStatus { authenticated })
    } else {
        Json(AuthStatus {
            authenticated: false,
        })
    }
}
