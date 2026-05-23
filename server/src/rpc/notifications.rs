use crate::push_notifications::candidate_from_message;
use pebble_core::{FolderRole, Message, PebbleError};
use pebble_store::Store;
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub async fn set_notifications_enabled(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    enabled: bool,
) -> std::result::Result<(), PebbleError> {
    state.notifications_enabled.store(enabled, Ordering::SeqCst);
    Ok(())
}

pub async fn notify_new_message_after_rules(
    state: &crate::state::AppState,
    store: &Arc<Store>,
    message: &Message,
    folder_ids: &[String],
    should_notify: bool,
    notification_deferred_by_remote_rule: bool,
) {
    if !should_notify
        || notification_deferred_by_remote_rule
        || message.is_deleted
        || message.is_read
    {
        return;
    }

    let inbox = match store.find_folder_by_role(&message.account_id, FolderRole::Inbox) {
        Ok(Some(inbox)) => inbox,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(message_id = %message.id, "Failed to load inbox folder before push notification: {error}");
            return;
        }
    };
    if !folder_ids.iter().any(|folder_id| folder_id == &inbox.id) {
        return;
    }

    let account = match store.get_account(&message.account_id) {
        Ok(Some(account)) => account,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(message_id = %message.id, "Failed to load account before push notification: {error}");
            return;
        }
    };

    let candidate = candidate_from_message(message, account.email);
    state
        .push_notifications
        .queue_mail(Arc::clone(store), candidate)
        .await;
}
