use crate::state::AppState;
use axum::{
    extract::{Path, State},
    routing::{delete, get},
    Json, Router,
};
use std::sync::Arc;

pub(super) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/templates", get(list_templates).post(save_template))
        .route("/api/templates/:id", delete(delete_template))
}

async fn list_templates(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<crate::rpc::user_data::EmailTemplate>>, crate::api::error::ApiError> {
    Ok(Json(
        crate::rpc::user_data::list_email_templates(axum::extract::State(state)).await?,
    ))
}

async fn save_template(
    State(state): State<Arc<AppState>>,
    Json(template): Json<crate::rpc::user_data::SaveEmailTemplateRequest>,
) -> Result<Json<crate::rpc::user_data::EmailTemplate>, crate::api::error::ApiError> {
    Ok(Json(
        crate::rpc::user_data::save_email_template(axum::extract::State(state), template).await?,
    ))
}

async fn delete_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::user_data::delete_email_template(axum::extract::State(state), id).await?;
    Ok(Json(()))
}
