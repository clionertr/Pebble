use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

const MAIL_LATENCY_TARGET: &str = "pebble::mail_latency";

#[derive(Clone, Debug)]
pub struct MailLatencyHint {
    pub source: &'static str,
    pub backend_received_at_ms: i64,
    pub history_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MailLatencyPayload {
    pub source: String,
    pub backend_received_at_ms: Option<i64>,
    pub backend_sse_at_ms: i64,
    pub message_received_at_ms: Option<i64>,
    pub history_id: Option<String>,
}

pub fn debug_enabled() -> bool {
    tracing::enabled!(target: MAIL_LATENCY_TARGET, tracing::Level::DEBUG)
}

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

pub fn seconds_to_ms(seconds: i64) -> Option<i64> {
    if seconds <= 0 {
        return None;
    }
    seconds.checked_mul(1000)
}

pub fn elapsed_ms(start_ms: Option<i64>, end_ms: i64) -> Option<i64> {
    start_ms.map(|start| end_ms.saturating_sub(start))
}

pub fn log_mail_latency(
    stage: &str,
    account_id: Option<&str>,
    message_id: Option<&str>,
    source: Option<&str>,
    detail: impl FnOnce() -> String,
) {
    if !debug_enabled() {
        return;
    }

    let detail = detail();
    tracing::debug!(
        target: MAIL_LATENCY_TARGET,
        stage,
        account_id = account_id.unwrap_or(""),
        message_id = message_id.unwrap_or(""),
        source = source.unwrap_or(""),
        detail = detail.as_str(),
        "mail latency event"
    );
}
