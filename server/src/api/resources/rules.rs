use crate::state::AppState;
use axum::{
    extract::{Path, State},
    routing::{get, put},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

pub(super) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/rules", get(list_rules).post(create_rule))
        .route("/api/rules/:id", put(update_rule).delete(delete_rule))
}

async fn list_rules(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<pebble_core::Rule>>, crate::api::error::ApiError> {
    Ok(Json(
        crate::rpc::rules::list_rules(axum::extract::State(state)).await?,
    ))
}

#[derive(Deserialize)]
pub struct CreateRuleRequest {
    pub name: String,
    pub priority: i32,
    pub conditions: String,
    pub actions: String,
}

async fn create_rule(
    State(state): State<Arc<AppState>>,
    Json(b): Json<CreateRuleRequest>,
) -> Result<Json<pebble_core::Rule>, crate::api::error::ApiError> {
    Ok(Json(
        crate::rpc::rules::create_rule(
            axum::extract::State(state),
            b.name,
            b.priority,
            b.conditions,
            b.actions,
        )
        .await?,
    ))
}

async fn update_rule(
    State(state): State<Arc<AppState>>,
    Path(_id): Path<String>,
    Json(rule): Json<pebble_core::Rule>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::rules::update_rule(axum::extract::State(state), rule).await?;
    Ok(Json(()))
}

async fn delete_rule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::rules::delete_rule(axum::extract::State(state), id).await?;
    Ok(Json(()))
}
