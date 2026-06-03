use pebble_core::{KnownContact, PebbleError};

pub(crate) async fn search_contacts(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    account_id: String,
    query: String,
    limit: Option<i64>,
) -> std::result::Result<Vec<KnownContact>, PebbleError> {
    let limit = limit.unwrap_or(20);
    state.store.list_known_contacts(&account_id, &query, limit)
}
