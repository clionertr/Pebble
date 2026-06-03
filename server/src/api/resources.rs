//! 聚合非账号/消息主干的资源 API 路由。

mod cloud_sync;
mod contacts;
mod diagnostics;
mod preferences;
mod proxy;
mod rules;
mod templates;
mod translate;
mod trusted_senders;

use crate::state::AppState;
use axum::Router;
use std::sync::Arc;

pub fn resource_routes() -> Router<Arc<AppState>> {
    Router::new()
        .merge(rules::routes())
        .merge(translate::routes())
        .merge(contacts::routes())
        .merge(cloud_sync::routes())
        .merge(trusted_senders::routes())
        .merge(templates::routes())
        .merge(preferences::routes())
        .merge(diagnostics::routes())
        .merge(proxy::routes())
}
