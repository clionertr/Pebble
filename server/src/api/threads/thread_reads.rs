use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use super::MAX_PAGE_LIMIT;

pub(super) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/threads", get(list_threads))
        .route("/api/threads/:id/messages", get(list_thread_messages))
}

#[derive(Deserialize)]
pub struct ThreadListQuery {
    #[serde(rename = "folderId")]
    pub folder_id: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    #[serde(rename = "folderIds")]
    pub folder_ids_raw: Option<String>,
}

impl ThreadListQuery {
    pub fn folder_ids(&self) -> Option<Vec<String>> {
        self.folder_ids_raw.as_ref().map(|s| {
            s.split(',')
                .map(|id| id.trim().to_string())
                .filter(|id| !id.is_empty())
                .collect()
        })
    }
}

async fn list_threads(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ThreadListQuery>,
) -> Result<Json<Vec<pebble_core::ThreadSummary>>, crate::api::error::ApiError> {
    let limit = query.limit.unwrap_or(50).min(MAX_PAGE_LIMIT) as u32;
    let threads = crate::rpc::threads::list_threads(
        axum::extract::State(state),
        query.folder_id.clone(),
        query.folder_ids(),
        limit,
        query.offset.unwrap_or(0) as u32,
    )
    .await?;
    Ok(Json(threads))
}

async fn list_thread_messages(
    State(state): State<Arc<AppState>>,
    Path(thread_id): Path<String>,
) -> Result<Json<Vec<pebble_core::Message>>, crate::api::error::ApiError> {
    let messages =
        crate::rpc::threads::list_thread_messages(axum::extract::State(state), thread_id).await?;
    Ok(Json(messages))
}
