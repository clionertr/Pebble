use crate::state::AppState;
use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

pub(super) fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/contacts", get(search_contacts_handler))
}

#[derive(Deserialize)]
pub struct ContactsQuery {
    #[serde(rename = "accountId")]
    pub account_id: String,
    pub q: String,
    pub limit: Option<usize>,
}

async fn search_contacts_handler(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ContactsQuery>,
) -> Result<Json<Vec<pebble_core::KnownContact>>, crate::api::error::ApiError> {
    Ok(Json(
        crate::rpc::contacts::search_contacts(
            axum::extract::State(state),
            q.account_id,
            q.q,
            q.limit.map(|l| (l.min(500)) as i64),
        )
        .await?,
    ))
}
