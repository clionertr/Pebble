// Label read/mutation endpoints.

use axum::{
    extract::{Path, State},
    Json,
    routing::{delete, get, post},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use crate::state::AppState;

pub fn label_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/labels", get(list_labels))
        .route("/api/messages/:id/labels", get(get_labels).post(add_label))
        .route("/api/messages/:id/labels/:name", delete(remove_label))
        .route("/api/messages/batch/labels", post(get_labels_batch))
}

async fn list_labels(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<pebble_store::labels::Label>>, crate::api::error::ApiError> {
    let labels = crate::rpc::labels::list_labels(axum::extract::State(state)).await?;
    Ok(Json(labels))
}

async fn get_labels(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
) -> Result<Json<Vec<pebble_store::labels::Label>>, crate::api::error::ApiError> {
    let labels = crate::rpc::labels::get_message_labels(
        axum::extract::State(state), message_id,
    ).await?;
    Ok(Json(labels))
}

#[derive(Deserialize)]
pub struct AddLabelRequest {
    #[serde(rename = "labelName")]
    pub label_name: String,
}

async fn add_label(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
    Json(body): Json<AddLabelRequest>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::labels::add_message_label(
        axum::extract::State(state), message_id, body.label_name,
    ).await?;
    Ok(Json(()))
}

async fn remove_label(
    State(state): State<Arc<AppState>>,
    Path((message_id, name)): Path<(String, String)>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::labels::remove_message_label(
        axum::extract::State(state), message_id, name,
    ).await?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct BatchLabelRequest {
    #[serde(rename = "messageIds")]
    pub message_ids: Vec<String>,
}

async fn get_labels_batch(
    State(state): State<Arc<AppState>>,
    Json(body): Json<BatchLabelRequest>,
) -> Result<Json<std::collections::HashMap<String, Vec<pebble_store::labels::Label>>>, crate::api::error::ApiError> {
    let result = crate::rpc::labels::get_message_labels_batch(
        axum::extract::State(state), body.message_ids,
    ).await?;
    Ok(Json(result))
}
