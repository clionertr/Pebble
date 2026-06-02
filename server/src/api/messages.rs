// Message read endpoints: batch fetch, single message, rendered HTML, full message.

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::state::AppState;
use pebble_core::PrivacyMode;

const MAX_PAGE_LIMIT: usize = 500;

#[derive(Deserialize)]
pub struct InboxQuery {
    #[serde(rename = "accountId")]
    pub account_id: Option<String>,
    #[serde(rename = "folderId")]
    pub folder_id: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    #[serde(rename = "folderIds")]
    pub folder_ids_raw: Option<String>,
}

impl InboxQuery {
    pub fn folder_ids(&self) -> Option<Vec<String>> {
        self.folder_ids_raw.as_ref().map(|s| {
            s.split(',')
                .map(|id| id.trim().to_string())
                .filter(|id| !id.is_empty())
                .collect()
        })
    }
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
        .route("/api/messages/:id", get(get_message_handler))
        .route("/api/messages/:id/html", get(html_handler))
        .route("/api/messages/:id/full", get(full_handler))
        .route("/api/messages/batch", post(batch_messages_handler))
        // Mutations — single
        .route("/api/messages/:id/flags", patch(update_flags_handler))
        .route("/api/messages/:id/archive", post(archive_handler))
        .route("/api/messages/:id/restore", post(restore_handler))
        .route("/api/messages/:id/move", post(move_handler))
        .route("/api/messages/:id", delete(delete_handler))
        // Mutations — batch
        .route("/api/messages/batch/archive", post(batch_archive_handler))
        .route("/api/messages/batch/delete", post(batch_delete_handler))
        .route("/api/messages/batch/read", post(batch_read_handler))
        .route("/api/messages/batch/star", post(batch_star_handler))
        .route("/api/pending-ops", get(list_pending_ops_handler))
        .route("/api/pending-ops/summary", get(pending_ops_summary_handler))
        .route(
            "/api/pending-ops/:id/cancel",
            post(cancel_pending_op_handler),
        )
        .route("/api/pending-ops/:id", delete(delete_pending_op_handler))
}

async fn inbox_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<InboxQuery>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let limit = query.limit.unwrap_or(50).min(MAX_PAGE_LIMIT) as u32;
    let messages = crate::rpc::messages::query::list_messages(
        axum::extract::State(state),
        query.folder_id.clone(),
        query.folder_ids(),
        limit,
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
    let limit = query.limit.unwrap_or(50).min(MAX_PAGE_LIMIT) as u32;
    let messages = crate::rpc::messages::query::list_starred_messages(
        axum::extract::State(state),
        query.account_id,
        limit,
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
    let msg =
        crate::rpc::messages::query::get_message(axum::extract::State(state), message_id).await?;

    match msg {
        Some(m) => Ok(Json(serde_json::to_value(m)?)),
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
    let target =
        crate::rpc::messages::lifecycle::archive_message(axum::extract::State(state), message_id)
            .await?;
    Ok(Json(target))
}

async fn delete_handler(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::messages::lifecycle::delete_message(axum::extract::State(state), message_id)
        .await?;
    Ok(Json(()))
}

async fn restore_handler(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::messages::lifecycle::restore_message(axum::extract::State(state), message_id)
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
    let count =
        crate::rpc::batch::batch_archive(axum::extract::State(state), body.message_ids).await?;
    Ok(Json(count))
}

async fn batch_delete_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<BatchRequest>,
) -> Result<Json<u32>, crate::api::error::ApiError> {
    let count =
        crate::rpc::batch::batch_delete(axum::extract::State(state), body.message_ids).await?;
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
    let count =
        crate::rpc::batch::batch_star(axum::extract::State(state), body.message_ids, body.starred)
            .await?;
    Ok(Json(count))
}

// ── HTML / Full message ──────────────────────────────────────────────

#[derive(Deserialize)]
pub struct HtmlQuery {
    #[serde(rename = "privacyMode")]
    pub privacy_mode: Option<String>,
}

fn parse_privacy_mode(value: Option<&str>) -> PrivacyMode {
    match value {
        Some("strict" | "Strict") => PrivacyMode::Strict,
        Some("off" | "Off") => PrivacyMode::Off,
        Some("load_once" | "LoadOnce") => PrivacyMode::LoadOnce,
        Some(s) if s.starts_with("trust:") => PrivacyMode::TrustSender(s[6..].to_string()),
        _ => PrivacyMode::Strict,
    }
}

async fn html_handler(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
    Query(query): Query<HtmlQuery>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let privacy = parse_privacy_mode(query.privacy_mode.as_deref());
    let html = crate::rpc::messages::rendering::get_rendered_html(
        axum::extract::State(state),
        message_id,
        privacy,
    )
    .await?;
    Ok(Json(serde_json::to_value(html)?))
}

async fn full_handler(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
    Query(query): Query<HtmlQuery>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let privacy = parse_privacy_mode(query.privacy_mode.as_deref());
    let result = crate::rpc::messages::rendering::get_message_with_html(
        axum::extract::State(state),
        message_id,
        privacy,
    )
    .await?;
    Ok(Json(serde_json::to_value(result)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_privacy_mode_query_values() {
        assert!(matches!(
            parse_privacy_mode(Some("strict")),
            PrivacyMode::Strict
        ));
        assert!(matches!(
            parse_privacy_mode(Some("Strict")),
            PrivacyMode::Strict
        ));
        assert!(matches!(
            parse_privacy_mode(Some("load_once")),
            PrivacyMode::LoadOnce
        ));
        assert!(matches!(
            parse_privacy_mode(Some("LoadOnce")),
            PrivacyMode::LoadOnce
        ));
        assert!(matches!(parse_privacy_mode(Some("off")), PrivacyMode::Off));
        assert!(matches!(parse_privacy_mode(Some("Off")), PrivacyMode::Off));

        let mode = parse_privacy_mode(Some("trust:sender@example.com"));
        assert!(matches!(mode, PrivacyMode::TrustSender(sender) if sender == "sender@example.com"));
    }

    #[test]
    fn defaults_unknown_privacy_mode_to_strict() {
        assert!(matches!(parse_privacy_mode(None), PrivacyMode::Strict));
        assert!(matches!(
            parse_privacy_mode(Some("LoadImages")),
            PrivacyMode::Strict
        ));
    }
}

// ── Pending Ops ──────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct PendingOpsQuery {
    #[serde(rename = "accountId")]
    pub account_id: Option<String>,
    pub limit: Option<usize>,
}

async fn pending_ops_summary_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PendingOpsQuery>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let summary = crate::rpc::pending_mail_ops::get_pending_mail_ops_summary(
        axum::extract::State(state),
        query.account_id,
    )?;
    Ok(Json(serde_json::to_value(summary)?))
}

async fn list_pending_ops_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PendingOpsQuery>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let ops = crate::rpc::pending_mail_ops::list_pending_mail_ops(
        axum::extract::State(state),
        query.account_id,
        query.limit.map(|n| (n as i64).min(MAX_PAGE_LIMIT as i64)),
    )?;
    Ok(Json(serde_json::to_value(ops)?))
}

async fn cancel_pending_op_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::pending_mail_ops::cancel_pending_mail_op(axum::extract::State(state), id)?;
    Ok(Json(()))
}

async fn delete_pending_op_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::pending_mail_ops::delete_pending_mail_op(axum::extract::State(state), id)?;
    Ok(Json(()))
}
