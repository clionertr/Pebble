use pebble_core::PebbleError;
use std::collections::HashMap;

pub async fn get_folder_unread_counts(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    account_id: String,
) -> std::result::Result<HashMap<String, u32>, PebbleError> {
    let store = state.store.clone();
    tokio::task::spawn_blocking(move || store.get_folder_unread_counts(&account_id))
        .await
        .map_err(|e| PebbleError::Internal(format!("Task join error: {e}")))?
}
