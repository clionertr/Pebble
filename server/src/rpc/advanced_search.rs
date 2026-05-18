use pebble_core::traits::SearchHit;
use pebble_core::PebbleError;
use pebble_search::AdvancedSearchParams;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvancedSearchQuery {
    pub text: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub subject: Option<String>,
    pub date_from: Option<i64>,
    pub date_to: Option<i64>,
    pub has_attachment: Option<bool>,
    pub folder_id: Option<String>,
}

pub async fn advanced_search(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    query: AdvancedSearchQuery,
    limit: Option<usize>,
) -> std::result::Result<Vec<SearchHit>, PebbleError> {
    const SNIPPET_MAX_LEN: usize = 150;

    let search = state.search.clone();
    let store = state.store.clone();
    let limit = limit.unwrap_or(50);
    let mut hits = tokio::task::spawn_blocking(move || {
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
    .await
    .map_err(|e| PebbleError::Internal(format!("Task join error: {e}")))??;

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
