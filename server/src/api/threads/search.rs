use crate::state::AppState;
use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use super::MAX_PAGE_LIMIT;

const MAX_SEARCH_QUERY_LEN: usize = 500;
const SEARCH_OVERLOAD_MESSAGE: &str = "Too many concurrent search requests";

pub(super) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/search", get(search_messages))
        .route("/api/search/advanced", post(advanced_search_handler))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<usize>,
}

async fn search_messages(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    if query.q.len() > MAX_SEARCH_QUERY_LEN {
        return Err(crate::api::error::ApiError::bad_request(format!(
            "Search query too long (max {} characters)",
            MAX_SEARCH_QUERY_LEN
        )));
    }
    let limit = query.limit.map(|l| l.min(MAX_PAGE_LIMIT));
    let _permit = state
        .rpc_semaphore
        .clone()
        .try_acquire_owned()
        .map_err(|_| crate::api::error::ApiError::too_many_requests(SEARCH_OVERLOAD_MESSAGE))?;
    let hits =
        crate::rpc::search::search_messages(axum::extract::State(state), query.q, limit).await?;

    Ok(Json(serde_json::json!({
        "hits": hits,
        "total": hits.len(),
    })))
}

async fn advanced_search_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let query: crate::rpc::advanced_search::AdvancedSearchQuery = serde_json::from_value(
        body.get("query")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    )
    .map_err(|e| crate::api::error::ApiError::bad_request(e.to_string()))?;
    let limit: Option<usize> = body
        .get("limit")
        .and_then(|v| v.as_u64().map(|n| n as usize))
        .map(|l| l.min(MAX_PAGE_LIMIT));
    let _permit = state
        .rpc_semaphore
        .clone()
        .try_acquire_owned()
        .map_err(|_| crate::api::error::ApiError::too_many_requests(SEARCH_OVERLOAD_MESSAGE))?;
    let hits =
        crate::rpc::advanced_search::advanced_search(axum::extract::State(state), query, limit)
            .await?;
    Ok(Json(serde_json::json!({
        "hits": hits,
        "total": hits.len(),
    })))
}
