use axum::{
    body::Bytes,
    extract::{Query, State},
    http::StatusCode,
};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use pebble_core::{now_timestamp, Account, PebbleError, ProviderType};
use pebble_mail::{GmailProvider, SyncConfig};
use pebble_store::Store;
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, info, warn};

use crate::events;
use crate::realtime::{RealtimeMode, RealtimeStatusPayload};
use crate::state::AppState;

const GMAIL_PUSH_STATE_KEY: &str = "gmail_push";
const DEFAULT_FALLBACK_INTERVAL_MINUTES: u64 = 15;
const MIN_FALLBACK_INTERVAL_MINUTES: u64 = 1;
const MAX_FALLBACK_INTERVAL_MINUTES: u64 = 60;
const WATCH_RENEWAL_WINDOW_MS: i64 = 24 * 60 * 60 * 1000;
const WATCH_RENEWAL_INTERVAL: Duration = Duration::from_secs(12 * 60 * 60);
const PUSH_COALESCE_WINDOW: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct GmailPushState {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topic_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiration_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_watch_history_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_watch_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(default = "default_fallback_interval_minutes")]
    pub fallback_interval_minutes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_push_history_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_push_at: Option<i64>,
}

impl Default for GmailPushState {
    fn default() -> Self {
        Self {
            enabled: false,
            topic_name: None,
            expiration_ms: None,
            last_watch_history_id: None,
            last_watch_at: None,
            last_error: None,
            fallback_interval_minutes: DEFAULT_FALLBACK_INTERVAL_MINUTES,
            last_push_history_id: None,
            last_push_at: None,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum GmailRealtimeStatus {
    NotEnabled,
    Enabling,
    RealtimeEnabled,
    Renewing,
    RealtimeError,
    ReconnectRequired,
    ConfigMissing,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GmailRealtimeConfig {
    pub account_id: String,
    pub enabled: bool,
    pub status: GmailRealtimeStatus,
    pub config_missing: bool,
    pub topic_name: Option<String>,
    pub expiration_ms: Option<i64>,
    pub last_watch_history_id: Option<String>,
    pub last_watch_at: Option<i64>,
    pub last_error: Option<String>,
    pub fallback_interval_minutes: u64,
}

#[derive(Debug, Clone)]
struct GmailPubSubConfig {
    topic_name: String,
    _webhook_secret: String,
}

#[derive(Debug, Clone, Deserialize)]
struct PubSubPush {
    message: PubSubMessage,
}

#[derive(Debug, Clone, Deserialize)]
struct PubSubMessage {
    data: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GmailPushNotification {
    pub email_address: String,
    #[serde(deserialize_with = "deserialize_required_string_from_number_or_string")]
    pub history_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WebhookPayloadError {
    InvalidJson,
    InvalidBase64,
    InvalidData,
}

pub(crate) fn default_fallback_interval_minutes() -> u64 {
    DEFAULT_FALLBACK_INTERVAL_MINUTES
}

fn deserialize_required_string_from_number_or_string<'de, D>(
    deserializer: D,
) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::String(s) if !s.trim().is_empty() => Ok(s),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        _ => Err(de::Error::custom("expected non-empty string or number")),
    }
}

pub(crate) fn validate_fallback_interval_minutes(minutes: u64) -> Result<u64, PebbleError> {
    if (MIN_FALLBACK_INTERVAL_MINUTES..=MAX_FALLBACK_INTERVAL_MINUTES).contains(&minutes) {
        Ok(minutes)
    } else {
        Err(PebbleError::Validation(format!(
            "Gmail realtime fallback interval must be between {MIN_FALLBACK_INTERVAL_MINUTES} and {MAX_FALLBACK_INTERVAL_MINUTES} minutes"
        )))
    }
}

fn normalized_fallback_interval_minutes(minutes: u64) -> u64 {
    if (MIN_FALLBACK_INTERVAL_MINUTES..=MAX_FALLBACK_INTERVAL_MINUTES).contains(&minutes) {
        minutes
    } else {
        DEFAULT_FALLBACK_INTERVAL_MINUTES
    }
}

pub(crate) fn load_gmail_push_state(
    store: &Store,
    account_id: &str,
) -> Result<GmailPushState, PebbleError> {
    let Some(sync_state) = store.get_sync_state(account_id)? else {
        return Ok(GmailPushState::default());
    };
    let Some(value) = sync_state.extra.get(GMAIL_PUSH_STATE_KEY) else {
        return Ok(GmailPushState::default());
    };

    serde_json::from_value(value.clone())
        .map_err(|e| PebbleError::Storage(format!("Invalid gmail_push sync state: {e}")))
}

fn save_gmail_push_state(
    store: &Store,
    account_id: &str,
    push_state: &GmailPushState,
) -> Result<(), PebbleError> {
    let value = serde_json::to_value(push_state)
        .map_err(|e| PebbleError::Storage(format!("Failed to serialize gmail_push state: {e}")))?;
    store.update_sync_state(account_id, |sync_state| {
        sync_state
            .extra
            .insert(GMAIL_PUSH_STATE_KEY.to_string(), value);
    })
}

fn mutate_gmail_push_state<F>(
    store: &Store,
    account_id: &str,
    mutate: F,
) -> Result<GmailPushState, PebbleError>
where
    F: FnOnce(&mut GmailPushState),
{
    let mut push_state = load_gmail_push_state(store, account_id)?;
    mutate(&mut push_state);
    save_gmail_push_state(store, account_id, &push_state)?;
    Ok(push_state)
}

pub(crate) fn effective_gmail_poll_interval_secs(
    provider: &ProviderType,
    push_state: Option<&GmailPushState>,
    requested_interval_secs: Option<u64>,
) -> Option<u64> {
    if !matches!(provider, ProviderType::Gmail) {
        return requested_interval_secs;
    }

    let Some(push_state) = push_state else {
        return requested_interval_secs;
    };
    if !push_state.enabled {
        return requested_interval_secs;
    }
    if matches!(requested_interval_secs, Some(0)) {
        return Some(0);
    }

    Some(normalized_fallback_interval_minutes(push_state.fallback_interval_minutes) * 60)
}

pub(crate) fn gmail_push_status_message(config: &SyncConfig, push_enabled: bool) -> String {
    if config.manual_only() {
        return "Manual only".to_string();
    }
    if push_enabled {
        let minutes = (config.poll_interval_secs / 60).max(1);
        return format!("Realtime enabled; fallback every {minutes}m");
    }
    format!("Polling every {}s", config.poll_interval_secs)
}

pub(crate) fn gmail_push_initial_mode(config: &SyncConfig, push_enabled: bool) -> RealtimeMode {
    if config.manual_only() {
        RealtimeMode::Manual
    } else if push_enabled {
        RealtimeMode::Realtime
    } else {
        RealtimeMode::Polling
    }
}

fn env_value(key: &str) -> Option<String> {
    crate::rpc::oauth::runtime_config_value(key).filter(|value| !value.trim().is_empty())
}

fn gmail_pubsub_config() -> Result<GmailPubSubConfig, PebbleError> {
    let topic_name = env_value("GMAIL_PUBSUB_TOPIC").ok_or_else(|| {
        PebbleError::Validation("GMAIL_PUBSUB_TOPIC is not configured".to_string())
    })?;
    let webhook_secret = env_value("GMAIL_WEBHOOK_SECRET").ok_or_else(|| {
        PebbleError::Validation("GMAIL_WEBHOOK_SECRET is not configured".to_string())
    })?;
    Ok(GmailPubSubConfig {
        topic_name,
        _webhook_secret: webhook_secret,
    })
}

fn gmail_pubsub_config_missing() -> bool {
    env_value("GMAIL_PUBSUB_TOPIC").is_none() || env_value("GMAIL_WEBHOOK_SECRET").is_none()
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }

    let mut diff = 0u8;
    for (a, b) in left.as_bytes().iter().zip(right.as_bytes()) {
        diff |= a ^ b;
    }
    diff == 0
}

fn is_auth_error(error: &PebbleError) -> bool {
    matches!(
        error,
        PebbleError::Auth(_)
            | PebbleError::OAuth(_)
            | PebbleError::TokenExpired(_)
            | PebbleError::TokenRefreshFailed(_)
    ) || error.to_string().to_ascii_lowercase().contains("401")
        || error
            .to_string()
            .to_ascii_lowercase()
            .contains("unauthorized")
}

fn status_from_state(push_state: &GmailPushState, config_missing: bool) -> GmailRealtimeStatus {
    if config_missing {
        return GmailRealtimeStatus::ConfigMissing;
    }
    if !push_state.enabled {
        return GmailRealtimeStatus::NotEnabled;
    }
    if let Some(error) = &push_state.last_error {
        let lower = error.to_ascii_lowercase();
        if lower.contains("auth")
            || lower.contains("oauth")
            || lower.contains("token")
            || lower.contains("401")
            || lower.contains("unauthorized")
        {
            return GmailRealtimeStatus::ReconnectRequired;
        }
        return GmailRealtimeStatus::RealtimeError;
    }
    GmailRealtimeStatus::RealtimeEnabled
}

fn config_from_push_state(
    account_id: &str,
    push_state: GmailPushState,
    config_missing: bool,
) -> GmailRealtimeConfig {
    GmailRealtimeConfig {
        account_id: account_id.to_string(),
        enabled: push_state.enabled,
        status: status_from_state(&push_state, config_missing),
        config_missing,
        topic_name: push_state.topic_name,
        expiration_ms: push_state.expiration_ms,
        last_watch_history_id: push_state.last_watch_history_id,
        last_watch_at: push_state.last_watch_at,
        last_error: push_state.last_error,
        fallback_interval_minutes: normalized_fallback_interval_minutes(
            push_state.fallback_interval_minutes,
        ),
    }
}

fn emit_gmail_realtime_status(
    state: &AppState,
    account_id: &str,
    mode: RealtimeMode,
    message: Option<String>,
) {
    state.emit(
        events::MAIL_REALTIME_STATUS,
        RealtimeStatusPayload {
            account_id: account_id.to_string(),
            mode,
            provider: "gmail".to_string(),
            last_success_at: Some(now_timestamp()),
            next_retry_at: None,
            message,
        },
    );
}

fn get_gmail_account(state: &AppState, account_id: &str) -> Result<Account, PebbleError> {
    let account = state
        .store
        .get_account(account_id)?
        .ok_or_else(|| PebbleError::Internal(format!("Account not found: {account_id}")))?;
    if account.provider != ProviderType::Gmail {
        return Err(PebbleError::UnsupportedProvider(
            "Gmail realtime can only be enabled for Gmail accounts".to_string(),
        ));
    }
    Ok(account)
}

pub(crate) fn get_gmail_realtime_config_raw(
    state: &AppState,
    account_id: &str,
) -> Result<GmailRealtimeConfig, PebbleError> {
    get_gmail_account(state, account_id)?;
    let push_state = load_gmail_push_state(&state.store, account_id)?;
    Ok(config_from_push_state(
        account_id,
        push_state,
        gmail_pubsub_config_missing(),
    ))
}

async fn gmail_provider_for_account(
    state: &AppState,
    account_id: &str,
) -> Result<GmailProvider, PebbleError> {
    let auth = crate::rpc::oauth::ensure_account_oauth_auth(state, account_id, "gmail").await?;
    GmailProvider::new_with_proxy(auth.tokens.access_token, auth.proxy)
}

async fn refreshed_gmail_provider_for_account(
    state: &AppState,
    account_id: &str,
) -> Result<GmailProvider, PebbleError> {
    let auth = crate::rpc::oauth::refresh_account_oauth_auth(state, account_id, "gmail").await?;
    GmailProvider::new_with_proxy(auth.tokens.access_token, auth.proxy)
}

async fn watch_gmail_with_retry(
    state: &AppState,
    account_id: &str,
    topic_name: &str,
) -> Result<pebble_mail::provider::gmail::GmailWatchResponse, PebbleError> {
    let provider = gmail_provider_for_account(state, account_id).await?;
    match provider.watch_inbox(topic_name).await {
        Ok(response) => Ok(response),
        Err(error) if is_auth_error(&error) => {
            let provider = refreshed_gmail_provider_for_account(state, account_id).await?;
            provider.watch_inbox(topic_name).await
        }
        Err(error) => Err(error),
    }
}

async fn stop_gmail_watch_with_retry(
    state: &AppState,
    account_id: &str,
) -> Result<(), PebbleError> {
    let provider = gmail_provider_for_account(state, account_id).await?;
    match provider.stop_watch().await {
        Ok(()) => Ok(()),
        Err(error) if is_auth_error(&error) => {
            let provider = refreshed_gmail_provider_for_account(state, account_id).await?;
            provider.stop_watch().await
        }
        Err(error) => Err(error),
    }
}

async fn sync_handle_running(state: &AppState, account_id: &str) -> bool {
    let handles = state.sync_handles.lock().await;
    handles
        .get(account_id)
        .is_some_and(|handle| !handle.task.is_finished())
}

async fn restart_running_sync_with_interval(
    state: Arc<AppState>,
    account_id: &str,
    poll_interval_secs: Option<u64>,
) -> Result<(), PebbleError> {
    if !sync_handle_running(&state, account_id).await {
        return Ok(());
    }

    crate::rpc::sync_cmd::stop_sync(State(state.clone()), account_id.to_string()).await?;
    crate::rpc::sync_cmd::start_sync(State(state), account_id.to_string(), poll_interval_secs)
        .await?;
    Ok(())
}

async fn ensure_sync_running_with_interval(
    state: Arc<AppState>,
    account_id: &str,
    poll_interval_secs: u64,
) -> Result<(), PebbleError> {
    if sync_handle_running(&state, account_id).await {
        crate::rpc::sync_cmd::stop_sync(State(state.clone()), account_id.to_string()).await?;
    }

    crate::rpc::sync_cmd::start_sync(
        State(state),
        account_id.to_string(),
        Some(poll_interval_secs),
    )
    .await?;
    Ok(())
}

async fn trigger_provider_push_sync(
    state: Arc<AppState>,
    account_id: &str,
) -> Result<(), PebbleError> {
    crate::rpc::sync_cmd::trigger_sync(
        State(state),
        account_id.to_string(),
        "provider_push".to_string(),
    )
    .await
}

pub(crate) async fn enable_gmail_realtime_raw(
    state: Arc<AppState>,
    account_id: String,
    fallback_interval_minutes: Option<u64>,
) -> Result<GmailRealtimeConfig, PebbleError> {
    get_gmail_account(&state, &account_id)?;
    let fallback_interval_minutes = validate_fallback_interval_minutes(
        fallback_interval_minutes.unwrap_or(DEFAULT_FALLBACK_INTERVAL_MINUTES),
    )?;
    let config = gmail_pubsub_config()?;
    let watch = watch_gmail_with_retry(&state, &account_id, &config.topic_name).await?;

    mutate_gmail_push_state(&state.store, &account_id, |push_state| {
        push_state.enabled = true;
        push_state.topic_name = Some(config.topic_name.clone());
        push_state.expiration_ms = watch.expiration_ms;
        push_state.last_watch_history_id = Some(watch.history_id.clone());
        push_state.last_watch_at = Some(now_timestamp());
        push_state.last_error = None;
        push_state.fallback_interval_minutes = fallback_interval_minutes;
    })?;

    emit_gmail_realtime_status(
        &state,
        &account_id,
        RealtimeMode::Realtime,
        Some(format!(
            "Realtime enabled; fallback every {fallback_interval_minutes}m"
        )),
    );

    let fallback_secs = fallback_interval_minutes * 60;
    if let Err(error) =
        ensure_sync_running_with_interval(state.clone(), &account_id, fallback_secs).await
    {
        warn!("Failed to start Gmail sync after enabling realtime for {account_id}: {error}");
        let _ = record_gmail_realtime_error(&state.store, &account_id, error.to_string());
    }

    if let Err(error) = trigger_provider_push_sync(state.clone(), &account_id).await {
        warn!("Failed to trigger Gmail sync after enabling realtime for {account_id}: {error}");
        let _ = record_gmail_realtime_error(&state.store, &account_id, error.to_string());
    }

    get_gmail_realtime_config_raw(&state, &account_id)
}

pub(crate) async fn disable_gmail_realtime_raw(
    state: Arc<AppState>,
    account_id: String,
) -> Result<GmailRealtimeConfig, PebbleError> {
    get_gmail_account(&state, &account_id)?;

    let stop_error = match stop_gmail_watch_with_retry(&state, &account_id).await {
        Ok(()) => None,
        Err(error) => {
            warn!("Failed to stop Gmail watch for {account_id}: {error}");
            Some(error.to_string())
        }
    };

    mutate_gmail_push_state(&state.store, &account_id, |push_state| {
        push_state.enabled = false;
        push_state.last_error = stop_error.clone();
    })?;

    emit_gmail_realtime_status(
        &state,
        &account_id,
        RealtimeMode::Polling,
        Some("Gmail realtime disabled".to_string()),
    );

    if let Err(error) = restart_running_sync_with_interval(state.clone(), &account_id, None).await {
        warn!("Failed to restart Gmail sync after disabling realtime for {account_id}: {error}");
        let _ = record_gmail_realtime_error(&state.store, &account_id, error.to_string());
    }

    get_gmail_realtime_config_raw(&state, &account_id)
}

pub(crate) async fn update_gmail_realtime_config_raw(
    state: Arc<AppState>,
    account_id: String,
    fallback_interval_minutes: u64,
) -> Result<GmailRealtimeConfig, PebbleError> {
    get_gmail_account(&state, &account_id)?;
    let fallback_interval_minutes = validate_fallback_interval_minutes(fallback_interval_minutes)?;
    let push_state = mutate_gmail_push_state(&state.store, &account_id, |push_state| {
        push_state.fallback_interval_minutes = fallback_interval_minutes;
        if !push_state.enabled {
            push_state.last_error = None;
        }
    })?;

    if push_state.enabled {
        let fallback_secs = fallback_interval_minutes * 60;
        if let Err(error) =
            ensure_sync_running_with_interval(state.clone(), &account_id, fallback_secs).await
        {
            warn!(
                "Failed to start Gmail sync after updating realtime config for {account_id}: {error}"
            );
            let _ = record_gmail_realtime_error(&state.store, &account_id, error.to_string());
        }
    }

    get_gmail_realtime_config_raw(&state, &account_id)
}

fn record_gmail_realtime_error(
    store: &Store,
    account_id: &str,
    error: String,
) -> Result<(), PebbleError> {
    mutate_gmail_push_state(store, account_id, |push_state| {
        push_state.last_error = Some(error);
    })?;
    Ok(())
}

fn record_gmail_push_delivery(
    store: &Store,
    account_id: &str,
    history_id: &str,
) -> Result<(), PebbleError> {
    mutate_gmail_push_state(store, account_id, |push_state| {
        push_state.last_push_history_id = Some(history_id.to_string());
        push_state.last_push_at = Some(now_timestamp());
    })?;
    Ok(())
}

pub(crate) fn should_renew_watch(push_state: &GmailPushState, now_ms: i64) -> bool {
    push_state.enabled
        && push_state
            .expiration_ms
            .map(|expiration| expiration - now_ms < WATCH_RENEWAL_WINDOW_MS)
            .unwrap_or(true)
}

async fn renew_gmail_watch_for_account(
    state: Arc<AppState>,
    account_id: &str,
    topic_name: &str,
) -> Result<(), PebbleError> {
    emit_gmail_realtime_status(
        &state,
        account_id,
        RealtimeMode::Realtime,
        Some("Renewing Gmail realtime".to_string()),
    );

    let watch = watch_gmail_with_retry(&state, account_id, topic_name).await?;
    mutate_gmail_push_state(&state.store, account_id, |push_state| {
        push_state.enabled = true;
        push_state.topic_name = Some(topic_name.to_string());
        push_state.expiration_ms = watch.expiration_ms;
        push_state.last_watch_history_id = Some(watch.history_id.clone());
        push_state.last_watch_at = Some(now_timestamp());
        push_state.last_error = None;
    })?;

    emit_gmail_realtime_status(
        &state,
        account_id,
        RealtimeMode::Realtime,
        Some("Realtime enabled".to_string()),
    );
    Ok(())
}

pub(crate) async fn run_gmail_watch_renewal_pass(state: Arc<AppState>) {
    let config = match gmail_pubsub_config() {
        Ok(config) => config,
        Err(error) => {
            let accounts = match state.store.list_accounts() {
                Ok(accounts) => accounts,
                Err(list_error) => {
                    warn!("Failed to list Gmail accounts for watch renewal: {list_error}");
                    return;
                }
            };
            for account in accounts
                .into_iter()
                .filter(|account| account.provider == ProviderType::Gmail)
            {
                if load_gmail_push_state(&state.store, &account.id)
                    .map(|push_state| push_state.enabled)
                    .unwrap_or(false)
                {
                    let _ =
                        record_gmail_realtime_error(&state.store, &account.id, error.to_string());
                    emit_gmail_realtime_status(
                        &state,
                        &account.id,
                        RealtimeMode::Error,
                        Some("Gmail realtime config missing".to_string()),
                    );
                }
            }
            return;
        }
    };

    let accounts = match state.store.list_accounts() {
        Ok(accounts) => accounts,
        Err(error) => {
            warn!("Failed to list Gmail accounts for watch renewal: {error}");
            return;
        }
    };
    let now_ms = now_timestamp().saturating_mul(1000);

    for account in accounts
        .into_iter()
        .filter(|account| account.provider == ProviderType::Gmail)
    {
        let push_state = match load_gmail_push_state(&state.store, &account.id) {
            Ok(push_state) => push_state,
            Err(error) => {
                warn!(
                    "Failed to load Gmail realtime state for account {}: {}",
                    account.id, error
                );
                continue;
            }
        };
        if !should_renew_watch(&push_state, now_ms) {
            continue;
        }

        match renew_gmail_watch_for_account(state.clone(), &account.id, &config.topic_name).await {
            Ok(()) => info!("Renewed Gmail watch for account {}", account.id),
            Err(error) => {
                warn!(
                    "Failed to renew Gmail watch for account {}: {}",
                    account.id, error
                );
                let _ = record_gmail_realtime_error(&state.store, &account.id, error.to_string());
                emit_gmail_realtime_status(
                    &state,
                    &account.id,
                    if is_auth_error(&error) {
                        RealtimeMode::AuthRequired
                    } else {
                        RealtimeMode::Error
                    },
                    Some(error.to_string()),
                );
            }
        }
    }
}

pub(crate) fn spawn_gmail_watch_renewal_task(state: Arc<AppState>) {
    tokio::spawn(async move {
        loop {
            run_gmail_watch_renewal_pass(state.clone()).await;
            sleep(WATCH_RENEWAL_INTERVAL).await;
        }
    });
}

pub(crate) fn decode_gmail_pubsub_payload(
    body: &[u8],
) -> Result<GmailPushNotification, WebhookPayloadError> {
    let push: PubSubPush =
        serde_json::from_slice(body).map_err(|_| WebhookPayloadError::InvalidJson)?;
    let decoded = BASE64_STANDARD
        .decode(push.message.data.as_bytes())
        .map_err(|_| WebhookPayloadError::InvalidBase64)?;
    let notification: GmailPushNotification =
        serde_json::from_slice(&decoded).map_err(|_| WebhookPayloadError::InvalidData)?;

    if notification.email_address.trim().is_empty() || notification.history_id.trim().is_empty() {
        return Err(WebhookPayloadError::InvalidData);
    }

    Ok(notification)
}

async fn claim_push_trigger(state: &AppState, account_id: &str, now: Instant) -> bool {
    let mut coalescer = state.gmail_push_coalescer.lock().await;
    if coalescer
        .get(account_id)
        .is_some_and(|last| now.duration_since(*last) < PUSH_COALESCE_WINDOW)
    {
        return false;
    }
    coalescer.insert(account_id.to_string(), now);
    true
}

async fn process_gmail_push_notification(
    state: Arc<AppState>,
    notification: GmailPushNotification,
) {
    let accounts = match state.store.list_accounts() {
        Ok(accounts) => accounts,
        Err(error) => {
            warn!("Failed to list accounts for Gmail push notification: {error}");
            return;
        }
    };

    let mut matched_count = 0usize;
    let pushed_email = notification.email_address.trim();
    for account in accounts.into_iter().filter(|account| {
        account.provider == ProviderType::Gmail
            && account.email.trim().eq_ignore_ascii_case(pushed_email)
    }) {
        let push_state = match load_gmail_push_state(&state.store, &account.id) {
            Ok(push_state) => push_state,
            Err(error) => {
                warn!(
                    "Failed to load Gmail realtime state for pushed account {}: {}",
                    account.id, error
                );
                continue;
            }
        };
        if !push_state.enabled {
            continue;
        }
        matched_count += 1;

        if let Err(error) =
            record_gmail_push_delivery(&state.store, &account.id, &notification.history_id)
        {
            warn!(
                "Failed to record Gmail push delivery for account {}: {}",
                account.id, error
            );
        }

        if !claim_push_trigger(&state, &account.id, Instant::now()).await {
            debug!("Coalesced Gmail push trigger for account {}", account.id);
            continue;
        }

        if let Err(error) = trigger_provider_push_sync(state.clone(), &account.id).await {
            warn!(
                "Failed to trigger sync from Gmail push for account {}: {}",
                account.id, error
            );
            let _ = record_gmail_realtime_error(&state.store, &account.id, error.to_string());
        }
    }

    if matched_count == 0 {
        info!(
            "Ignoring Gmail push for unmapped or disabled address {}",
            notification.email_address
        );
    }
}

fn webhook_secret_authorized(query: &HashMap<String, String>) -> bool {
    let Some(expected) = env_value("GMAIL_WEBHOOK_SECRET") else {
        return false;
    };
    let Some(actual) = query.get("secret") else {
        return false;
    };
    constant_time_eq(&expected, actual)
}

pub(crate) async fn gmail_webhook_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HashMap<String, String>>,
    body: Bytes,
) -> StatusCode {
    if !webhook_secret_authorized(&query) {
        return StatusCode::UNAUTHORIZED;
    }

    let notification = match decode_gmail_pubsub_payload(&body) {
        Ok(notification) => notification,
        Err(WebhookPayloadError::InvalidJson)
        | Err(WebhookPayloadError::InvalidBase64)
        | Err(WebhookPayloadError::InvalidData) => return StatusCode::BAD_REQUEST,
    };

    tokio::spawn(process_gmail_push_notification(state, notification));
    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pubsub_body(decoded: &str) -> Vec<u8> {
        let data = BASE64_STANDARD.encode(decoded.as_bytes());
        serde_json::json!({ "message": { "data": data } })
            .to_string()
            .into_bytes()
    }

    #[test]
    fn decodes_pubsub_payload_with_numeric_history_id() {
        let body = pubsub_body(r#"{"emailAddress":"user@example.com","historyId":12345}"#);
        let decoded = decode_gmail_pubsub_payload(&body).unwrap();

        assert_eq!(decoded.email_address, "user@example.com");
        assert_eq!(decoded.history_id, "12345");
    }

    #[test]
    fn rejects_pubsub_payload_with_invalid_base64() {
        let body = br#"{"message":{"data":"not base64!"}}"#;

        assert_eq!(
            decode_gmail_pubsub_payload(body).unwrap_err(),
            WebhookPayloadError::InvalidBase64
        );
    }

    #[test]
    fn rejects_empty_gmail_push_fields() {
        let body = pubsub_body(r#"{"emailAddress":"","historyId":12345}"#);

        assert_eq!(
            decode_gmail_pubsub_payload(&body).unwrap_err(),
            WebhookPayloadError::InvalidData
        );
    }

    #[test]
    fn validates_fallback_interval_range() {
        assert_eq!(validate_fallback_interval_minutes(1).unwrap(), 1);
        assert_eq!(validate_fallback_interval_minutes(60).unwrap(), 60);
        assert!(validate_fallback_interval_minutes(0).is_err());
        assert!(validate_fallback_interval_minutes(61).is_err());
    }

    #[test]
    fn push_enabled_gmail_overrides_non_manual_polling_interval() {
        let push_state = GmailPushState {
            enabled: true,
            fallback_interval_minutes: 20,
            ..Default::default()
        };

        assert_eq!(
            effective_gmail_poll_interval_secs(&ProviderType::Gmail, Some(&push_state), Some(3),),
            Some(1200)
        );
        assert_eq!(
            effective_gmail_poll_interval_secs(&ProviderType::Gmail, Some(&push_state), Some(0),),
            Some(0)
        );
        assert_eq!(
            effective_gmail_poll_interval_secs(&ProviderType::Imap, Some(&push_state), Some(3),),
            Some(3)
        );
    }

    #[test]
    fn renews_missing_or_expiring_watch_only_when_enabled() {
        let now = 1_000_000;
        let disabled = GmailPushState::default();
        assert!(!should_renew_watch(&disabled, now));

        let missing_expiration = GmailPushState {
            enabled: true,
            expiration_ms: None,
            ..Default::default()
        };
        assert!(should_renew_watch(&missing_expiration, now));

        let far_future = GmailPushState {
            enabled: true,
            expiration_ms: Some(now + WATCH_RENEWAL_WINDOW_MS + 1_000),
            ..Default::default()
        };
        assert!(!should_renew_watch(&far_future, now));
    }
}
