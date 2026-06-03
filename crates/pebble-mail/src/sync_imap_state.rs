use crate::idle::MailboxUidState;
use pebble_core::{Folder, FolderRole, PebbleError};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub(crate) struct ImapFolderCursor {
    pub(crate) uidvalidity: Option<u64>,
    pub(crate) last_uid: Option<u32>,
    pub(crate) highest_modseq: Option<u64>,
}

pub(crate) fn parse_imap_folder_cursor(state: Option<&str>) -> ImapFolderCursor {
    match state {
        Some(raw) => serde_json::from_str(raw).unwrap_or_default(),
        None => ImapFolderCursor::default(),
    }
}

pub(crate) fn prepare_imap_folder_cursor_for_status(
    mut cursor: ImapFolderCursor,
    uidvalidity: Option<u64>,
    highest_modseq: Option<u64>,
) -> ImapFolderCursor {
    if let (Some(stored), Some(current)) = (cursor.uidvalidity, uidvalidity) {
        if stored != current {
            cursor.last_uid = None;
        }
    }
    if uidvalidity.is_some() {
        cursor.uidvalidity = uidvalidity;
    }
    if highest_modseq.is_some() {
        cursor.highest_modseq = highest_modseq;
    }
    cursor
}

pub(crate) fn serialize_imap_folder_cursor(cursor: &ImapFolderCursor) -> Option<String> {
    serde_json::to_string(cursor).ok()
}

pub(crate) fn can_advance_imap_folder_cursor(has_unresolved_failures: bool) -> bool {
    !has_unresolved_failures
}

pub(crate) fn should_run_imap_deletion_diff(_server_exists: u32, local_count: usize) -> bool {
    local_count > 0
}

pub(crate) fn can_seed_imap_polling_baseline_after_startup(initial_sync_succeeded: bool) -> bool {
    initial_sync_succeeded
}

pub(crate) fn can_refresh_imap_polling_baseline_after_idle_fallback(
    catch_up_succeeded: bool,
) -> bool {
    catch_up_succeeded
}

pub(crate) fn apply_local_inbox_uid_baseline(
    last_exists: &mut Option<MailboxUidState>,
    uidvalidity: Option<u64>,
    local_max_uid: Option<u32>,
    has_unresolved_failures: bool,
) -> bool {
    if has_unresolved_failures {
        return false;
    }
    *last_exists = Some(MailboxUidState {
        uidvalidity,
        highest_uid: local_max_uid.unwrap_or(0),
    });
    true
}

pub(crate) fn should_skip_missing_imap_mailbox_during_initial_sync(
    folder_role: Option<FolderRole>,
) -> bool {
    folder_role != Some(FolderRole::Inbox)
}

pub(crate) fn should_fail_initial_sync_for_folder_error(
    folder_role: Option<FolderRole>,
    is_retryable: bool,
) -> bool {
    folder_role == Some(FolderRole::Inbox) || is_retryable
}

pub(crate) fn idle_check_recovery_user_error(
    reconnect_error: Option<String>,
    poll_error: Option<String>,
) -> Option<(&'static str, String)> {
    if let Some(error) = reconnect_error {
        return Some((
            "connection",
            format!("IMAP reconnect after idle check failed: {error}"),
        ));
    }
    if let Some(error) = poll_error {
        return Some((
            "poll",
            format!("Poll after idle check reconnect failed: {error}"),
        ));
    }
    None
}

pub(crate) fn is_retryable_imap_connection_error(error: &PebbleError) -> bool {
    let PebbleError::Network(message) = error else {
        return false;
    };
    let lower = message.to_ascii_lowercase();

    lower.contains("os error 10053")
        || lower.contains("connection reset")
        || lower.contains("connection aborted")
        || lower.contains("broken pipe")
        || lower.contains("connection closed")
        || lower.contains("closed connection")
        || lower.contains("tls close_notify")
        || lower.contains("unexpected eof")
        || lower.contains("unexpected-eof")
        || lower.contains("timed out")
        || lower == "not connected"
}

pub(crate) fn is_missing_imap_mailbox_error(error: &PebbleError) -> bool {
    let PebbleError::Network(message) = error else {
        return false;
    };
    let lower = message.to_ascii_lowercase();

    lower.contains("folder not exist")
        || lower.contains("mailbox does not exist")
        || lower.contains("mailbox doesn't exist")
        || lower.contains("no such mailbox")
}

pub(crate) fn should_attempt_imap_remote_folder(folder: &Folder) -> bool {
    !folder.remote_id.starts_with("__local_")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImapPollScope {
    Realtime,
    Full,
}

pub(crate) fn should_poll_imap_folder_for_scope(folder: &Folder, scope: ImapPollScope) -> bool {
    if !should_attempt_imap_remote_folder(folder) {
        return false;
    }

    match scope {
        ImapPollScope::Realtime => folder.role == Some(FolderRole::Inbox),
        ImapPollScope::Full => true,
    }
}

#[cfg(test)]
fn should_poll_imap_folder_for_realtime(folder: &Folder) -> bool {
    should_poll_imap_folder_for_scope(folder, ImapPollScope::Realtime)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imap_folder_cursor_roundtrips() {
        let cursor = ImapFolderCursor {
            uidvalidity: Some(1234),
            last_uid: Some(987),
            highest_modseq: Some(4567),
        };

        let json = serde_json::to_string(&cursor).unwrap();
        let decoded: ImapFolderCursor = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, cursor);
    }

    #[test]
    fn imap_folder_cursor_resets_last_uid_when_uidvalidity_changes() {
        let stored = ImapFolderCursor {
            uidvalidity: Some(1234),
            last_uid: Some(987),
            highest_modseq: Some(4567),
        };

        let prepared = prepare_imap_folder_cursor_for_status(stored, Some(9999), Some(7000));

        assert_eq!(prepared.uidvalidity, Some(9999));
        assert_eq!(prepared.last_uid, None);
        assert_eq!(prepared.highest_modseq, Some(7000));
    }

    #[test]
    fn imap_folder_cursor_preserves_last_uid_when_uidvalidity_matches() {
        let stored = ImapFolderCursor {
            uidvalidity: Some(1234),
            last_uid: Some(987),
            highest_modseq: Some(4567),
        };

        let prepared = prepare_imap_folder_cursor_for_status(stored, Some(1234), Some(7000));

        assert_eq!(prepared.uidvalidity, Some(1234));
        assert_eq!(prepared.last_uid, Some(987));
        assert_eq!(prepared.highest_modseq, Some(7000));
    }

    #[test]
    fn imap_folder_cursor_does_not_advance_with_unresolved_failures() {
        assert!(!can_advance_imap_folder_cursor(true));
    }

    #[test]
    fn imap_folder_cursor_advances_without_unresolved_failures() {
        assert!(can_advance_imap_folder_cursor(false));
    }

    #[test]
    fn imap_deletion_diff_runs_when_server_and_local_counts_match() {
        assert!(should_run_imap_deletion_diff(2, 2));
    }

    #[test]
    fn imap_deletion_diff_skips_empty_local_state() {
        assert!(!should_run_imap_deletion_diff(10, 0));
    }

    #[test]
    fn startup_baseline_seed_requires_successful_initial_sync() {
        assert!(can_seed_imap_polling_baseline_after_startup(true));
        assert!(!can_seed_imap_polling_baseline_after_startup(false));
    }

    #[test]
    fn idle_fallback_baseline_refresh_requires_successful_catch_up() {
        assert!(can_refresh_imap_polling_baseline_after_idle_fallback(true));
        assert!(!can_refresh_imap_polling_baseline_after_idle_fallback(
            false
        ));
    }

    #[test]
    fn imap_polling_baseline_refuses_unresolved_inbox_failures() {
        let mut last_exists = Some(MailboxUidState {
            uidvalidity: Some(42),
            highest_uid: 7,
        });

        let seeded = apply_local_inbox_uid_baseline(&mut last_exists, Some(42), Some(12), true);

        assert!(!seeded);
        assert_eq!(
            last_exists,
            Some(MailboxUidState {
                uidvalidity: Some(42),
                highest_uid: 7,
            })
        );
    }

    #[test]
    fn imap_polling_baseline_seeds_local_max_uid_without_unresolved_inbox_failures() {
        let mut last_exists = Some(MailboxUidState {
            uidvalidity: Some(42),
            highest_uid: 7,
        });

        let seeded = apply_local_inbox_uid_baseline(&mut last_exists, Some(43), Some(12), false);

        assert!(seeded);
        assert_eq!(
            last_exists,
            Some(MailboxUidState {
                uidvalidity: Some(43),
                highest_uid: 12,
            })
        );
    }

    #[test]
    fn imap_polling_baseline_seeds_zero_for_clean_empty_local_inbox() {
        let mut last_exists = Some(MailboxUidState {
            uidvalidity: Some(42),
            highest_uid: 7,
        });

        let seeded = apply_local_inbox_uid_baseline(&mut last_exists, Some(43), None, false);

        assert!(seeded);
        assert_eq!(
            last_exists,
            Some(MailboxUidState {
                uidvalidity: Some(43),
                highest_uid: 0,
            })
        );
    }

    #[test]
    fn inbox_missing_mailbox_is_not_skipped_during_initial_sync() {
        assert!(!should_skip_missing_imap_mailbox_during_initial_sync(Some(
            FolderRole::Inbox
        )));
    }

    #[test]
    fn non_inbox_missing_mailbox_can_be_skipped_during_initial_sync() {
        assert!(should_skip_missing_imap_mailbox_during_initial_sync(Some(
            FolderRole::Sent
        )));
    }

    #[test]
    fn inbox_initial_sync_folder_failure_fails_initial_sync() {
        assert!(should_fail_initial_sync_for_folder_error(
            Some(FolderRole::Inbox),
            false
        ));
    }

    #[test]
    fn non_inbox_non_retryable_initial_sync_folder_failure_does_not_fail_initial_sync() {
        assert!(!should_fail_initial_sync_for_folder_error(
            Some(FolderRole::Sent),
            false
        ));
    }

    #[test]
    fn non_inbox_retryable_initial_sync_folder_failure_fails_initial_sync() {
        assert!(should_fail_initial_sync_for_folder_error(None, true));
    }

    #[test]
    fn idle_check_disconnect_does_not_surface_when_recovery_succeeds() {
        let message = idle_check_recovery_user_error(None, None);

        assert!(message.is_none());
    }

    #[test]
    fn idle_check_reconnect_failure_surfaces_connection_error() {
        let message =
            idle_check_recovery_user_error(Some("Network error: os error 10053".to_string()), None);

        assert_eq!(
            message,
            Some((
                "connection",
                "IMAP reconnect after idle check failed: Network error: os error 10053".to_string()
            ))
        );
    }

    #[test]
    fn imap_windows_connection_abort_is_retryable_for_polling() {
        let error = PebbleError::Network(
            "SELECT failed: io: 你的主机中的软件中止了一个已建立的连接。 (os error 10053)"
                .to_string(),
        );

        assert!(is_retryable_imap_connection_error(&error));
    }

    #[test]
    fn imap_rustls_unexpected_eof_is_retryable_for_polling() {
        let error = PebbleError::Network(
            "SELECT failed: io: peer closed connection without sending TLS close_notify: https://docs.rs/rustls/latest/rustls/manual/_03_howto/index.html#unexpected-eof"
                .to_string(),
        );

        assert!(is_retryable_imap_connection_error(&error));
    }

    #[test]
    fn imap_missing_folder_select_error_is_not_retryable_for_polling() {
        let error = PebbleError::Network(
            "SELECT failed: no response: code: None, info: Some(\"SELECT Folder not exist\")"
                .to_string(),
        );

        assert!(!is_retryable_imap_connection_error(&error));
    }

    #[test]
    fn imap_missing_folder_select_error_is_detected_for_suppression() {
        let error = PebbleError::Network(
            "SELECT failed: no response: code: None, info: Some(\"SELECT Folder not exist\")"
                .to_string(),
        );

        assert!(is_missing_imap_mailbox_error(&error));
    }

    #[test]
    fn imap_local_archive_is_not_a_remote_sync_target() {
        let folder = Folder {
            id: "folder-1".to_string(),
            account_id: "account-1".to_string(),
            remote_id: "__local_archive__".to_string(),
            name: "Archive".to_string(),
            folder_type: pebble_core::FolderType::Folder,
            role: Some(FolderRole::Archive),
            parent_id: None,
            color: None,
            is_system: true,
            sort_order: 3,
        };

        assert!(!should_attempt_imap_remote_folder(&folder));
    }

    fn test_folder(role: Option<FolderRole>, remote_id: &str) -> Folder {
        Folder {
            id: format!("folder-{remote_id}"),
            account_id: "account-1".to_string(),
            remote_id: remote_id.to_string(),
            name: remote_id.to_string(),
            folder_type: pebble_core::FolderType::Folder,
            role,
            parent_id: None,
            color: None,
            is_system: true,
            sort_order: 0,
        }
    }

    #[test]
    fn imap_realtime_poll_targets_inbox_only() {
        let inbox = test_folder(Some(FolderRole::Inbox), "INBOX");
        let sent = test_folder(Some(FolderRole::Sent), "Sent");
        let spam = test_folder(Some(FolderRole::Spam), "Spam");
        let custom = test_folder(None, "Newsletters");

        assert!(should_poll_imap_folder_for_realtime(&inbox));
        assert!(!should_poll_imap_folder_for_realtime(&sent));
        assert!(!should_poll_imap_folder_for_realtime(&spam));
        assert!(!should_poll_imap_folder_for_realtime(&custom));
    }

    #[test]
    fn imap_full_poll_targets_all_remote_folders() {
        let inbox = test_folder(Some(FolderRole::Inbox), "INBOX");
        let sent = test_folder(Some(FolderRole::Sent), "Sent");
        let spam = test_folder(Some(FolderRole::Spam), "Spam");
        let custom = test_folder(None, "Newsletters");
        let local = test_folder(Some(FolderRole::Archive), "__local_archive__");

        assert!(should_poll_imap_folder_for_scope(
            &inbox,
            ImapPollScope::Full
        ));
        assert!(should_poll_imap_folder_for_scope(
            &sent,
            ImapPollScope::Full
        ));
        assert!(should_poll_imap_folder_for_scope(
            &spam,
            ImapPollScope::Full
        ));
        assert!(should_poll_imap_folder_for_scope(
            &custom,
            ImapPollScope::Full
        ));
        assert!(!should_poll_imap_folder_for_scope(
            &local,
            ImapPollScope::Full
        ));
    }
}
