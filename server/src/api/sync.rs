// 同步聚合入口：浏览器只表达“唤醒同步”的意图，账号级 worker 细节留给后端。

use crate::api::error::ApiError;
use crate::state::AppState;
use axum::{extract::State, routing::post, Json, Router};
use serde::Deserialize;
use std::sync::Arc;

pub fn sync_routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/sync/wake", post(wake_sync_handler))
}

#[derive(Debug, Deserialize)]
struct WakeSyncRequest {
    account_ids: Option<Vec<String>>,
    reason: String,
    ensure_running: Option<bool>,
    poll_interval_secs: Option<u64>,
}

async fn wake_sync_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<WakeSyncRequest>,
) -> Result<Json<crate::rpc::sync_cmd::SyncWakeResult>, ApiError> {
    let result = crate::rpc::sync_cmd::wake_sync(
        axum::extract::State(state),
        crate::rpc::sync_cmd::SyncWakeRequest {
            account_ids: body.account_ids,
            reason: body.reason,
            ensure_running: body.ensure_running.unwrap_or(false),
            poll_interval_secs: body.poll_interval_secs,
        },
    )
    .await?;
    Ok(Json(result))
}
