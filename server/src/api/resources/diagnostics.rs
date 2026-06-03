use axum::{
    extract::Query,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::state::AppState;

pub(super) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/logs", get(read_logs))
        .route("/api/diagnostics/mail-timing", post(record_timing))
}

#[derive(Deserialize)]
pub struct LogsQuery {
    #[serde(rename = "maxBytes")]
    pub max_bytes: Option<u64>,
}

async fn read_logs(
    Query(q): Query<LogsQuery>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let snapshot = crate::rpc::diagnostics::read_app_log(q.max_bytes)?;
    Ok(Json(serde_json::to_value(snapshot)?))
}

async fn record_timing(
    Json(timing): Json<serde_json::Value>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    let timing: crate::rpc::diagnostics::MailDisplayTiming = serde_json::from_value(timing)
        .map_err(|e| crate::api::error::ApiError::bad_request(e.to_string()))?;
    crate::rpc::diagnostics::record_mail_display_timing(timing)?;
    Ok(Json(()))
}
