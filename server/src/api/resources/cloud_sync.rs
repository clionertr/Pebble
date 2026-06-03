use crate::state::AppState;
use axum::{extract::State, routing::post, Json, Router};
use serde::Deserialize;
use std::sync::Arc;

pub(super) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/cloud-sync/webdav/test", post(webdav_test))
        .route("/api/cloud-sync/webdav/backup", post(webdav_backup))
        .route("/api/cloud-sync/webdav/preview", post(webdav_preview))
        .route("/api/cloud-sync/webdav/restore", post(webdav_restore))
}

#[derive(Deserialize)]
pub struct WebdavRequest {
    pub url: String,
    pub username: String,
    pub password: String,
}

async fn webdav_test(
    Json(b): Json<WebdavRequest>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let result =
        crate::rpc::cloud_sync::test_webdav_connection(b.url, b.username, b.password).await?;
    Ok(Json(serde_json::json!({ "status": result })))
}

async fn webdav_backup(
    State(state): State<Arc<AppState>>,
    Json(b): Json<WebdavRequest>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let result = crate::rpc::cloud_sync::backup_to_webdav(
        axum::extract::State(state),
        b.url,
        b.username,
        b.password,
    )
    .await?;
    Ok(Json(serde_json::json!({ "status": result })))
}

async fn webdav_preview(
    Json(b): Json<WebdavRequest>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let result =
        crate::rpc::cloud_sync::preview_webdav_backup(b.url, b.username, b.password).await?;
    Ok(Json(serde_json::to_value(result)?))
}

async fn webdav_restore(
    State(state): State<Arc<AppState>>,
    Json(b): Json<WebdavRequest>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let result = crate::rpc::cloud_sync::restore_from_webdav(
        axum::extract::State(state),
        b.url,
        b.username,
        b.password,
    )
    .await?;
    Ok(Json(serde_json::json!({ "status": result })))
}
