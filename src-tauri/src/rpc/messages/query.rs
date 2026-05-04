use crate::state::AppState;
use pebble_core::{Message, MessageSummary, PebbleError};
use axum::extract::State;

pub async fn list_messages(
    state: State<std::sync::Arc<crate::state::AppState>>,
    folder_id: String,
    folder_ids: Option<Vec<String>>,
    limit: u32,
    offset: u32,
) -> std::result::Result<Vec<MessageSummary>, PebbleError> {
    let store = state.store.clone();
    tokio::task::spawn_blocking(move || match folder_ids {
        Some(ids) if !ids.is_empty() => store.list_messages_by_folders(&ids, limit, offset),
        _ => store.list_messages_by_folder(&folder_id, limit, offset),
    })
    .await
    .map_err(|e| PebbleError::Internal(format!("Task join error: {e}")))?
}

pub async fn list_starred_messages(
    state: State<std::sync::Arc<crate::state::AppState>>,
    account_id: String,
    limit: u32,
    offset: u32,
) -> std::result::Result<Vec<MessageSummary>, PebbleError> {
    let store = state.store.clone();
    tokio::task::spawn_blocking(move || store.list_starred_messages(&account_id, limit, offset))
        .await
        .map_err(|e| PebbleError::Internal(format!("Task join error: {e}")))?
}

pub async fn get_message(
    state: State<std::sync::Arc<crate::state::AppState>>,
    message_id: String,
) -> std::result::Result<Option<Message>, PebbleError> {
    let store = state.store.clone();
    tokio::task::spawn_blocking(move || store.get_message(&message_id))
        .await
        .map_err(|e| PebbleError::Internal(format!("Task join error: {e}")))?
}

pub async fn get_messages_batch(
    state: State<std::sync::Arc<crate::state::AppState>>,
    message_ids: Vec<String>,
) -> std::result::Result<Vec<Message>, PebbleError> {
    let store = state.store.clone();
    tokio::task::spawn_blocking(move || store.get_messages_batch(&message_ids))
        .await
        .map_err(|e| PebbleError::Internal(format!("Task join error: {e}")))?
}
