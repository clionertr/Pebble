use pebble_core::PebbleError;
use std::collections::HashMap;

pub async fn get_folder_unread_counts(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    account_id: String,
) -> std::result::Result<HashMap<String, u32>, PebbleError> {
    state
        .store
        .with_blocking_async(move |store| store.get_folder_unread_counts(&account_id))
        .await
}
