// API integration test target root.
// Entry point is main.rs — Cargo requirement for directory-based integration test targets.

mod auth;
mod compose;
mod health;
mod messages;
mod notifications;
mod search;
mod shell;
mod snooze;
mod trusted_senders;

use axum::{
    extract::State,
    routing::{get, post},
    Router,
};
use pebble::api::api_routes;
use pebble::api::auth_api;
use pebble::auth as oauth_auth;
use pebble::middleware;
use pebble::state::AppState;
use std::sync::Arc;
use tempfile::TempDir;

/// Route that requires `State<Arc<AppState>>` for type inference.
async fn __state_marker(_: State<Arc<AppState>>) -> &'static str {
    unreachable!()
}

/// Creates a fully wired test app Router backed by in-memory SQLite.
/// Returns the Router, TempDir, and the underlying AppState for test setup.
pub async fn test_app() -> (Router, TempDir, Arc<AppState>) {
    let dir = tempfile::tempdir().unwrap();

    let store = pebble_store::Store::open_in_memory().unwrap();
    let search = pebble_search::TantivySearch::open_in_memory().unwrap();

    let key_path = dir.path().join("pebble.key");
    let crypto = pebble_crypto::CryptoService::init(&key_path).unwrap();

    let attachments_dir = dir.path().join("attachments");
    std::fs::create_dir_all(&attachments_dir).unwrap();

    let (snooze_stop_tx, _snooze_stop_rx) = std::sync::mpsc::channel::<()>();
    // bcrypt hash of "test-password", cost 4 (fast for tests)
    let password_hash = bcrypt::hash("test-password", 4).unwrap();
    let state = Arc::new(AppState::new(
        store,
        search,
        crypto,
        snooze_stop_tx,
        attachments_dir,
        password_hash,
    ));

    let app: Router = Router::new()
        .route("/__test_state_marker", get(__state_marker))
        .route("/events", get(|| async { "events" }))
        .route("/webhook/gmail", post(|| async { "ok" }))
        .route("/auth/login", get(oauth_auth::login_handler))
        .route("/auth/callback", get(oauth_auth::callback_handler))
        .merge(api_routes())
        .merge(auth_api::auth_routes())
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ))
        .with_state(state.clone());

    (app, dir, state)
}
