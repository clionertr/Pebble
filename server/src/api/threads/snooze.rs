use crate::state::AppState;
use axum::{
    extract::{Path, State},
    routing::{delete, get},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

pub(super) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/snoozed",
            get(list_snoozed).post(snooze_message_handler),
        )
        .route("/api/snoozed/:messageId", delete(unsnooze_message_handler))
}

#[derive(Deserialize)]
pub struct SnoozeMessageBody {
    #[serde(rename = "messageId")]
    pub message_id: String,
    pub until: i64,
    #[serde(rename = "returnTo")]
    pub return_to: String,
}

async fn snooze_message_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SnoozeMessageBody>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::snooze::snooze_message(
        axum::extract::State(state),
        body.message_id,
        body.until,
        body.return_to,
    )
    .await?;
    Ok(Json(()))
}

async fn unsnooze_message_handler(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::snooze::unsnooze_message(axum::extract::State(state), message_id).await?;
    Ok(Json(()))
}

async fn list_snoozed(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<pebble_core::SnoozedMessage>>, crate::api::error::ApiError> {
    let messages = crate::rpc::snooze::list_snoozed(axum::extract::State(state)).await?;
    Ok(Json(messages))
}
