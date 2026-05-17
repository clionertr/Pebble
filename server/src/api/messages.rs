// Message read endpoints: batch fetch, single message, rendered HTML, full message.

use axum::{
    extract::{Path, Query, State},
    Json,
    routing::{get, patch, post, delete},
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
        // Reads
        .route("/api/inbox", get(inbox_handler))
        .route("/api/starred", get(starred_handler))
        .route("/api/messages/{id}", get(get_message_handler))
        .route("/api/messages/batch", post(batch_messages_handler))
        // Mutations — single
        .route("/api/messages/{id}/flags", patch(update_flags_handler))
        .route("/api/messages/{id}/archive", post(archive_handler))
        .route("/api/messages/{id}/restore", post(restore_handler))
        .route("/api/messages/{id}/move", post(move_handler))
        .route("/api/messages/{id}", delete(delete_handler))
        // Mutations — batch
        .route("/api/messages/batch/archive", post(batch_archive_handler))
        .route("/api/messages/batch/delete", post(batch_delete_handler))
        .route("/api/messages/batch/read", post(batch_read_handler))
        .route("/api/messages/batch/star", post(batch_star_handler))
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

// ── Mutation handlers ─────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct UpdateFlagsRequest {
    #[serde(rename = "isRead")]
    pub is_read: Option<bool>,
    #[serde(rename = "isStarred")]
    pub is_starred: Option<bool>,
}

async fn update_flags_handler(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
    Json(body): Json<UpdateFlagsRequest>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::messages::flags::update_message_flags(
        axum::extract::State(state),
        message_id,
        body.is_read,
        body.is_starred,
    )
    .await?;
    Ok(Json(()))
}

async fn archive_handler(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
) -> Result<Json<String>, crate::api::error::ApiError> {
    let target = crate::rpc::messages::lifecycle::archive_message(
        axum::extract::State(state),
        message_id,
    )
    .await?;
    Ok(Json(target))
}

async fn delete_handler(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::messages::lifecycle::delete_message(
        axum::extract::State(state),
        message_id,
    )
    .await?;
    Ok(Json(()))
}

async fn restore_handler(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::messages::lifecycle::restore_message(
        axum::extract::State(state),
        message_id,
    )
    .await?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct MoveRequest {
    #[serde(rename = "targetFolderId")]
    pub target_folder_id: String,
}

async fn move_handler(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
    Json(body): Json<MoveRequest>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::messages::lifecycle::move_to_folder(
        axum::extract::State(state),
        message_id,
        body.target_folder_id,
    )
    .await?;
    Ok(Json(()))
}

async fn batch_archive_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<BatchRequest>,
) -> Result<Json<u32>, crate::api::error::ApiError> {
    let count = crate::rpc::batch::batch_archive(
        axum::extract::State(state),
        body.message_ids,
    )
    .await?;
    Ok(Json(count))
}

async fn batch_delete_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<BatchRequest>,
) -> Result<Json<u32>, crate::api::error::ApiError> {
    let count = crate::rpc::batch::batch_delete(
        axum::extract::State(state),
        body.message_ids,
    )
    .await?;
    Ok(Json(count))
}

#[derive(Deserialize)]
pub struct BatchReadRequest {
    #[serde(rename = "messageIds")]
    pub message_ids: Vec<String>,
    #[serde(rename = "isRead")]
    pub is_read: bool,
}

async fn batch_read_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<BatchReadRequest>,
) -> Result<Json<u32>, crate::api::error::ApiError> {
    let count = crate::rpc::batch::batch_mark_read(
        axum::extract::State(state),
        body.message_ids,
        body.is_read,
    )
    .await?;
    Ok(Json(count))
}

#[derive(Deserialize)]
pub struct BatchStarRequest {
    #[serde(rename = "messageIds")]
    pub message_ids: Vec<String>,
    pub starred: bool,
}

async fn batch_star_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<BatchStarRequest>,
) -> Result<Json<u32>, crate::api::error::ApiError> {
    let count = crate::rpc::batch::batch_star(
        axum::extract::State(state),
        body.message_ids,
        body.starred,
    )
    .await?;
    Ok(Json(count))
}
