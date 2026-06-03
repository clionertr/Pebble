use pebble_core::PebbleError;
use pebble_store::labels::Label;

pub(crate) async fn get_message_labels(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    message_id: String,
) -> std::result::Result<Vec<Label>, PebbleError> {
    state.store.get_message_labels(&message_id)
}

pub(crate) async fn get_message_labels_batch(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    message_ids: Vec<String>,
) -> std::result::Result<std::collections::HashMap<String, Vec<Label>>, PebbleError> {
    state.store.get_message_labels_batch(&message_ids)
}

pub(crate) async fn add_message_label(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    message_id: String,
    label_name: String,
) -> std::result::Result<(), PebbleError> {
    state.store.add_label(&message_id, &label_name)
}

pub(crate) async fn remove_message_label(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    message_id: String,
    label_name: String,
) -> std::result::Result<(), PebbleError> {
    state.store.remove_label(&message_id, &label_name)
}

pub(crate) async fn list_labels(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
) -> std::result::Result<Vec<Label>, PebbleError> {
    state.store.list_labels()
}
