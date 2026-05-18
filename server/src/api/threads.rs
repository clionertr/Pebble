// Thread and search read endpoints.

use axum::{
    extract::{Path, Query, State},
    Json,
    routing::{get, patch, post, delete, put},
    Router,
};
use serde::{Deserialize, Serialize};
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
        .route("/api/search/advanced", post(advanced_search_handler))
        .route("/api/kanban", get(list_kanban))
        .route("/api/kanban/cards", post(move_to_kanban_handler))
        .route("/api/kanban/cards/{messageId}", delete(remove_from_kanban_handler))
        .route("/api/kanban/notes/{messageId}", put(set_kanban_note_handler))
        .route("/api/kanban/notes", get(list_kanban_notes_handler).patch(merge_kanban_notes_handler))
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

async fn advanced_search_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let query: crate::rpc::advanced_search::AdvancedSearchQuery = serde_json::from_value(body.get("query").cloned().unwrap_or(serde_json::Value::Null))
        .map_err(|e| crate::api::error::ApiError::bad_request(e.to_string()))?;
    let limit: Option<usize> = body.get("limit").and_then(|v| v.as_u64().map(|n| n as usize));
    let hits = crate::rpc::advanced_search::advanced_search(
        axum::extract::State(state),
        query,
        limit,
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

// ── Kanban mutations ──────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct MoveToKanbanBody {
    #[serde(rename = "messageId")]
    pub message_id: String,
    pub column: String,
    pub position: Option<i32>,
}

async fn move_to_kanban_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<MoveToKanbanBody>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    use pebble_core::KanbanColumn;
    let column = match body.column.as_str() {
        "todo" => KanbanColumn::Todo,
        "waiting" => KanbanColumn::Waiting,
        "done" => KanbanColumn::Done,
        _ => return Err(crate::api::error::ApiError::bad_request(format!("Invalid column: {}", body.column))),
    };
    crate::rpc::kanban::move_to_kanban(
        axum::extract::State(state),
        body.message_id,
        column,
        body.position,
    ).await?;
    Ok(Json(()))
}

async fn remove_from_kanban_handler(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::kanban::remove_from_kanban(
        axum::extract::State(state),
        message_id,
    ).await?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct SetKanbanNoteBody {
    pub note: String,
}

async fn set_kanban_note_handler(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
    Json(body): Json<SetKanbanNoteBody>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let notes = crate::rpc::kanban::set_kanban_context_note(
        axum::extract::State(state),
        message_id,
        body.note,
    ).await?;
    Ok(Json(serde_json::to_value(notes).unwrap()))
}

async fn merge_kanban_notes_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let notes: std::collections::HashMap<String, String> = serde_json::from_value(body)
        .map_err(|e| crate::api::error::ApiError::bad_request(e.to_string()))?;
    let result = crate::rpc::kanban::merge_kanban_context_notes(
        axum::extract::State(state),
        notes,
    ).await?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}

async fn list_kanban_notes_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let notes = crate::rpc::kanban::list_kanban_context_notes(
        axum::extract::State(state),
    ).await?;
    Ok(Json(serde_json::to_value(notes).unwrap()))
}
