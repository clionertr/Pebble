// Send email + drafts endpoints.

use axum::{
    extract::{Path, Query, State},
    Json,
    routing::{delete, post},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use crate::state::AppState;

pub fn compose_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/messages/send", post(send_handler))
        .route("/api/drafts", post(save_draft_handler))
        .route("/api/drafts/:id", delete(delete_draft_handler))
}

// ── Send ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SendRequest {
    #[serde(rename = "accountId")]
    pub account_id: String,
    pub to: Vec<String>,
    pub cc: Option<Vec<String>>,
    pub bcc: Option<Vec<String>>,
    pub subject: String,
    #[serde(rename = "bodyText")]
    pub body_text: String,
    #[serde(rename = "bodyHtml")]
    pub body_html: Option<String>,
    #[serde(rename = "inReplyTo")]
    pub in_reply_to: Option<String>,
    #[serde(rename = "attachmentPaths")]
    pub attachment_paths: Option<Vec<String>>,
}

async fn send_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SendRequest>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::compose::send_email(
        axum::extract::State(state),
        body.account_id,
        body.to,
        body.cc.unwrap_or_default(),
        body.bcc.unwrap_or_default(),
        body.subject,
        body.body_text,
        body.body_html,
        body.in_reply_to,
        body.attachment_paths,
    )
    .await?;
    Ok(Json(()))
}

// ── Drafts ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SaveDraftRequest {
    #[serde(rename = "accountId")]
    pub account_id: String,
    pub to: Vec<String>,
    pub cc: Option<Vec<String>>,
    pub bcc: Option<Vec<String>>,
    pub subject: String,
    #[serde(rename = "bodyText")]
    pub body_text: String,
    #[serde(rename = "bodyHtml")]
    pub body_html: Option<String>,
    #[serde(rename = "inReplyTo")]
    pub in_reply_to: Option<String>,
    #[serde(rename = "existingDraftId")]
    pub existing_draft_id: Option<String>,
    #[serde(rename = "attachmentPaths")]
    pub attachment_paths: Option<Vec<String>>,
}

async fn save_draft_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SaveDraftRequest>,
) -> Result<Json<String>, crate::api::error::ApiError> {
    // The RPC save_draft doesn't take existing_draft_id directly — it's handled
    // by the frontend creating a new draft or updating. For API, we just save.
    let draft_id = crate::rpc::drafts::save_draft(
        axum::extract::State(state),
        body.account_id,
        body.to,
        body.cc.unwrap_or_default(),
        body.bcc.unwrap_or_default(),
        body.subject,
        body.body_text,
        body.body_html,
        body.in_reply_to,
        body.attachment_paths,
        body.existing_draft_id,
    )
    .await?;
    Ok(Json(draft_id))
}

#[derive(Deserialize)]
pub struct DeleteDraftQuery {
    #[serde(rename = "accountId")]
    pub account_id: String,
}

async fn delete_draft_handler(
    State(state): State<Arc<AppState>>,
    Path(draft_id): Path<String>,
    Query(query): Query<DeleteDraftQuery>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::drafts::delete_draft(
        axum::extract::State(state),
        query.account_id,
        draft_id,
    )
    .await?;
    Ok(Json(()))
}
