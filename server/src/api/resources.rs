// Rules + Translate + Contacts + Cloud Sync + Trusted Senders + Templates
// + Diagnostics + Preferences + Proxy
// Each handler is a thin delegate to existing crate::rpc::* functions.

use axum::{
    extract::{Path, Query, State},
    Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use crate::state::AppState;

pub fn resource_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/rules", get(list_rules).post(create_rule))
        .route("/api/rules/{id}", put(update_rule).delete(delete_rule))
        .route("/api/translate", post(translate_text))
        .route("/api/translate/config", get(get_translate_config).put(save_translate_config))
        .route("/api/translate/test", post(test_translate))
        .route("/api/contacts", get(search_contacts_handler))
        .route("/api/cloud-sync/webdav/test", post(webdav_test))
        .route("/api/cloud-sync/webdav/backup", post(webdav_backup))
        .route("/api/cloud-sync/webdav/preview", post(webdav_preview))
        .route("/api/cloud-sync/webdav/restore", post(webdav_restore))
        .route("/api/trusted-senders", get(list_trusted).post(trust_sender_handler))
        .route("/api/trusted-senders/check", get(check_trusted))
        .route("/api/templates", get(list_templates).post(save_template))
        .route("/api/templates/{id}", delete(delete_template))
        .route("/api/preferences/realtime", put(set_realtime))
        .route("/api/preferences/notifications", put(set_notifications))
        .route("/api/logs", get(read_logs))
        .route("/api/diagnostics/mail-timing", post(record_timing))
        .route("/api/proxy", get(get_global_proxy_handler).put(update_global_proxy_handler))
}

// ── Rules ────────────────────────────────────────────────────────────

async fn list_rules(State(state): State<Arc<AppState>>) -> Result<Json<Vec<pebble_core::Rule>>, crate::api::error::ApiError> {
    Ok(Json(crate::rpc::rules::list_rules(axum::extract::State(state)).await?))
}

#[derive(Deserialize)] pub struct CreateRuleRequest { pub name: String, pub priority: i32, pub conditions: String, pub actions: String }

async fn create_rule(State(state): State<Arc<AppState>>, Json(b): Json<CreateRuleRequest>) -> Result<Json<pebble_core::Rule>, crate::api::error::ApiError> {
    Ok(Json(crate::rpc::rules::create_rule(axum::extract::State(state), b.name, b.priority, b.conditions, b.actions).await?))
}

async fn update_rule(State(state): State<Arc<AppState>>, Path(_id): Path<String>, Json(rule): Json<pebble_core::Rule>) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::rules::update_rule(axum::extract::State(state), rule).await?;
    Ok(Json(()))
}

async fn delete_rule(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::rules::delete_rule(axum::extract::State(state), id).await?;
    Ok(Json(()))
}

// ── Translate ─────────────────────────────────────────────────────────

#[derive(Deserialize)] pub struct TranslateRequest { pub text: String, #[serde(rename = "fromLang")] pub from: String, #[serde(rename = "toLang")] pub to: String }

async fn translate_text(State(state): State<Arc<AppState>>, Json(b): Json<TranslateRequest>) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let result = crate::rpc::translate::translate_text(axum::extract::State(state), b.text, b.from, b.to).await?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}

async fn get_translate_config(State(state): State<Arc<AppState>>) -> Result<Json<Option<pebble_core::TranslateConfig>>, crate::api::error::ApiError> {
    Ok(Json(crate::rpc::translate::get_translate_config(axum::extract::State(state)).await?))
}

#[derive(Deserialize)] pub struct SaveTranslateConfigRequest { #[serde(rename = "providerType")] pub provider_type: String, pub config: String, #[serde(rename = "isEnabled")] pub enabled: bool }

async fn save_translate_config(State(state): State<Arc<AppState>>, Json(b): Json<SaveTranslateConfigRequest>) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::translate::save_translate_config(axum::extract::State(state), b.provider_type, b.config, b.enabled).await?;
    Ok(Json(()))
}

#[derive(Deserialize)] pub struct TestTranslateRequest { pub config: String }

async fn test_translate(State(state): State<Arc<AppState>>, Json(b): Json<TestTranslateRequest>) -> Result<Json<String>, crate::api::error::ApiError> {
    Ok(Json(crate::rpc::translate::test_translate_connection(axum::extract::State(state), b.config).await?))
}

// ── Contacts ──────────────────────────────────────────────────────────

#[derive(Deserialize)] pub struct ContactsQuery { #[serde(rename = "accountId")] pub account_id: String, pub q: String, pub limit: Option<usize> }

async fn search_contacts_handler(State(state): State<Arc<AppState>>, Query(q): Query<ContactsQuery>) -> Result<Json<Vec<pebble_core::KnownContact>>, crate::api::error::ApiError> {
    Ok(Json(crate::rpc::contacts::search_contacts(axum::extract::State(state), q.account_id, q.q, q.limit.map(|l| l as i64)).await?))
}

// ── Cloud Sync ──────────────────────────────────────────────────────

#[derive(Deserialize)] pub struct WebdavRequest { pub url: String, pub username: String, pub password: String }

async fn webdav_test(Json(b): Json<WebdavRequest>) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let result = crate::rpc::cloud_sync::test_webdav_connection(b.url, b.username, b.password).await?;
    Ok(Json(serde_json::json!({ "status": result })))
}
async fn webdav_backup(State(state): State<Arc<AppState>>, Json(b): Json<WebdavRequest>) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let result = crate::rpc::cloud_sync::backup_to_webdav(axum::extract::State(state), b.url, b.username, b.password).await?;
    Ok(Json(serde_json::json!({ "status": result })))
}
async fn webdav_preview(Json(b): Json<WebdavRequest>) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let result = crate::rpc::cloud_sync::preview_webdav_backup(b.url, b.username, b.password).await?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}
async fn webdav_restore(State(state): State<Arc<AppState>>, Json(b): Json<WebdavRequest>) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let result = crate::rpc::cloud_sync::restore_from_webdav(axum::extract::State(state), b.url, b.username, b.password).await?;
    Ok(Json(serde_json::json!({ "status": result })))
}

// ── Trusted Senders ───────────────────────────────────────────────────

#[derive(Deserialize)] pub struct TrustedQuery { #[serde(rename = "accountId")] pub account_id: Option<String> }

async fn list_trusted(State(state): State<Arc<AppState>>, Query(q): Query<TrustedQuery>) -> Result<Json<Vec<pebble_core::TrustedSender>>, crate::api::error::ApiError> {
    let account_id = q.account_id.unwrap_or_default();
    Ok(Json(crate::rpc::trusted_senders::list_trusted_senders(axum::extract::State(state), account_id).await?))
}

#[derive(Deserialize)] pub struct TrustSenderRequest { #[serde(rename = "accountId")] pub account_id: String, pub email: String, #[serde(rename = "trustType")] pub trust_type: String }

async fn trust_sender_handler(State(state): State<Arc<AppState>>, Json(b): Json<TrustSenderRequest>) -> Result<Json<()>, crate::api::error::ApiError> {
    let trust_type = match b.trust_type.as_str() {
        "all" => pebble_core::TrustType::All,
        _ => pebble_core::TrustType::Images,
    };
    crate::rpc::trusted_senders::trust_sender(axum::extract::State(state), b.account_id, b.email, trust_type).await?;
    Ok(Json(()))
}

#[derive(Deserialize)] pub struct CheckTrustedQuery { #[serde(rename = "accountId")] pub account_id: String, pub email: String }

async fn check_trusted(State(state): State<Arc<AppState>>, Query(q): Query<CheckTrustedQuery>) -> Result<Json<bool>, crate::api::error::ApiError> {
    Ok(Json(crate::rpc::messages::rendering::is_trusted_sender(axum::extract::State(state), q.account_id, q.email).await?))
}

// ── Email Templates ──────────────────────────────────────────────────

async fn list_templates(State(state): State<Arc<AppState>>) -> Result<Json<Vec<crate::rpc::user_data::EmailTemplate>>, crate::api::error::ApiError> {
    Ok(Json(crate::rpc::user_data::list_email_templates(axum::extract::State(state)).await?))
}

async fn save_template(State(state): State<Arc<AppState>>, Json(template): Json<crate::rpc::user_data::SaveEmailTemplateRequest>) -> Result<Json<crate::rpc::user_data::EmailTemplate>, crate::api::error::ApiError> {
    Ok(Json(crate::rpc::user_data::save_email_template(axum::extract::State(state), template).await?))
}

async fn delete_template(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::user_data::delete_email_template(axum::extract::State(state), id).await?;
    Ok(Json(()))
}

// ── Preferences ───────────────────────────────────────────────────────

#[derive(Deserialize)] pub struct RealtimePref { pub mode: String }

async fn set_realtime(State(state): State<Arc<AppState>>, Json(b): Json<RealtimePref>) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::sync_cmd::set_realtime_preference(axum::extract::State(state), b.mode).await?;
    Ok(Json(()))
}

#[derive(Deserialize)] pub struct NotificationsPref { pub enabled: bool }

async fn set_notifications(State(state): State<Arc<AppState>>, Json(b): Json<NotificationsPref>) -> Result<Json<()>, crate::api::error::ApiError> {
    crate::rpc::notifications::set_notifications_enabled(axum::extract::State(state), b.enabled).await?;
    Ok(Json(()))
}

// ── Diagnostics ──────────────────────────────────────────────────────

#[derive(Deserialize)] pub struct LogsQuery { #[serde(rename = "maxBytes")] pub max_bytes: Option<u64> }

async fn read_logs(Query(q): Query<LogsQuery>) -> Result<Json<serde_json::Value>, crate::api::error::ApiError> {
    let snapshot = crate::rpc::diagnostics::read_app_log(q.max_bytes)
        .map_err(|e| crate::api::error::ApiError::internal(e))?;
    Ok(Json(serde_json::to_value(snapshot).unwrap()))
}

async fn record_timing(Json(timing): Json<serde_json::Value>) -> Result<Json<()>, crate::api::error::ApiError> {
    let timing: crate::rpc::diagnostics::MailDisplayTiming = serde_json::from_value(timing)
        .map_err(|e| crate::api::error::ApiError::bad_request(e.to_string()))?;
    crate::rpc::diagnostics::record_mail_display_timing(timing)
        .map_err(|e| crate::api::error::ApiError::internal(e))?;
    Ok(Json(()))
}

// ── Proxy ──────────────────────────────────────────────────────────────

async fn get_global_proxy_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Option<pebble_core::HttpProxyConfig>>, crate::api::error::ApiError> {
    let proxy = crate::rpc::network::get_global_proxy_raw(&state.crypto, &state.store)
        .map_err(|e| crate::api::error::ApiError::internal(e.to_string()))?;
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
    ).await?;
    Ok(Json(()))
}
