use crate::state::AppState;
use axum::{extract::State, routing::get, Json, Router};
use serde::Deserialize;
use std::sync::Arc;

pub(super) fn routes() -> Router<Arc<AppState>> {
    Router::new().route(
        "/api/proxy",
        get(get_global_proxy_handler).put(update_global_proxy_handler),
    )
}

async fn get_global_proxy_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Option<pebble_core::HttpProxyConfig>>, crate::api::error::ApiError> {
    let proxy =
        crate::rpc::network::get_global_proxy_raw(&state.crypto, &state.store).map_err(|e| {
            tracing::error!("Failed to get global proxy: {e}");
            crate::api::error::ApiError::internal("Internal server error")
        })?;
    Ok(Json(proxy))
}

#[derive(Deserialize)]
pub struct UpdateGlobalProxyBody {
    pub proxy_host: Option<String>,
    pub proxy_port: Option<u16>,
}

async fn update_global_proxy_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateGlobalProxyBody>,
) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::network::update_global_proxy(
        axum::extract::State(state),
        body.proxy_host,
        body.proxy_port,
    )
    .await?;
    Ok(Json(()))
}
