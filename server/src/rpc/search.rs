use pebble_core::traits::SearchHit;
use pebble_core::PebbleError;

pub async fn search_messages(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    query: String,
    limit: Option<usize>,
) -> std::result::Result<Vec<SearchHit>, PebbleError> {
    const SNIPPET_MAX_LEN: usize = 150;

    let limit = limit.unwrap_or(50);
    let search = state.search.clone();
    let store = state.store.clone();
    let mut hits = tokio::task::spawn_blocking(move || search.search(&query, limit))
        .await
        .map_err(|e| PebbleError::Internal(format!("Task join error: {e}")))??;

    // Enrich search results with body snippets from SQLite since Tantivy
    // no longer stores the full body_text field.
    for hit in &mut hits {
        if let Ok(Some(msg)) = store.get_message(&hit.message_id) {
            let body = msg.body_text.trim();
            hit.snippet = if body.len() > SNIPPET_MAX_LEN {
                format!("{}…", &body[..body.floor_char_boundary(SNIPPET_MAX_LEN)])
            } else {
                body.to_string()
            };
        }
    }

    Ok(hits)
}
