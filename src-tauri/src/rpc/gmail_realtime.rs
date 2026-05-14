use axum::extract::State;
use pebble_core::PebbleError;
use std::sync::Arc;

use crate::gmail_realtime::GmailRealtimeConfig;
use crate::state::AppState;

pub async fn get_gmail_realtime_config(
    state: State<Arc<AppState>>,
    account_id: String,
) -> std::result::Result<GmailRealtimeConfig, PebbleError> {
    crate::gmail_realtime::get_gmail_realtime_config_raw(&state, &account_id)
}

pub async fn enable_gmail_realtime(
    state: State<Arc<AppState>>,
    account_id: String,
    fallback_interval_minutes: Option<u64>,
) -> std::result::Result<GmailRealtimeConfig, PebbleError> {
    crate::gmail_realtime::enable_gmail_realtime_raw(state.0, account_id, fallback_interval_minutes)
        .await
}

pub async fn disable_gmail_realtime(
    state: State<Arc<AppState>>,
    account_id: String,
) -> std::result::Result<GmailRealtimeConfig, PebbleError> {
    crate::gmail_realtime::disable_gmail_realtime_raw(state.0, account_id).await
}

pub async fn update_gmail_realtime_config(
    state: State<Arc<AppState>>,
    account_id: String,
    fallback_interval_minutes: u64,
) -> std::result::Result<GmailRealtimeConfig, PebbleError> {
    crate::gmail_realtime::update_gmail_realtime_config_raw(
        state.0,
        account_id,
        fallback_interval_minutes,
    )
    .await
}
