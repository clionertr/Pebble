use crate::state::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

pub(super) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/translate", post(translate_text))
        .route("/api/translate/stream", post(translate_text_stream))
        .route(
            "/api/translate/config",
            get(get_translate_config).put(save_translate_config),
        )
        .route("/api/translate/test", post(test_translate))
}

#[derive(Deserialize)]
pub struct TranslateRequest {
    pub text: String,
    #[serde(rename = "fromLang")]
    pub from: String,
    #[serde(rename = "toLang")]
    pub to: String,
}

async fn translate_text(
    State(state): State<Arc<AppState>>,
    Json(b): Json<TranslateRequest>,
) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let result =
        crate::rpc::translate::translate_text(axum::extract::State(state), b.text, b.from, b.to)
            .await?;
    Ok(Json(serde_json::to_value(result)?))
}

async fn translate_text_stream(
    State(state): State<Arc<AppState>>,
    Json(b): Json<TranslateRequest>,
) -> Result<impl IntoResponse, crate::api::error::ApiError> {
    let response = crate::rpc::translate::translate_text_stream(
        axum::extract::State(state),
        b.text,
        b.from,
        b.to,
    )
    .await?;
    let body = Body::from_stream(response.bytes_stream());
    let headers = [
        (header::CONTENT_TYPE, "text/event-stream; charset=utf-8"),
        (header::CACHE_CONTROL, "no-cache, no-transform"),
        (header::CONNECTION, "keep-alive"),
        (header::HeaderName::from_static("x-accel-buffering"), "no"),
    ];

    Ok((StatusCode::OK, headers, body))
}

async fn get_translate_config(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Option<pebble_core::TranslateConfig>>, crate::api::error::ApiError> {
    Ok(Json(
        crate::rpc::translate::get_translate_config(axum::extract::State(state)).await?,
    ))
}

#[derive(Deserialize)]
pub struct SaveTranslateConfigRequest {
    #[serde(rename = "providerType")]
    pub provider_type: String,
    pub config: String,
    #[serde(rename = "isEnabled")]
    pub enabled: bool,
}

async fn save_translate_config(
    State(state): State<Arc<AppState>>,
    Json(b): Json<SaveTranslateConfigRequest>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::translate::save_translate_config(
        axum::extract::State(state),
        b.provider_type,
        b.config,
        b.enabled,
    )
    .await?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct TestTranslateRequest {
    pub config: String,
}

async fn test_translate(
    State(state): State<Arc<AppState>>,
    Json(b): Json<TestTranslateRequest>,
) -> Result<Json<String>, crate::api::error::ApiError> {
    Ok(Json(
        crate::rpc::translate::test_translate_connection(axum::extract::State(state), b.config)
            .await?,
    ))
}
