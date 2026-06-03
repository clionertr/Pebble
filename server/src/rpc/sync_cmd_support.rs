use crate::realtime::{RealtimeMode, RealtimeStatusPayload, SyncTrigger};
use pebble_core::{PebbleError, ProviderType};
use pebble_mail::{SyncConfig, SyncError};
use pebble_store::Store;
use serde::Serialize;
use std::collections::HashSet;

pub(crate) fn provider_slug(provider: &ProviderType) -> &'static str {
    match provider {
        ProviderType::Imap => "imap",
        ProviderType::Gmail => "gmail",
        ProviderType::Outlook => "outlook",
    }
}

pub(crate) fn realtime_status_payload(
    account_id: &str,
    provider: &ProviderType,
    mode: RealtimeMode,
    last_success_at: Option<i64>,
    next_retry_at: Option<i64>,
    message: Option<String>,
) -> RealtimeStatusPayload {
    RealtimeStatusPayload {
        account_id: account_id.to_string(),
        mode,
        provider: provider_slug(provider).to_string(),
        last_success_at,
        next_retry_at,
        message,
    }
}

pub(crate) fn manual_realtime_status_payload(
    account_id: &str,
    provider: &ProviderType,
) -> RealtimeStatusPayload {
    realtime_status_payload(
        account_id,
        provider,
        RealtimeMode::Manual,
        None,
        None,
        Some("Manual only".to_string()),
    )
}

pub(crate) fn realtime_error_mode(sync_error: &SyncError) -> RealtimeMode {
    let text = format!(
        "{} {}",
        sync_error.error_type.to_ascii_lowercase(),
        sync_error.message.to_ascii_lowercase()
    );
    if text.contains("auth")
        || text.contains("token")
        || text.contains("unauthorized")
        || text.contains("401")
    {
        RealtimeMode::AuthRequired
    } else if text.contains("offline") || text.contains("network") {
        RealtimeMode::Offline
    } else if text.contains("circuit") || text.contains("backoff") {
        RealtimeMode::Backoff
    } else {
        RealtimeMode::Error
    }
}

pub(crate) fn polling_status_message(config: &SyncConfig) -> String {
    if config.manual_only() {
        "Manual only".to_string()
    } else {
        format!("Polling every {}s", config.poll_interval_secs)
    }
}

pub(crate) fn realtime_preference_poll_interval(
    mode: &str,
) -> std::result::Result<u64, PebbleError> {
    match mode {
        "realtime" => Ok(3),
        "balanced" => Ok(15),
        "battery" => Ok(60),
        "manual" => Ok(0),
        other => Err(PebbleError::Validation(format!(
            "Invalid realtime preference: {other}"
        ))),
    }
}

pub(crate) fn imap_initial_realtime_mode(config: &SyncConfig) -> RealtimeMode {
    if config.manual_only() {
        RealtimeMode::Manual
    } else {
        RealtimeMode::Polling
    }
}

pub(crate) fn imap_capability_realtime_mode(
    config: &SyncConfig,
    supports_idle: bool,
) -> RealtimeMode {
    if config.manual_only() {
        RealtimeMode::Manual
    } else if supports_idle {
        RealtimeMode::Realtime
    } else {
        RealtimeMode::Polling
    }
}

#[derive(Debug, Default)]
pub(crate) struct RealtimePreferenceStartSummary {
    pub(crate) started_count: usize,
    pub(crate) failures: Vec<(String, String)>,
}

impl RealtimePreferenceStartSummary {
    pub(crate) fn record_start_result(
        &mut self,
        account_id: &str,
        result: std::result::Result<SyncStartOutcome, PebbleError>,
    ) {
        match result {
            Ok(SyncStartOutcome::Started) => self.started_count += 1,
            Ok(SyncStartOutcome::AlreadyRunning) => {}
            Err(e) => self.failures.push((account_id.to_string(), e.to_string())),
        }
    }

    pub(crate) fn into_command_result(self) -> std::result::Result<(), PebbleError> {
        if self.failures.is_empty() {
            return Ok(());
        }

        let failures = self
            .failures
            .iter()
            .map(|(account_id, error)| format!("{account_id}: {error}"))
            .collect::<Vec<_>>()
            .join("; ");
        Err(PebbleError::Internal(format!(
            "Realtime preference applied with {} account start failure(s); {} account(s) started; failures: {}",
            self.failures.len(),
            self.started_count,
            failures
        )))
    }
}

#[derive(Debug, Clone)]
pub struct SyncWakeRequest {
    pub account_ids: Option<Vec<String>>,
    pub reason: String,
    pub ensure_running: bool,
    pub poll_interval_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncWakeFailure {
    pub account_id: String,
    pub error: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SyncWakeResult {
    pub account_count: usize,
    pub ensured_count: usize,
    pub triggered_count: usize,
    pub one_shot_count: usize,
    pub skipped_count: usize,
    pub failures: Vec<SyncWakeFailure>,
}

impl SyncWakeResult {
    pub(crate) fn new(account_count: usize) -> Self {
        Self {
            account_count,
            ..Self::default()
        }
    }

    pub(crate) fn record_dispatch(&mut self, outcome: TriggerDispatchOutcome) {
        match outcome {
            TriggerDispatchOutcome::Sent => {
                self.triggered_count += 1;
            }
            TriggerDispatchOutcome::StartedOneShot => {
                self.triggered_count += 1;
                self.one_shot_count += 1;
            }
            TriggerDispatchOutcome::SkippedNoWorker => {
                self.skipped_count += 1;
            }
        }
    }

    pub(crate) fn record_failure(&mut self, account_id: &str, error: &PebbleError) {
        self.failures.push(SyncWakeFailure {
            account_id: account_id.to_string(),
            error: error.to_string(),
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TriggerDispatchOutcome {
    Sent,
    StartedOneShot,
    SkippedNoWorker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SyncStartOutcome {
    Started,
    AlreadyRunning,
}

pub(crate) fn normalize_explicit_account_ids(
    account_ids: Vec<String>,
) -> std::result::Result<Vec<String>, PebbleError> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();
    for account_id in account_ids {
        let account_id = account_id.trim();
        if account_id.is_empty() {
            return Err(PebbleError::Validation(
                "account_ids cannot contain empty account IDs".to_string(),
            ));
        }
        if seen.insert(account_id.to_string()) {
            normalized.push(account_id.to_string());
        }
    }
    Ok(normalized)
}

pub(crate) fn all_account_ids(store: &Store) -> std::result::Result<Vec<String>, PebbleError> {
    Ok(store
        .list_accounts()?
        .into_iter()
        .map(|account| account.id)
        .collect())
}

pub(crate) fn should_start_missing_worker_for_wake(
    ensure_running: bool,
    trigger: SyncTrigger,
) -> bool {
    !ensure_running && trigger.should_sync_now()
}

pub(crate) fn should_start_missing_worker_for_trigger_route(trigger: SyncTrigger) -> bool {
    trigger.should_sync_now()
}

pub(crate) fn should_dispatch_wake_trigger_after_ensure(
    _trigger: SyncTrigger,
    start_outcome: SyncStartOutcome,
) -> bool {
    !matches!(start_outcome, SyncStartOutcome::Started)
}

pub(crate) fn should_send_stop_signal_to_handle(task_finished: bool) -> bool {
    !task_finished
}

pub(crate) fn should_drop_trigger_handle(task_finished: bool, send_failed: bool) -> bool {
    task_finished || send_failed
}

#[allow(dead_code)]
#[derive(Default)]
pub(crate) struct TriggerCoalescer {
    pending: HashSet<String>,
}

#[allow(dead_code)]
impl TriggerCoalescer {
    pub(crate) fn mark_pending(&mut self, account_id: &str) -> bool {
        self.pending.insert(account_id.to_string())
    }

    pub(crate) fn clear_pending(&mut self, account_id: &str) {
        self.pending.remove(account_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coalesces_duplicate_realtime_triggers_for_same_account() {
        let mut state = TriggerCoalescer::default();

        assert!(state.mark_pending("account-1"));
        assert!(!state.mark_pending("account-1"));
        state.clear_pending("account-1");
        assert!(state.mark_pending("account-1"));
    }

    #[test]
    fn realtime_status_payload_uses_provider_mode_contract() {
        let payload = realtime_status_payload(
            "account-1",
            &ProviderType::Imap,
            RealtimeMode::Realtime,
            Some(1_700_000_000),
            None,
            None,
        );

        let json = serde_json::to_value(payload).unwrap();

        assert_eq!(json["account_id"], "account-1");
        assert_eq!(json["provider"], "imap");
        assert_eq!(json["mode"], "realtime");
        assert_eq!(json["last_success_at"], 1_700_000_000);
        assert!(json["next_retry_at"].is_null());
        assert!(json["message"].is_null());
    }

    #[test]
    fn realtime_preference_maps_to_backend_poll_interval() {
        assert_eq!(realtime_preference_poll_interval("realtime").unwrap(), 3);
        assert_eq!(realtime_preference_poll_interval("balanced").unwrap(), 15);
        assert_eq!(realtime_preference_poll_interval("battery").unwrap(), 60);
        assert_eq!(realtime_preference_poll_interval("manual").unwrap(), 0);
        assert!(realtime_preference_poll_interval("turbo").is_err());
    }

    #[test]
    fn manual_preference_status_payload_reports_manual_mode() {
        let payload = manual_realtime_status_payload("account-1", &ProviderType::Gmail);

        let json = serde_json::to_value(payload).unwrap();

        assert_eq!(json["account_id"], "account-1");
        assert_eq!(json["provider"], "gmail");
        assert_eq!(json["mode"], "manual");
        assert_eq!(json["message"], "Manual only");
    }

    #[test]
    fn realtime_error_mode_classifies_common_failures() {
        let auth = SyncError {
            error_type: "Auth".to_string(),
            message: "token expired".to_string(),
            timestamp: 1,
        };
        let network = SyncError {
            error_type: "Network".to_string(),
            message: "offline".to_string(),
            timestamp: 1,
        };
        let backoff = SyncError {
            error_type: "Runtime".to_string(),
            message: "circuit breaker is open".to_string(),
            timestamp: 1,
        };

        assert_eq!(realtime_error_mode(&auth), RealtimeMode::AuthRequired);
        assert_eq!(realtime_error_mode(&network), RealtimeMode::Offline);
        assert_eq!(realtime_error_mode(&backoff), RealtimeMode::Backoff);
    }

    #[test]
    fn realtime_preference_start_summary_keeps_successes_after_account_failure() {
        let mut summary = RealtimePreferenceStartSummary::default();

        summary.record_start_result(
            "bad-account",
            Err(PebbleError::Internal("No auth data".to_string())),
        );
        summary.record_start_result("good-account", Ok(SyncStartOutcome::Started));

        assert_eq!(summary.started_count, 1);
        assert_eq!(summary.failures.len(), 1);
        let err = summary
            .into_command_result()
            .expect_err("partial realtime preference failures should be visible to the UI");
        assert!(err.to_string().contains("bad-account"));
        assert!(err.to_string().contains("1 account(s) started"));
    }

    #[test]
    fn wake_account_ids_are_deduped_and_validated() {
        let ids = normalize_explicit_account_ids(vec![
            " account-1 ".to_string(),
            "account-2".to_string(),
            "account-1".to_string(),
        ])
        .unwrap();

        assert_eq!(ids, vec!["account-1", "account-2"]);
        assert!(normalize_explicit_account_ids(vec![" ".to_string()]).is_err());
    }

    #[test]
    fn passive_wake_does_not_start_missing_worker() {
        assert!(!should_start_missing_worker_for_wake(
            false,
            SyncTrigger::WindowBlur
        ));
        assert!(should_start_missing_worker_for_wake(
            false,
            SyncTrigger::Manual
        ));
        assert!(!should_start_missing_worker_for_wake(
            true,
            SyncTrigger::Manual
        ));
    }

    #[test]
    fn wake_does_not_retrigger_worker_started_by_ensure_running() {
        assert!(!should_dispatch_wake_trigger_after_ensure(
            SyncTrigger::WindowFocus,
            SyncStartOutcome::Started,
        ));
        assert!(should_dispatch_wake_trigger_after_ensure(
            SyncTrigger::WindowFocus,
            SyncStartOutcome::AlreadyRunning,
        ));
    }

    #[test]
    fn legacy_trigger_route_does_not_start_missing_worker_for_passive_reason() {
        assert!(!should_start_missing_worker_for_trigger_route(
            SyncTrigger::WindowBlur
        ));
        assert!(!should_start_missing_worker_for_trigger_route(
            SyncTrigger::Timer
        ));
        assert!(should_start_missing_worker_for_trigger_route(
            SyncTrigger::Manual
        ));
        assert!(should_start_missing_worker_for_trigger_route(
            SyncTrigger::ProviderPush
        ));
    }

    #[test]
    fn finished_sync_handle_does_not_need_stop_signal() {
        assert!(!should_send_stop_signal_to_handle(true));
        assert!(should_send_stop_signal_to_handle(false));
    }

    #[test]
    fn trigger_handle_is_dropped_when_finished_or_channel_closed() {
        assert!(should_drop_trigger_handle(true, false));
        assert!(should_drop_trigger_handle(false, true));
        assert!(!should_drop_trigger_handle(false, false));
    }

    #[test]
    fn imap_initial_status_is_polling_until_idle_is_confirmed() {
        let config = SyncConfig {
            poll_interval_secs: 10,
            ..Default::default()
        };

        assert_eq!(imap_initial_realtime_mode(&config), RealtimeMode::Polling);
    }

    #[test]
    fn imap_capability_status_reports_realtime_only_when_idle_is_available() {
        let config = SyncConfig {
            poll_interval_secs: 10,
            ..Default::default()
        };

        assert_eq!(
            imap_capability_realtime_mode(&config, true),
            RealtimeMode::Realtime
        );
        assert_eq!(
            imap_capability_realtime_mode(&config, false),
            RealtimeMode::Polling
        );
    }
}
