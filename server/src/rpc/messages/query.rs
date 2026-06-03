use axum::extract::State;
use pebble_core::{Message, MessageSummary, PebbleError};

pub(crate) async fn list_messages(
    state: State<std::sync::Arc<crate::state::AppState>>,
    folder_id: String,
    folder_ids: Option<Vec<String>>,
    limit: u32,
    offset: u32,
) -> std::result::Result<Vec<MessageSummary>, PebbleError> {
    state
        .store
        .with_blocking_async(move |store| match folder_ids {
            Some(ids) if !ids.is_empty() => store.list_messages_by_folders(&ids, limit, offset),
            _ => store.list_messages_by_folder(&folder_id, limit, offset),
        })
        .await
}

pub(crate) async fn list_starred_messages(
    state: State<std::sync::Arc<crate::state::AppState>>,
    account_id: String,
    limit: u32,
    offset: u32,
) -> std::result::Result<Vec<MessageSummary>, PebbleError> {
    state
        .store
        .with_blocking_async(move |store| store.list_starred_messages(&account_id, limit, offset))
        .await
}

pub(crate) async fn get_message(
    state: State<std::sync::Arc<crate::state::AppState>>,
    message_id: String,
) -> std::result::Result<Option<Message>, PebbleError> {
    state
        .store
        .with_blocking_async(move |store| store.get_message(&message_id))
        .await
}

pub(crate) async fn get_messages_batch(
    state: State<std::sync::Arc<crate::state::AppState>>,
    message_ids: Vec<String>,
) -> std::result::Result<Vec<Message>, PebbleError> {
    state
        .store
        .with_blocking_async(move |store| store.get_messages_batch(&message_ids))
        .await
}
