use pebble_core::traits::SearchHit;
use pebble_core::PebbleError;

pub async fn search_messages(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    query: String,
    limit: Option<usize>,
) -> std::result::Result<Vec<SearchHit>, PebbleError> {
    let limit = limit.unwrap_or(50);
    let search = state.search.clone();
    let hits = tokio::task::spawn_blocking(move || search.search(&query, limit))
        .await
        .map_err(|e| PebbleError::Internal(format!("Task join error: {e}")))??;

    Ok(hits)
}
