// Thread and search read endpoints.

use axum::{
    extract::{Path, Query, State},
    Json,
    routing::get,
    Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct ThreadListQuery {
    #[serde(rename = "folderId")]
    pub folder_id: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    #[serde(rename = "folderIds")]
    pub folder_ids: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct KanbanQuery {
    pub column: Option<String>,
}

// ── Routes ───────────────────────────────────────────────────────────

pub fn thread_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/threads", get(list_threads))
        .route("/api/threads/{id}/messages", get(list_thread_messages))
        .route("/api/search", get(search_messages))
        .route("/api/kanban", get(list_kanban))
        .route("/api/snoozed", get(list_snoozed))
}

// ── Handlers ─────────────────────────────────────────────────────────

async fn list_threads(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ThreadListQuery>,
) -> Result<Json<Vec<pebble_core::ThreadSummary>>, crate::api::error::ApiError> {
    let threads = crate::rpc::threads::list_threads(
        axum::extract::State(state),
        query.folder_id,
        query.folder_ids,
        query.limit.unwrap_or(50) as u32,
        query.offset.unwrap_or(0) as u32,
    )
    .await?;
    Ok(Json(threads))
}

async fn list_thread_messages(
    State(state): State<Arc<AppState>>,
    Path(thread_id): Path<String>,
) -> Result<Json<Vec<pebble_core::Message>>, crate::api::error::ApiError> {
    let messages = crate::rpc::threads::list_thread_messages(
        axum::extract::State(state),
        thread_id,
    )
    .await?;
    Ok(Json(messages))
}

async fn search_messages(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let hits = crate::rpc::search::search_messages(
        axum::extract::State(state),
        query.q,
        query.limit,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "hits": hits,
        "total": hits.len(),
    })))
}

async fn list_kanban(
    State(state): State<Arc<AppState>>,
    Query(query): Query<KanbanQuery>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    use pebble_core::KanbanColumn;

    let column: Option<KanbanColumn> = match query.column.as_deref() {
        Some("todo") => Some(KanbanColumn::Todo),
        Some("waiting") => Some(KanbanColumn::Waiting),
        Some("done") => Some(KanbanColumn::Done),
        _ => None,
    };

    let cards = crate::rpc::kanban::list_kanban_cards(
        axum::extract::State(state.clone()),
        column,
    )
    .await?;

    let notes = crate::rpc::kanban::list_kanban_context_notes(
        axum::extract::State(state),
    )
    .await?;

    Ok(Json(serde_json::json!({
        "cards": cards,
        "notes": notes,
    })))
}

async fn list_snoozed(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<pebble_core::SnoozedMessage>>, crate::api::error::ApiError> {
    let messages = crate::rpc::snooze::list_snoozed(
        axum::extract::State(state),
    )
    .await?;
    Ok(Json(messages))
}
