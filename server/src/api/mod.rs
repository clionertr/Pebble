pub mod auth_api;
pub mod error;

use axum::{Router, routing::get};

/// Build the /api router with all resource sub-routers nested under it.
/// Called from main.rs and composed into the top-level app.
/// Generic over state type S so it can be merged into any Router.
pub fn api_routes<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/health", get(health_check))
}

/// GET /api/health — liveness probe.
async fn health_check() -> &'static str {
    "ok"
}
