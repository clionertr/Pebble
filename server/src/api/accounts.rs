// Account-related mutation endpoints + proxy, sync commands, signatures.

use axum::{
    extract::{Path, State},
    Json,
    routing::{delete, get, patch, post, put},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use crate::state::AppState;
use crate::api::error::ApiError;

pub fn account_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/accounts", get(list_accounts).post(add_account_handler))
        .route("/api/accounts/{id}", patch(update_account_handler).delete(delete_account_handler))
        .route("/api/accounts/{id}/signature", get(get_signature).put(set_signature))
        .route("/api/accounts/{id}/sync/start", post(start_sync_handler))
        .route("/api/accounts/{id}/sync/trigger", post(trigger_sync))
        .route("/api/accounts/{id}/sync/stop", post(stop_sync_handler))
        .route("/api/accounts/{id}/test-connection", post(test_connection_handler))
        .route("/api/accounts/{id}/gmail-realtime", get(get_gmail_realtime).put(update_gmail_realtime_handler))
        .route("/api/accounts/{id}/gmail-realtime/enable", post(enable_gmail_realtime_handler))
        .route("/api/accounts/{id}/gmail-realtime/disable", post(disable_gmail_realtime_handler))
        .route("/api/accounts/{id}/proxy", get(get_account_proxy_handler).put(update_account_proxy_handler))
        .route("/api/accounts/{id}/proxy-setting", get(get_proxy_setting_handler).put(update_proxy_setting_handler))
        .route("/api/accounts/{id}/trash", delete(empty_trash_handler))
        .route("/api/imap/test-connection", post(test_imap_handler))
}

// ── Handlers ─────────────────────────────────────────────────────────

async fn list_accounts(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<pebble_core::Account>>, ApiError> {
    let accounts = crate::rpc::accounts::list_accounts(
        axum::extract::State(state),
    ).await?;
    Ok(Json(accounts))
}

#[derive(Deserialize)]
pub struct TriggerSyncRequest {
    pub reason: String,
}

async fn trigger_sync(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
    Json(body): Json<TriggerSyncRequest>,
) -> Result<Json<()>, ApiError> {
    crate::rpc::sync_cmd::trigger_sync(
        axum::extract::State(state),
        account_id,
        body.reason,
    ).await?;
    Ok(Json(()))
}

async fn get_signature(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
) -> Result<Json<String>, ApiError> {
    let sig = crate::rpc::user_data::get_email_signature(
        axum::extract::State(state),
        account_id,
    ).await?;
    Ok(Json(sig))
}

#[derive(Deserialize)]
pub struct SetSignatureRequest {
    pub signature: String,
}

async fn set_signature(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
    Json(body): Json<SetSignatureRequest>,
) -> Result<Json<()>, ApiError> {
    crate::rpc::user_data::set_email_signature(
        axum::extract::State(state),
        account_id,
        body.signature,
    ).await?;
    Ok(Json(()))
}

async fn empty_trash_handler(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
) -> Result<Json<u32>, ApiError> {
    let count = crate::rpc::messages::lifecycle::empty_trash(
        axum::extract::State(state),
        account_id,
    ).await?;
    Ok(Json(count))
}

// ── Account CRUD ───────────────────────────────────────────────────────

async fn add_account_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<crate::rpc::accounts::AddAccountRequest>,
) -> Result<Json<pebble_core::Account>, ApiError> {
    let account = crate::rpc::accounts::add_account(
        axum::extract::State(state),
        body,
    ).await?;
    Ok(Json(account))
}

#[derive(Deserialize)]
pub struct UpdateAccountBody {
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub password: Option<String>,
    pub imap_host: Option<String>,
    pub imap_port: Option<u16>,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub imap_security: Option<String>,
    pub smtp_security: Option<String>,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<u16>,
    pub account_color: Option<String>,
}

async fn update_account_handler(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
    Json(body): Json<UpdateAccountBody>,
) -> Result<Json<()>, ApiError> {
    let imap_sec: Option<pebble_mail::imap::ConnectionSecurity> =
        body.imap_security.and_then(|s| serde_json::from_str(&format!("\"{}\"", s)).ok());
    let smtp_sec: Option<pebble_mail::imap::ConnectionSecurity> =
        body.smtp_security.and_then(|s| serde_json::from_str(&format!("\"{}\"", s)).ok());
    crate::rpc::accounts::update_account(
        axum::extract::State(state),
        account_id,
        body.email.unwrap_or_default(),
        body.display_name.unwrap_or_default(),
        body.password,
        body.imap_host,
        body.imap_port,
        body.smtp_host,
        body.smtp_port,
        imap_sec,
        smtp_sec,
        body.proxy_host,
        body.proxy_port,
        body.account_color,
    ).await?;
    Ok(Json(()))
}

async fn delete_account_handler(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
) -> Result<Json<()>, ApiError> {
    crate::rpc::accounts::delete_account(
        axum::extract::State(state),
        account_id,
    ).await?;
    Ok(Json(()))
}

// ── Sync ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct StartSyncBody {
    pub poll_interval_secs: Option<u64>,
}

async fn start_sync_handler(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
    Json(body): Json<StartSyncBody>,
) -> Result<Json<String>, ApiError> {
    let result = crate::rpc::sync_cmd::start_sync(
        axum::extract::State(state),
        account_id,
        body.poll_interval_secs,
    ).await?;
    Ok(Json(result))
}

async fn stop_sync_handler(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
) -> Result<Json<()>, ApiError> {
    crate::rpc::sync_cmd::stop_sync(
        axum::extract::State(state),
        account_id,
    ).await?;
    Ok(Json(()))
}

// ── Connection testing ─────────────────────────────────────────────────

async fn test_connection_handler(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
) -> Result<Json<String>, ApiError> {
    let result = crate::rpc::accounts::test_account_connection(
        axum::extract::State(state),
        account_id,
    ).await?;
    Ok(Json(result))
}

async fn test_imap_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<crate::rpc::accounts::TestConnectionRequest>,
) -> Result<Json<String>, ApiError> {
    let result = crate::rpc::accounts::test_imap_connection(
        axum::extract::State(state),
        body,
    ).await?;
    Ok(Json(result))
}

// ── Gmail realtime ─────────────────────────────────────────────────────

async fn get_gmail_realtime(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
) -> Result<Json<crate::gmail_realtime::GmailRealtimeConfig>, ApiError> {
    let config = crate::rpc::gmail_realtime::get_gmail_realtime_config(
        axum::extract::State(state),
        account_id,
    ).await?;
    Ok(Json(config))
}

#[derive(Deserialize)]
pub struct EnableGmailRealtimeBody {
    pub fallback_interval_minutes: Option<u64>,
}

async fn enable_gmail_realtime_handler(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
    Json(body): Json<EnableGmailRealtimeBody>,
) -> Result<Json<crate::gmail_realtime::GmailRealtimeConfig>, ApiError> {
    let config = crate::rpc::gmail_realtime::enable_gmail_realtime(
        axum::extract::State(state),
        account_id,
        body.fallback_interval_minutes,
    ).await?;
    Ok(Json(config))
}

async fn disable_gmail_realtime_handler(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
) -> Result<Json<crate::gmail_realtime::GmailRealtimeConfig>, ApiError> {
    let config = crate::rpc::gmail_realtime::disable_gmail_realtime(
        axum::extract::State(state),
        account_id,
    ).await?;
    Ok(Json(config))
}

#[derive(Deserialize)]
pub struct UpdateGmailRealtimeBody {
    pub fallback_interval_minutes: u64,
}

async fn update_gmail_realtime_handler(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
    Json(body): Json<UpdateGmailRealtimeBody>,
) -> Result<Json<crate::gmail_realtime::GmailRealtimeConfig>, ApiError> {
    let config = crate::rpc::gmail_realtime::update_gmail_realtime_config(
        axum::extract::State(state),
        account_id,
        body.fallback_interval_minutes,
    ).await?;
    Ok(Json(config))
}

// ── Proxy ──────────────────────────────────────────────────────────────

async fn get_account_proxy_handler(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
) -> Result<Json<Option<pebble_core::HttpProxyConfig>>, ApiError> {
    let proxy = crate::rpc::accounts::get_account_proxy(
        axum::extract::State(state),
        account_id,
    ).await?;
    Ok(Json(proxy))
}

#[derive(Deserialize)]
pub struct UpdateProxyBody {
    pub proxy_host: Option<String>,
    pub proxy_port: Option<u16>,
}

async fn update_account_proxy_handler(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
    Json(body): Json<UpdateProxyBody>,
) -> Result<Json<()>, ApiError> {
    crate::rpc::accounts::update_account_proxy(
        axum::extract::State(state),
        account_id,
        body.proxy_host,
        body.proxy_port,
    ).await?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct UpdateProxySettingBody {
    pub mode: String,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<u16>,
}

async fn get_proxy_setting_handler(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
) -> Result<Json<crate::rpc::network::AccountProxySetting>, ApiError> {
    let setting = crate::rpc::accounts::get_account_proxy_setting(
        axum::extract::State(state),
        account_id,
    ).await?;
    Ok(Json(setting))
}

async fn update_proxy_setting_handler(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
    Json(body): Json<UpdateProxySettingBody>,
) -> Result<Json<()>, ApiError> {
    let mode: crate::rpc::network::AccountProxyMode =
        serde_json::from_str(&format!("\"{}\"", body.mode)).unwrap_or_default();
    crate::rpc::accounts::update_account_proxy_setting(
        axum::extract::State(state),
        account_id,
        mode,
        body.proxy_host,
        body.proxy_port,
    ).await?;
    Ok(Json(()))
}
