use crate::push_notifications::{candidate_from_message, NOTIFICATION_SESSION_TTL_SECS};
use crate::state::AppState;
use axum::http::{header, HeaderMap};
use pebble_core::{FolderRole, Message, PebbleError};
use pebble_store::notification_devices::{NotificationDevice, UpsertNotificationDevice};
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

pub struct BrowserSubscriptionKeys {
    pub p256dh: String,
    pub auth: String,
}

pub struct BrowserSubscription {
    pub endpoint: String,
    pub keys: BrowserSubscriptionKeys,
}

pub struct UpsertSubscriptionInput {
    pub device_id: String,
    pub device_name: Option<String>,
    pub subscription: BrowserSubscription,
    pub session_id: Option<String>,
    pub user_agent: Option<String>,
}

pub fn user_agent_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
}

pub fn default_device_name(user_agent: Option<&str>) -> String {
    let user_agent = user_agent.unwrap_or_default();
    let browser = if user_agent.contains("Firefox") {
        "Firefox"
    } else if user_agent.contains("Edg/") {
        "Edge"
    } else if user_agent.contains("Chrome") {
        "Chrome"
    } else if user_agent.contains("Safari") {
        "Safari"
    } else {
        "Browser"
    };
    let os = if user_agent.contains("Windows") {
        "Windows"
    } else if user_agent.contains("Mac OS X") {
        "macOS"
    } else if user_agent.contains("Linux") {
        "Linux"
    } else if user_agent.contains("Android") {
        "Android"
    } else if user_agent.contains("iPhone") || user_agent.contains("iPad") {
        "iOS"
    } else {
        "this device"
    };
    format!("{browser} on {os}")
}

pub async fn list_notification_devices(
    state: &AppState,
) -> Result<Vec<NotificationDevice>, PebbleError> {
    state
        .store
        .pause_expired_notification_devices(pebble_core::now_timestamp())?;
    state.store.list_notification_devices()
}

pub async fn upsert_subscription(
    state: &AppState,
    input: UpsertSubscriptionInput,
) -> Result<NotificationDevice, PebbleError> {
    let device_id = input.device_id.trim();
    if device_id.is_empty() {
        return Err(PebbleError::Validation("device_id is required".to_string()));
    }
    if input.subscription.endpoint.trim().is_empty()
        || input.subscription.keys.p256dh.trim().is_empty()
        || input.subscription.keys.auth.trim().is_empty()
    {
        return Err(PebbleError::Validation(
            "subscription endpoint and keys are required".to_string(),
        ));
    }

    let now = pebble_core::now_timestamp();
    let device_name = input
        .device_name
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| default_device_name(input.user_agent.as_deref()));
    let device = state
        .store
        .upsert_notification_device(UpsertNotificationDevice {
            id: device_id.to_string(),
            endpoint: input.subscription.endpoint,
            p256dh: input.subscription.keys.p256dh,
            auth: input.subscription.keys.auth,
            device_name,
            user_agent: input.user_agent,
            session_id: input.session_id,
            session_expires_at: Some(now + NOTIFICATION_SESSION_TTL_SECS),
        })?;

    if device.summary_sent_at.is_none() {
        state
            .push_notifications
            .send_unread_summary_to_device(&state.store, &device)
            .await?;
    }

    Ok(state
        .store
        .get_notification_device(device_id)?
        .unwrap_or(device))
}

pub fn rename_notification_device(
    state: &AppState,
    device_id: &str,
    device_name: &str,
) -> Result<Option<NotificationDevice>, PebbleError> {
    let device_name = device_name.trim();
    if device_name.is_empty() {
        return Err(PebbleError::Validation(
            "device_name is required".to_string(),
        ));
    }
    state
        .store
        .rename_notification_device(device_id, device_name)?;
    state.store.get_notification_device(device_id)
}

pub fn delete_notification_device(state: &AppState, device_id: &str) -> Result<(), PebbleError> {
    state.store.delete_notification_device(device_id)
}

pub async fn send_test_notification(state: &AppState, device_id: &str) -> Result<(), PebbleError> {
    if device_id.trim().is_empty() {
        return Err(PebbleError::Validation("device_id is required".to_string()));
    }
    state
        .push_notifications
        .send_test_to_device(&state.store, device_id)
        .await
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
