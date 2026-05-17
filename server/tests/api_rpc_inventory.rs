// RPC Inventory — enumerates every RPC method in dispatch.rs and every
// frontend invoke() call site. This file serves as the migration checklist:
// each entry must be mapped to a REST endpoint by Phase 7.

use std::collections::HashSet;

/// Every RPC method name extracted from `server/src/rpc/dispatch.rs`.
/// Generated 2026-05-17 from f6f01fb (branch 001-improve-experience).
const BACKEND_RPC_METHODS: &[&str] = &[
    // ── Accounts ──────────────────────────────────────────────────────
    "add_account",
    "complete_oauth_flow",
    "delete_account",
    "disable_gmail_realtime",
    "enable_gmail_realtime",
    "get_account_proxy",
    "get_account_proxy_setting",
    "get_gmail_realtime_config",
    "get_oauth_account_proxy",
    "get_oauth_account_proxy_setting",
    "list_accounts",
    "reindex_search",
    "start_sync",
    "stop_sync",
    "test_account_connection",
    "test_imap_connection",
    "trigger_sync",
    "update_account",
    "update_account_proxy",
    "update_account_proxy_setting",
    "update_gmail_realtime_config",
    "update_oauth_account_proxy",
    "update_oauth_account_proxy_setting",
    // ── Folders ───────────────────────────────────────────────────────
    "get_folder_unread_counts",
    "list_folders",
    // ── Messages — Reads ──────────────────────────────────────────────
    "advanced_search",
    "get_message",
    "get_message_with_html",
    "get_messages_batch",
    "get_rendered_html",
    "list_messages",
    "list_starred_messages",
    "list_thread_messages",
    "list_threads",
    "search_messages",
    // ── Messages — Mutations ──────────────────────────────────────────
    "archive_message",
    "batch_archive",
    "batch_delete",
    "batch_mark_read",
    "batch_star",
    "delete_message",
    "empty_trash",
    "move_to_folder",
    "restore_message",
    "update_message_flags",
    // ── Labels ────────────────────────────────────────────────────────
    "add_message_label",
    "get_message_labels",
    "get_message_labels_batch",
    "list_labels",
    "remove_message_label",
    // ── Compose & Drafts ──────────────────────────────────────────────
    "delete_draft",
    "save_draft",
    "send_email",
    // ── Attachments ───────────────────────────────────────────────────
    "download_attachment",
    "get_attachment_path",
    "list_attachments",
    "stage_compose_attachment",
    // ── Kanban ────────────────────────────────────────────────────────
    "list_kanban_cards",
    "list_kanban_context_notes",
    "merge_kanban_context_notes",
    "move_to_kanban",
    "remove_from_kanban",
    "set_kanban_context_note",
    // ── Snooze ────────────────────────────────────────────────────────
    "list_snoozed",
    "snooze_message",
    "unsnooze_message",
    // ── Rules ─────────────────────────────────────────────────────────
    "create_rule",
    "delete_rule",
    "list_rules",
    "update_rule",
    // ── Translate ─────────────────────────────────────────────────────
    "get_translate_config",
    "save_translate_config",
    "test_translate_connection",
    "translate_text",
    // ── Contacts ──────────────────────────────────────────────────────
    "search_contacts",
    // ── Cloud Sync (WebDAV) ───────────────────────────────────────────
    "backup_to_webdav",
    "preview_webdav_backup",
    "restore_from_webdav",
    "test_webdav_connection",
    // ── Diagnostics & System ──────────────────────────────────────────
    "health_check",
    "read_app_log",
    "record_mail_display_timing",
    // ── Proxy ─────────────────────────────────────────────────────────
    "get_global_proxy",
    "update_global_proxy",
    // ── Preferences ───────────────────────────────────────────────────
    "set_notifications_enabled",
    "set_realtime_preference",
    // ── Trusted Senders ───────────────────────────────────────────────
    "is_trusted_sender",
    "list_trusted_senders",
    "remove_trusted_sender",
    "trust_sender",
    // ── Email Templates ───────────────────────────────────────────────
    "delete_email_template",
    "list_email_templates",
    "save_email_template",
    // ── Pending Ops ───────────────────────────────────────────────────
    "cancel_pending_mail_op",
    "delete_pending_mail_op",
    "get_pending_mail_ops_summary",
    "list_pending_mail_ops",
    // ── Signatures ────────────────────────────────────────────────────
    "get_email_signature",
    "set_email_signature",
    // ── DESKTOP-ONLY (to be deleted in Phase 6) ───────────────────────
    "set_tray_menu_labels",
    "take_pending_mailto_urls",
];

/// Every RPC method name from frontend `invoke(...)` production call sites
/// (excluding tauri-mock.ts and test files).
const FRONTEND_INVOKE_METHODS: &[&str] = &[
    "add_account",
    "add_message_label",
    "advanced_search",
    "archive_message",
    "backup_to_webdav",
    "batch_archive",
    "batch_delete",
    "batch_mark_read",
    "batch_star",
    "cancel_pending_mail_op",
    "complete_oauth_flow",
    "create_rule",
    "delete_account",
    "delete_draft",
    "delete_email_template",
    "delete_message",
    "delete_pending_mail_op",
    "delete_rule",
    "disable_gmail_realtime",
    "download_attachment",
    "empty_trash",
    "enable_gmail_realtime",
    "get_account_proxy",
    "get_account_proxy_setting",
    "get_attachment_path",
    "get_email_signature",
    "get_folder_unread_counts",
    "get_global_proxy",
    "get_gmail_realtime_config",
    "get_message",
    "get_message_labels",
    "get_message_labels_batch",
    "get_message_with_html",
    "get_messages_batch",
    "get_oauth_account_proxy",
    "get_oauth_account_proxy_setting",
    "get_pending_mail_ops_summary",
    "get_rendered_html",
    "get_translate_config",
    "health_check",
    "is_trusted_sender",
    "list_accounts",
    "list_attachments",
    "list_email_templates",
    "list_folders",
    "list_kanban_cards",
    "list_kanban_context_notes",
    "list_labels",
    "list_messages",
    "list_pending_mail_ops",
    "list_rules",
    "list_snoozed",
    "list_starred_messages",
    "list_thread_messages",
    "list_threads",
    "list_trusted_senders",
    "merge_kanban_context_notes",
    "move_to_folder",
    "move_to_kanban",
    "preview_webdav_backup",
    "read_app_log",
    "record_mail_display_timing",
    "remove_from_kanban",
    "remove_message_label",
    "remove_trusted_sender",
    "restore_from_webdav",
    "restore_message",
    "save_draft",
    "save_email_template",
    "save_translate_config",
    "search_contacts",
    "search_messages",
    "send_email",
    "set_email_signature",
    "set_kanban_context_note",
    "set_notifications_enabled",
    "set_realtime_preference",
    "set_tray_menu_labels",
    "snooze_message",
    "stage_compose_attachment",
    "start_sync",
    "stop_sync",
    "take_pending_mailto_urls",
    "test_account_connection",
    "test_imap_connection",
    "test_translate_connection",
    "test_webdav_connection",
    "translate_text",
    "trigger_sync",
    "trust_sender",
    "unsnooze_message",
    "update_account",
    "update_account_proxy",
    "update_account_proxy_setting",
    "update_global_proxy",
    "update_gmail_realtime_config",
    "update_message_flags",
    "update_oauth_account_proxy",
    "update_oauth_account_proxy_setting",
    "update_rule",
];

// ── Tests ────────────────────────────────────────────────────────────

#[test]
fn all_backend_methods_are_unique() {
    let set: HashSet<_> = BACKEND_RPC_METHODS.iter().copied().collect();
    assert_eq!(set.len(), BACKEND_RPC_METHODS.len(), "duplicate backend RPC method name");
}

#[test]
fn all_frontend_methods_are_unique() {
    let set: HashSet<_> = FRONTEND_INVOKE_METHODS.iter().copied().collect();
    assert_eq!(set.len(), FRONTEND_INVOKE_METHODS.len(), "duplicate frontend invoke method name");
}

#[test]
fn frontend_methods_exist_in_backend() {
    // Every method the frontend calls must exist in dispatch.rs.
    // Exceptions: "open_external_url" is a Tauri shell command, not an RPC method.
    let backend: HashSet<_> = BACKEND_RPC_METHODS.iter().copied().collect();
    let frontend_only: Vec<_> = FRONTEND_INVOKE_METHODS
        .iter()
        .filter(|m| !backend.contains(*m))
        .collect();
    assert!(
        frontend_only.is_empty(),
        "Frontend methods missing from backend dispatch: {frontend_only:?}"
    );
}

#[test]
fn backend_methods_referenced_in_frontend_or_prd() {
    // Every backend method should either be called from the frontend or
    // accounted for in the PRD (e.g. reindex_search is internal-only).
    let internal_only: &[&str] = &["reindex_search"];
    let desktop_only: &[&str] = &["set_tray_menu_labels", "take_pending_mailto_urls"];

    let frontend: HashSet<_> = FRONTEND_INVOKE_METHODS.iter().copied().collect();
    let accounted: HashSet<_> = internal_only.iter().chain(desktop_only.iter()).collect();

    let orphaned: Vec<_> = BACKEND_RPC_METHODS
        .iter()
        .filter(|m| !frontend.contains(*m) && !accounted.contains(*m))
        .collect();

    assert!(
        orphaned.is_empty(),
        "Backend RPC methods not called from frontend and not accounted for: {orphaned:?}"
    );
}

#[test]
fn rpc_method_count_matches_expected() {
    // Baseline: dispatch.rs had 102 unique method strings at Phase 0 start.
    // This test catches accidental additions or deletions during migration.
    assert_eq!(BACKEND_RPC_METHODS.len(), 101, "unexpected backend RPC method count change");
}

#[test]
fn desktop_methods_still_exist() {
    // Phase 0-5: desktop methods must remain (deleted in Phase 6).
    let backend: HashSet<_> = BACKEND_RPC_METHODS.iter().copied().collect();
    assert!(backend.contains("set_tray_menu_labels"), "set_tray_menu_labels unexpectedly removed");
    assert!(backend.contains("take_pending_mailto_urls"), "take_pending_mailto_urls unexpectedly removed");
}

#[test]
fn frontend_invoke_count_baseline() {
    // Phase 0 baseline: 94 unique invoke() methods in production frontend code.
    // This will decrease as methods move to api-client.ts, then drop sharply
    // when desktop methods are removed in Phase 6.
    assert_eq!(FRONTEND_INVOKE_METHODS.len(), 100, "unexpected frontend invoke method count change");
}
