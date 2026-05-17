pub mod auth_api;
pub mod error;
pub mod messages;
pub mod shell;
pub mod threads;

use axum::{Router, routing::get};
use std::sync::Arc;
use crate::state::AppState;

/// Build the /api router with all resource sub-routers nested under it.
/// Called from main.rs and composed into the top-level app.
pub fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/health", get(health_check))
        .merge(messages::message_routes())
        .merge(shell::shell_routes())
        .merge(threads::thread_routes())
}

/// GET /api/health — liveness probe.
async fn health_check() -> &'static str {
    "ok"
}
