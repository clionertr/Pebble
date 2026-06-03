use crate::state::AppState;
use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

pub(super) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/trusted-senders",
            get(list_trusted)
                .post(trust_sender_handler)
                .delete(remove_trusted),
        )
        .route("/api/trusted-senders/check", get(check_trusted))
}

#[derive(Deserialize)]
pub struct TrustedQuery {
    #[serde(rename = "accountId")]
    pub account_id: Option<String>,
}

async fn list_trusted(
    State(state): State<Arc<AppState>>,
    Query(q): Query<TrustedQuery>,
) -> Result<Json<Vec<pebble_core::TrustedSender>>, crate::api::error::ApiError> {
    Ok(Json(
        crate::rpc::trusted_senders::list_trusted_senders(
            axum::extract::State(state),
            q.account_id,
        )
        .await?,
    ))
}

#[derive(Deserialize)]
pub struct TrustSenderRequest {
    #[serde(rename = "accountId")]
    pub account_id: String,
    pub email: String,
    #[serde(rename = "trustType")]
    pub trust_type: String,
}

async fn trust_sender_handler(
    State(state): State<Arc<AppState>>,
    Json(b): Json<TrustSenderRequest>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    let trust_type = match b.trust_type.as_str() {
        "all" => pebble_core::TrustType::All,
        _ => pebble_core::TrustType::Images,
    };
    crate::rpc::trusted_senders::trust_sender(
        axum::extract::State(state),
        b.account_id,
        b.email,
        trust_type,
    )
    .await?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct RemoveTrustedQuery {
    #[serde(rename = "accountId")]
    pub account_id: String,
    pub email: String,
}

async fn remove_trusted(
    State(state): State<Arc<AppState>>,
    Query(q): Query<RemoveTrustedQuery>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::trusted_senders::remove_trusted_sender(
        axum::extract::State(state),
        q.account_id,
        q.email,
    )
    .await?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct CheckTrustedQuery {
    #[serde(rename = "accountId")]
    pub account_id: String,
    pub email: String,
}

async fn check_trusted(
    State(state): State<Arc<AppState>>,
    Query(q): Query<CheckTrustedQuery>,
) -> Result<Json<bool>, crate::api::error::ApiError> {
    Ok(Json(
        crate::rpc::messages::rendering::is_trusted_sender(
            axum::extract::State(state),
            q.account_id,
            q.email,
        )
        .await?,
    ))
}
