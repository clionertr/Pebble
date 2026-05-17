use pebble_core::{Message, PebbleError, ThreadSummary};


use crate::state::AppState;


pub async fn list_thread_messages(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    thread_id: String,
) -> std::result::Result<Vec<Message>, PebbleError> {
    let store = state.store.clone();
    tokio::task::spawn_blocking(move || store.list_messages_by_thread(&thread_id))
        .await
        .map_err(|e| PebbleError::Internal(format!("Task join error: {e}")))?
}


pub async fn list_threads(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    folder_id: String,
    folder_ids: Option<Vec<String>>,
    limit: u32,
    offset: u32,
) -> std::result::Result<Vec<ThreadSummary>, PebbleError> {
    let store = state.store.clone();
    tokio::task::spawn_blocking(move || match folder_ids {
        Some(ids) if !ids.is_empty() => store.list_threads_by_folders(&ids, limit, offset),
        _ => store.list_threads_by_folder(&folder_id, limit, offset),
    })
    .await
    .map_err(|e| PebbleError::Internal(format!("Task join error: {e}")))?
}
