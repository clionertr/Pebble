use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

pub(super) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/kanban", get(list_kanban))
        .route("/api/kanban/cards", post(move_to_kanban_handler))
        .route(
            "/api/kanban/cards/:messageId",
            delete(remove_from_kanban_handler),
        )
        .route("/api/kanban/notes/:messageId", put(set_kanban_note_handler))
        .route(
            "/api/kanban/notes",
            get(list_kanban_notes_handler).patch(merge_kanban_notes_handler),
        )
}

#[derive(Deserialize)]
pub struct KanbanQuery {
    pub column: Option<String>,
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

    let cards =
        crate::rpc::kanban::list_kanban_cards(axum::extract::State(state.clone()), column).await?;

    let notes = crate::rpc::kanban::list_kanban_context_notes(axum::extract::State(state)).await?;

    Ok(Json(serde_json::json!({
        "cards": cards,
        "notes": notes,
    })))
}

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
        _ => {
            return Err(crate::api::error::ApiError::bad_request(format!(
                "Invalid column: {}",
                body.column
            )))
        }
    };
    crate::rpc::kanban::move_to_kanban(
        axum::extract::State(state),
        body.message_id,
        column,
        body.position,
    )
    .await?;
    Ok(Json(()))
}

async fn remove_from_kanban_handler(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::kanban::remove_from_kanban(axum::extract::State(state), message_id).await?;
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
    )
    .await?;
    Ok(Json(serde_json::to_value(notes)?))
}

async fn merge_kanban_notes_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let notes_value = body.get("notes").cloned().unwrap_or(body);
    let notes: std::collections::HashMap<String, String> = serde_json::from_value(notes_value)
        .map_err(|e| crate::api::error::ApiError::bad_request(e.to_string()))?;
    let result =
        crate::rpc::kanban::merge_kanban_context_notes(axum::extract::State(state), notes).await?;
    Ok(Json(serde_json::to_value(result)?))
}

async fn list_kanban_notes_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let notes = crate::rpc::kanban::list_kanban_context_notes(axum::extract::State(state)).await?;
    Ok(Json(serde_json::to_value(notes)?))
}
