use crate::state::AppState;
use axum::{extract::State, routing::put, Json, Router};
use serde::Deserialize;
use std::sync::Arc;

pub(super) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/preferences/realtime", put(set_realtime))
        .route("/api/preferences/notifications", put(set_notifications))
}

#[derive(Deserialize)]
pub struct RealtimePref {
    pub mode: String,
}

async fn set_realtime(
    State(state): State<Arc<AppState>>,
    Json(b): Json<RealtimePref>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::sync_cmd::set_realtime_preference(axum::extract::State(state), b.mode).await?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct NotificationsPref {
    pub enabled: bool,
}

async fn set_notifications(
    State(state): State<Arc<AppState>>,
    Json(b): Json<NotificationsPref>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::notifications::set_notifications_enabled(axum::extract::State(state), b.enabled)
        .await?;
    Ok(Json(()))
}
