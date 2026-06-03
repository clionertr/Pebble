use crate::rpc::blocking::run_blocking;
use pebble_core::traits::{SearchHit, StructuredQuery};
use pebble_core::PebbleError;
use pebble_search::AdvancedSearchParams;

pub type AdvancedSearchQuery = StructuredQuery;

pub async fn advanced_search(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    query: AdvancedSearchQuery,
    limit: Option<usize>,
) -> std::result::Result<Vec<SearchHit>, PebbleError> {
    let search = state.search.clone();
    let limit = limit.unwrap_or(50);
    let hits = run_blocking(move || {
        search.advanced_search(AdvancedSearchParams {
            text: query.text.as_deref(),
            from: query.from.as_deref(),
            to: query.to.as_deref(),
            subject: query.subject.as_deref(),
            date_from: query.date_from,
            date_to: query.date_to,
            has_attachment: query.has_attachment,
            folder_id: query.folder_id.as_deref(),
            limit,
        })
    })
    .await?;

    Ok(hits)
}
