use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use p256::elliptic_curve::rand_core::OsRng;
use pebble_core::{Message, PebbleError, Result};
use pebble_store::notification_devices::{NotificationDevice, UnreadInboxSummary};
use pebble_store::Store;
use reqwest::header::{HeaderName, HeaderValue};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use web_push::{
    request_builder::{build_request, parse_response},
    ContentEncoding, PartialVapidSignatureBuilder, SubscriptionInfo, Urgency,
    VapidSignatureBuilder, WebPushError, WebPushMessageBuilder,
};

const VAPID_PRIVATE_KEY_ENV: &str = "PEBBLE_VAPID_PRIVATE_KEY";
const VAPID_PUBLIC_KEY_ENV: &str = "PEBBLE_VAPID_PUBLIC_KEY";
const VAPID_STORE_KEY: &str = "web_push_vapid_private_key";
const ORDINARY_BATCH_SECS: u64 = 5;
const ORDINARY_TTL_SECS: u32 = 24 * 60 * 60;
const OTP_TTL_SECS: u32 = 15 * 60;
pub const NOTIFICATION_SESSION_TTL_SECS: i64 = 7 * 24 * 3600;

#[derive(Default)]
struct OrdinaryBatch {
    items: Vec<MailPushCandidate>,
    scheduled: bool,
}

#[derive(Clone, Debug)]
pub struct MailPushCandidate {
    pub message_id: String,
    pub subject: String,
    pub sender: String,
    pub account_email: String,
    pub is_otp: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PushPayload {
    pub kind: String,
    pub title: String,
    pub body: String,
    pub url: String,
    pub tag: String,
    pub timestamp: i64,
    pub allow_foreground: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
}

pub struct PushNotificationService {
    public_key: String,
    vapid: PartialVapidSignatureBuilder,
    client: reqwest::Client,
    ordinary_batch: Mutex<OrdinaryBatch>,
}

impl PushNotificationService {
    pub fn new(store: &Store) -> Result<Self> {
        let private_key = load_or_create_vapid_private_key(store)?;
        let vapid = VapidSignatureBuilder::from_base64_no_sub(&private_key)
            .map_err(|e| PebbleError::Internal(format!("Invalid VAPID private key: {e}")))?;
        let derived_public = URL_SAFE_NO_PAD.encode(vapid.get_public_key());
        let public_key = match std::env::var(VAPID_PUBLIC_KEY_ENV) {
            Ok(configured) => {
                let configured = configured.trim();
                if configured.is_empty() {
                    return Err(PebbleError::Validation(format!(
                        "{VAPID_PUBLIC_KEY_ENV} cannot be empty"
                    )));
                }
                if configured != derived_public {
                    return Err(PebbleError::Validation(format!(
                        "{VAPID_PUBLIC_KEY_ENV} must match {VAPID_PRIVATE_KEY_ENV}"
                    )));
                }
                configured.to_string()
            }
            Err(_) => derived_public,
        };
        let client = reqwest::Client::new();

        Ok(Self {
            public_key,
            vapid,
            client,
            ordinary_batch: Mutex::new(OrdinaryBatch::default()),
        })
    }

    pub fn public_key(&self) -> &str {
        &self.public_key
    }

    pub async fn queue_mail(self: &Arc<Self>, store: Arc<Store>, candidate: MailPushCandidate) {
        if candidate.is_otp {
            let payload = single_mail_payload(&candidate, true);
            self.send_payload_to_active_devices(&store, &payload, OTP_TTL_SECS, Urgency::High)
                .await;
            return;
        }

        let should_spawn = {
            let mut batch = self.ordinary_batch.lock().await;
            batch.items.push(candidate);
            if batch.scheduled {
                false
            } else {
                batch.scheduled = true;
                true
            }
        };

        if should_spawn {
            let service = Arc::clone(self);
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(ORDINARY_BATCH_SECS)).await;
                service.flush_ordinary_batch(store).await;
            });
        }
    }

    pub async fn send_test_to_device(&self, store: &Store, device_id: &str) -> Result<()> {
        let Some(device) = store.get_notification_device(device_id)? else {
            return Err(PebbleError::Validation(
                "Notification device not found".to_string(),
            ));
        };
        let payload = PushPayload {
            kind: "test".to_string(),
            title: "Pebble test notification".to_string(),
            body: "If you can see this, browser push notifications are working.".to_string(),
            url: "/".to_string(),
            tag: "pebble-test".to_string(),
            timestamp: pebble_core::now_timestamp(),
            allow_foreground: true,
            message_id: None,
        };
        self.send_payload_to_device(&device, &payload, ORDINARY_TTL_SECS, Urgency::Normal)
            .await
            .map_err(|e| PebbleError::Network(e.to_string()))
    }

    pub async fn send_unread_summary_to_device(
        &self,
        store: &Store,
        device: &NotificationDevice,
    ) -> Result<()> {
        let summary = store.unread_inbox_summary()?;
        if summary.unread_count == 0 {
            store.mark_notification_summary_sent(&device.id)?;
            return Ok(());
        }

        let payload = summary_payload(&summary);
        self.send_payload_to_device(device, &payload, ORDINARY_TTL_SECS, Urgency::Normal)
            .await
            .map_err(|e| PebbleError::Network(e.to_string()))?;
        store.mark_notification_summary_sent(&device.id)?;
        Ok(())
    }

    async fn flush_ordinary_batch(self: Arc<Self>, store: Arc<Store>) {
        let items = {
            let mut batch = self.ordinary_batch.lock().await;
            batch.scheduled = false;
            std::mem::take(&mut batch.items)
        };
        if items.is_empty() {
            return;
        }

        let payload = if items.len() == 1 {
            single_mail_payload(&items[0], false)
        } else {
            batch_payload(&items)
        };
        self.send_payload_to_active_devices(&store, &payload, ORDINARY_TTL_SECS, Urgency::Normal)
            .await;
    }

    async fn send_payload_to_active_devices(
        &self,
        store: &Store,
        payload: &PushPayload,
        ttl: u32,
        urgency: Urgency,
    ) {
        let now = pebble_core::now_timestamp();
        if let Err(error) = store.pause_expired_notification_devices(now) {
            tracing::warn!("Failed to pause expired notification devices: {error}");
        }
        let devices = match store.list_active_notification_devices(now) {
            Ok(devices) => devices,
            Err(error) => {
                tracing::warn!("Failed to list active notification devices: {error}");
                return;
            }
        };

        for device in devices {
            if let Err(error) = self
                .send_payload_to_device(&device, payload, ttl, urgency)
                .await
            {
                tracing::warn!(device_id = %device.id, "Web Push send failed: {error}");
                if is_permanent_push_error(&error) {
                    if let Err(delete_error) = store.delete_notification_device(&device.id) {
                        tracing::warn!(device_id = %device.id, "Failed to delete invalid notification device: {delete_error}");
                    }
                }
            }
        }
    }

    async fn send_payload_to_device(
        &self,
        device: &NotificationDevice,
        payload: &PushPayload,
        ttl: u32,
        urgency: Urgency,
    ) -> std::result::Result<(), WebPushError> {
        let subscription = SubscriptionInfo::new(
            device.endpoint.clone(),
            device.p256dh.clone(),
            device.auth.clone(),
        );
        let signature = self.vapid.clone().add_sub_info(&subscription).build()?;
        let content = serde_json::to_vec(payload)?;
        let mut builder = WebPushMessageBuilder::new(&subscription);
        builder.set_payload(ContentEncoding::Aes128Gcm, &content);
        builder.set_vapid_signature(signature);
        builder.set_ttl(ttl);
        builder.set_urgency(urgency);
        let request = build_request::<Vec<u8>>(builder.build()?);
        let method = reqwest::Method::from_bytes(request.method().as_str().as_bytes())
            .map_err(|_| WebPushError::InvalidUri)?;
        let uri = request.uri().to_string();
        let mut send_request = self.client.request(method, uri);
        for (name, value) in request.headers() {
            let header_name = HeaderName::from_bytes(name.as_str().as_bytes())
                .map_err(|_| WebPushError::InvalidResponse)?;
            let header_value = HeaderValue::from_bytes(value.as_bytes())
                .map_err(|_| WebPushError::InvalidResponse)?;
            send_request = send_request.header(header_name, header_value);
        }

        let response = send_request
            .body(request.into_body())
            .send()
            .await
            .map_err(|_| WebPushError::Unspecified)?;
        let status = http02::StatusCode::from_u16(response.status().as_u16())
            .map_err(|_| WebPushError::InvalidResponse)?;
        let body = response
            .bytes()
            .await
            .map_err(|_| WebPushError::InvalidResponse)?;
        parse_response(status, body.to_vec())
    }
}

pub fn candidate_from_message(message: &Message, account_email: String) -> MailPushCandidate {
    MailPushCandidate {
        message_id: message.id.clone(),
        subject: message.subject.clone(),
        sender: sender_display(message),
        account_email,
        is_otp: looks_like_otp_message(message),
    }
}

pub fn looks_like_otp_message(message: &Message) -> bool {
    let text = format!(
        "{}\n{}\n{}",
        message.subject, message.snippet, message.body_text
    )
    .to_lowercase();
    let has_keyword = [
        "code",
        "verification",
        "verify",
        "otp",
        "2fa",
        "two-factor",
        "one-time",
        "验证码",
        "驗證碼",
        "校验码",
        "認證碼",
    ]
    .iter()
    .any(|keyword| text.contains(keyword));
    if !has_keyword {
        return false;
    }

    text.split(|c: char| !c.is_ascii_alphanumeric())
        .any(|part| part.len() >= 4 && part.len() <= 8 && part.chars().any(|c| c.is_ascii_digit()))
}

fn single_mail_payload(candidate: &MailPushCandidate, otp: bool) -> PushPayload {
    let subject = if candidate.subject.trim().is_empty() {
        "(no subject)"
    } else {
        candidate.subject.trim()
    };
    PushPayload {
        kind: if otp { "otp" } else { "mail" }.to_string(),
        title: if otp {
            format!("Verification code · {subject}")
        } else {
            subject.to_string()
        },
        body: format!("{} · {}", candidate.sender, candidate.account_email),
        url: format!("/?messageId={}", candidate.message_id),
        tag: format!("pebble-mail-{}", candidate.message_id),
        timestamp: pebble_core::now_timestamp(),
        allow_foreground: otp,
        message_id: Some(candidate.message_id.clone()),
    }
}

fn batch_payload(items: &[MailPushCandidate]) -> PushPayload {
    let senders = items
        .iter()
        .rev()
        .map(|item| item.sender.clone())
        .filter(|sender| !sender.trim().is_empty())
        .take(3)
        .collect::<Vec<_>>();
    PushPayload {
        kind: "mail_batch".to_string(),
        title: format!("{} new emails", items.len()),
        body: if senders.is_empty() {
            "Open Pebble to view your inbox".to_string()
        } else {
            senders.join(", ")
        },
        url: "/".to_string(),
        tag: "pebble-mail-batch".to_string(),
        timestamp: pebble_core::now_timestamp(),
        allow_foreground: false,
        message_id: None,
    }
}

fn summary_payload(summary: &UnreadInboxSummary) -> PushPayload {
    PushPayload {
        kind: "summary".to_string(),
        title: format!("{} unread emails", summary.unread_count),
        body: if summary.sample_senders.is_empty() {
            "Open Pebble to view your inbox".to_string()
        } else {
            summary.sample_senders.join(", ")
        },
        url: "/".to_string(),
        tag: "pebble-unread-summary".to_string(),
        timestamp: pebble_core::now_timestamp(),
        allow_foreground: true,
        message_id: None,
    }
}

fn sender_display(message: &Message) -> String {
    if !message.from_name.trim().is_empty() {
        return message.from_name.clone();
    }
    if !message.from_address.trim().is_empty() {
        return message.from_address.clone();
    }
    "Unknown sender".to_string()
}

fn load_or_create_vapid_private_key(store: &Store) -> Result<String> {
    if let Ok(private_key) = std::env::var(VAPID_PRIVATE_KEY_ENV) {
        if private_key.trim().is_empty() {
            return Err(PebbleError::Validation(format!(
                "{VAPID_PRIVATE_KEY_ENV} cannot be empty"
            )));
        }
        return Ok(private_key.trim().to_string());
    }

    if let Some(stored) = store.get_secure_user_data(VAPID_STORE_KEY)? {
        let private_key = String::from_utf8(stored)
            .map_err(|e| PebbleError::Storage(format!("Invalid stored VAPID key: {e}")))?;
        if !private_key.trim().is_empty() {
            return Ok(private_key);
        }
    }

    let secret = p256::SecretKey::random(&mut OsRng);
    let private_key = URL_SAFE_NO_PAD.encode(secret.to_bytes());
    store.set_secure_user_data(VAPID_STORE_KEY, private_key.as_bytes())?;
    Ok(private_key)
}

fn is_permanent_push_error(error: &WebPushError) -> bool {
    matches!(
        error,
        WebPushError::EndpointNotFound(_)
            | WebPushError::EndpointNotValid(_)
            | WebPushError::Unauthorized(_)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(subject: &str, snippet: &str, body_text: &str) -> Message {
        let now = pebble_core::now_timestamp();
        Message {
            id: "message-1".to_string(),
            account_id: "account-1".to_string(),
            remote_id: "remote-1".to_string(),
            message_id_header: None,
            in_reply_to: None,
            references_header: None,
            thread_id: None,
            subject: subject.to_string(),
            snippet: snippet.to_string(),
            from_address: "sender@example.com".to_string(),
            from_name: "Sender".to_string(),
            to_list: vec![],
            cc_list: vec![],
            bcc_list: vec![],
            body_text: body_text.to_string(),
            body_html_raw: String::new(),
            has_attachments: false,
            is_read: false,
            is_starred: false,
            is_draft: false,
            date: now,
            remote_version: None,
            is_deleted: false,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn otp_detection_requires_keyword_and_code_like_token() {
        assert!(looks_like_otp_message(&message(
            "Your verification code",
            "Use 123456 to sign in",
            ""
        )));
        assert!(!looks_like_otp_message(&message(
            "Weekly report",
            "There were 123456 events",
            ""
        )));
    }
}
