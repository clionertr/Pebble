// Account-related mutation endpoints + proxy, sync commands, signatures.

use axum::{
    extract::{Path, State},
    Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use crate::state::AppState;

pub fn account_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/accounts", get(list_accounts))
        .route("/api/accounts/{id}/signature", get(get_signature).put(set_signature))
        .route("/api/accounts/{id}/sync/trigger", post(trigger_sync))
        .route("/api/accounts/{id}/trash", delete(empty_trash_handler))
}

// ── Handlers ─────────────────────────────────────────────────────────

async fn list_accounts(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<pebble_core::Account>>, crate::api::error::ApiError> {
    let accounts = crate::rpc::accounts::list_accounts(
        axum::extract::State(state),
    ).await?;
    Ok(Json(accounts))
}

#[derive(Deserialize)]
pub struct TriggerSyncRequest {
    pub reason: String,
}

async fn trigger_sync(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
    Json(body): Json<TriggerSyncRequest>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::sync_cmd::trigger_sync(
        axum::extract::State(state),
        account_id,
        body.reason,
    ).await?;
    Ok(Json(()))
}

async fn get_signature(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
) -> Result<Json<String>, crate::api::error::ApiError> {
    let sig = crate::rpc::user_data::get_email_signature(
        axum::extract::State(state),
        account_id,
    ).await?;
    Ok(Json(sig))
}

#[derive(Deserialize)]
pub struct SetSignatureRequest {
    pub signature: String,
}

async fn set_signature(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
    Json(body): Json<SetSignatureRequest>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::user_data::set_email_signature(
        axum::extract::State(state),
        account_id,
        body.signature,
    ).await?;
    Ok(Json(()))
}

async fn empty_trash_handler(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
) -> Result<Json<u32>, crate::api::error::ApiError> {
    let count = crate::rpc::messages::lifecycle::empty_trash(
        axum::extract::State(state),
        account_id,
    ).await?;
    Ok(Json(count))
}
