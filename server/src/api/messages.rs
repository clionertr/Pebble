// Message read endpoints: batch fetch, single message, rendered HTML, full message.

use axum::{
    extract::{Query, State},
    Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct InboxQuery {
    #[serde(rename = "accountId")]
    pub account_id: String,
    #[serde(rename = "folderId")]
    pub folder_id: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    #[serde(rename = "folderIds")]
    pub folder_ids: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct StarredQuery {
    #[serde(rename = "accountId")]
    pub account_id: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub fn message_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/inbox", get(inbox_handler))
        .route("/api/starred", get(starred_handler))
        .route("/api/messages/{id}", get(get_message_handler))
        .route("/api/messages/batch", post(batch_messages_handler))
}

async fn inbox_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<InboxQuery>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let messages = crate::rpc::messages::query::list_messages(
        axum::extract::State(state),
        query.folder_id,
        query.folder_ids,
        query.limit.unwrap_or(50) as u32,
        query.offset.unwrap_or(0) as u32,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "messages": messages,
        "total": messages.len(),
        "hasMore": false,
    })))
}

async fn starred_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StarredQuery>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let messages = crate::rpc::messages::query::list_starred_messages(
        axum::extract::State(state),
        query.account_id,
        query.limit.unwrap_or(50) as u32,
        query.offset.unwrap_or(0) as u32,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "messages": messages,
        "total": messages.len(),
        "hasMore": false,
    })))
}

async fn get_message_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(message_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let msg = crate::rpc::messages::query::get_message(
        axum::extract::State(state),
        message_id,
    )
    .await?;

    match msg {
        Some(m) => Ok(Json(serde_json::to_value(m).unwrap())),
        None => Err(crate::api::error::ApiError::not_found("Message not found")),
    }
}

#[derive(Deserialize)]
pub struct BatchRequest {
    #[serde(rename = "messageIds")]
    pub message_ids: Vec<String>,
}

async fn batch_messages_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<BatchRequest>,
) -> Result<Json<Vec<pebble_core::Message>>, crate::api::error::ApiError> {
    let messages = crate::rpc::messages::query::get_messages_batch(
        axum::extract::State(state),
        body.message_ids,
    )
    .await?;
    Ok(Json(messages))
}
