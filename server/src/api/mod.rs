pub mod accounts;
pub mod attachments;
pub mod auth_api;
pub mod compose;
pub mod error;
pub mod labels;
pub mod messages;
pub mod resources;
pub mod shell;
pub mod threads;

use axum::{Router, routing::get};
use std::sync::Arc;
use crate::state::AppState;

/// Build the /api router with all resource sub-routers nested under it.
pub fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/health", get(health_check))
        .merge(accounts::account_routes())
        .merge(attachments::attachment_routes())
        .merge(compose::compose_routes())
        .merge(labels::label_routes())
        .merge(messages::message_routes())
        .merge(resources::resource_routes())
        .merge(shell::shell_routes())
        .merge(threads::thread_routes())
}

async fn health_check() -> &'static str {
    "ok"
}