use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use p256::elliptic_curve::rand_core::OsRng;
use pebble_core::{Message, PebbleError, Result};
use pebble_store::notification_devices::{NotificationDevice, UnreadInboxSummary};
use pebble_store::Store;
use reqwest::header::{HeaderName, HeaderValue};
use serde::Serialize;
use std::collections::HashMap;
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
const RECENTLY_NOTIFIED_TTL_SECS: i64 = 60 * 60;
const RECENTLY_NOTIFIED_MAX: usize = 4096;
const OTP_CODE_NEAR_KEYWORD_BYTES: usize = 80;
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
    pub kind: MailPushKind,
}

#[derive(Clone, Debug)]
pub enum MailPushKind {
    Mail,
    Otp { code: Option<String> },
}

impl MailPushKind {
    fn is_otp(&self) -> bool {
        matches!(self, Self::Otp { .. })
    }

    fn otp_code(&self) -> Option<&str> {
        match self {
            Self::Otp { code } => code.as_deref(),
            Self::Mail => None,
        }
    }
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
    recently_notified: Mutex<HashMap<String, i64>>,
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
            recently_notified: Mutex::new(HashMap::new()),
        })
    }

    pub fn public_key(&self) -> &str {
        &self.public_key
    }

    pub async fn queue_mail(self: &Arc<Self>, store: Arc<Store>, candidate: MailPushCandidate) {
        {
            let mut seen = self.recently_notified.lock().await;
            if !mark_recently_notified(
                &mut seen,
                &candidate.message_id,
                pebble_core::now_timestamp(),
            ) {
                return;
            }
        }

        if candidate.kind.is_otp() {
            let payload = single_mail_payload(&candidate);
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
            single_mail_payload(&items[0])
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
    let kind = if looks_like_otp_message(message) {
        MailPushKind::Otp {
            code: extract_otp_code(message),
        }
    } else {
        MailPushKind::Mail
    };
    MailPushCandidate {
        message_id: message.id.clone(),
        subject: message.subject.clone(),
        sender: sender_display(message),
        account_email,
        kind,
    }
}

pub fn looks_like_otp_message(message: &Message) -> bool {
    let text = message_search_text(message);
    let lower = text.to_ascii_lowercase();
    if keyword_matches(&lower, STRONG_OTP_KEYWORDS)
        .into_iter()
        .chain(keyword_matches(&lower, STRONG_OTP_WORDS))
        .next()
        .is_some()
    {
        return true;
    }
    keyword_matches(&lower, WEAK_OTP_WORDS)
        .into_iter()
        .next()
        .is_some()
        && first_code_token(&text).is_some()
}

pub fn extract_otp_code(message: &Message) -> Option<String> {
    extract_otp_code_inner(message)
}

fn extract_otp_code_inner(message: &Message) -> Option<String> {
    if !looks_like_otp_message(message) {
        return None;
    }

    let text = message_search_text(message);
    code_token_near_keyword(&text).or_else(|| first_code_token(&text))
}

const STRONG_OTP_KEYWORDS: &[&str] = &[
    "verification code",
    "two-factor",
    "one-time",
    "验证码",
    "驗證碼",
    "校验码",
    "認證碼",
];

const STRONG_OTP_WORDS: &[&str] = &["otp", "2fa"];
const WEAK_OTP_WORDS: &[&str] = &["code", "verification", "verify"];

#[derive(Clone, Copy)]
struct TextSpan {
    start: usize,
    end: usize,
}

fn message_search_text(message: &Message) -> String {
    format!(
        "{}\n{}\n{}",
        message.subject, message.snippet, message.body_text
    )
}

fn keyword_matches(lower_text: &str, keywords: &[&str]) -> Vec<TextSpan> {
    let mut matches = Vec::new();
    for keyword in keywords {
        for (start, _) in lower_text.match_indices(keyword) {
            let end = start + keyword.len();
            if ascii_word_boundaries_match(lower_text, start, end) {
                matches.push(TextSpan { start, end });
            }
        }
    }
    matches
}

fn ascii_word_boundaries_match(text: &str, start: usize, end: usize) -> bool {
    let before = text[..start].chars().next_back();
    let after = text[end..].chars().next();
    !before.is_some_and(|c| c.is_ascii_alphanumeric())
        && !after.is_some_and(|c| c.is_ascii_alphanumeric())
}

fn all_keyword_spans(text: &str) -> Vec<TextSpan> {
    let lower = text.to_ascii_lowercase();
    let mut matches = keyword_matches(&lower, STRONG_OTP_KEYWORDS);
    matches.extend(keyword_matches(&lower, STRONG_OTP_WORDS));
    matches.extend(keyword_matches(&lower, WEAK_OTP_WORDS));
    matches
}

fn code_token_near_keyword(text: &str) -> Option<String> {
    let keywords = all_keyword_spans(text);
    if keywords.is_empty() {
        return None;
    }
    code_tokens(text)
        .into_iter()
        .filter_map(|token| {
            let distance = keywords
                .iter()
                .map(|keyword| span_distance(token.span, *keyword))
                .min()?;
            (distance <= OTP_CODE_NEAR_KEYWORD_BYTES).then_some((distance, token.value))
        })
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, code)| code)
}

fn first_code_token(text: &str) -> Option<String> {
    code_tokens(text)
        .into_iter()
        .next()
        .map(|token| token.value)
}

struct CodeToken {
    span: TextSpan,
    value: String,
}

fn code_tokens(text: &str) -> Vec<CodeToken> {
    let mut tokens = Vec::new();
    let mut start = None;
    for (idx, ch) in text.char_indices() {
        if ch.is_ascii_alphanumeric() {
            if start.is_none() {
                start = Some(idx);
            }
        } else if let Some(token_start) = start.take() {
            push_code_token(text, token_start, idx, &mut tokens);
        }
    }
    if let Some(token_start) = start {
        push_code_token(text, token_start, text.len(), &mut tokens);
    }
    tokens
}

fn push_code_token(text: &str, start: usize, end: usize, tokens: &mut Vec<CodeToken>) {
    let value = &text[start..end];
    if is_code_candidate(value) && !looks_like_year_or_date(value) {
        tokens.push(CodeToken {
            span: TextSpan { start, end },
            value: value.to_string(),
        });
    }
}

fn is_code_candidate(token: &str) -> bool {
    (4..=8).contains(&token.len())
        && token.chars().all(|c| c.is_ascii_alphanumeric())
        && token.chars().any(|c| c.is_ascii_digit())
}

fn looks_like_year_or_date(token: &str) -> bool {
    if !token.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    match token.len() {
        4 => {
            let value = token.parse::<u32>().unwrap_or_default();
            (1900..=2099).contains(&value) || valid_month_day(token)
        }
        8 => valid_year_month_day(token),
        _ => false,
    }
}

fn valid_month_day(token: &str) -> bool {
    let month = token[0..2].parse::<u32>().unwrap_or_default();
    let day = token[2..4].parse::<u32>().unwrap_or_default();
    (1..=12).contains(&month) && (1..=31).contains(&day)
}

fn valid_year_month_day(token: &str) -> bool {
    let year = token[0..4].parse::<u32>().unwrap_or_default();
    (1900..=2099).contains(&year) && valid_month_day(&token[4..8])
}

fn span_distance(left: TextSpan, right: TextSpan) -> usize {
    if left.end <= right.start {
        right.start - left.end
    } else {
        left.start.saturating_sub(right.end)
    }
}

fn mark_recently_notified(
    recently_notified: &mut HashMap<String, i64>,
    message_id: &str,
    now: i64,
) -> bool {
    prune_recently_notified(recently_notified, now);
    if recently_notified.contains_key(message_id) {
        return false;
    }
    recently_notified.insert(message_id.to_string(), now);
    trim_recently_notified(recently_notified);
    true
}

fn prune_recently_notified(recently_notified: &mut HashMap<String, i64>, now: i64) {
    recently_notified
        .retain(|_, notified_at| now.saturating_sub(*notified_at) < RECENTLY_NOTIFIED_TTL_SECS);
}

fn trim_recently_notified(recently_notified: &mut HashMap<String, i64>) {
    while recently_notified.len() > RECENTLY_NOTIFIED_MAX {
        let Some(oldest_id) = recently_notified
            .iter()
            .min_by_key(|(_, notified_at)| *notified_at)
            .map(|(message_id, _)| message_id.clone())
        else {
            break;
        };
        recently_notified.remove(&oldest_id);
    }
}

fn single_mail_payload(candidate: &MailPushCandidate) -> PushPayload {
    let subject = if candidate.subject.trim().is_empty() {
        "(no subject)"
    } else {
        candidate.subject.trim()
    };
    let body = if let Some(code) = candidate.kind.otp_code() {
        format!(
            "{code} · {} · {}",
            candidate.sender, candidate.account_email
        )
    } else {
        format!("{} · {}", candidate.sender, candidate.account_email)
    };
    PushPayload {
        kind: if candidate.kind.is_otp() {
            "otp"
        } else {
            "mail"
        }
        .to_string(),
        title: subject.to_string(),
        body,
        url: format!("/?messageId={}", candidate.message_id),
        tag: format!("pebble-mail-{}", candidate.message_id),
        timestamp: pebble_core::now_timestamp(),
        allow_foreground: candidate.kind.is_otp(),
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
    fn otp_detection_triggers_for_strong_keyword_without_display_code() {
        let candidate = candidate_from_message(
            &message(
                "Your verification code",
                "Open the app to finish signing in",
                "",
            ),
            "account@example.com".to_string(),
        );
        assert!(looks_like_otp_message(&message(
            "Your verification code",
            "Open the app to finish signing in",
            ""
        )));
        assert_eq!(
            extract_otp_code(&message(
                "Your verification code",
                "Open the app to finish signing in",
                ""
            )),
            None
        );
        let payload = single_mail_payload(&candidate);
        assert_eq!(payload.kind, "otp");
        assert!(payload.allow_foreground);
        assert_eq!(payload.body, "Sender · account@example.com");
    }

    #[test]
    fn weak_otp_keywords_require_code_like_token() {
        assert!(!looks_like_otp_message(&message(
            "Please verify your billing address",
            "Open settings to review the change",
            ""
        )));
        assert!(looks_like_otp_message(&message(
            "Please verify your sign-in",
            "Use 123456 to continue",
            ""
        )));
    }

    #[test]
    fn otp_code_extraction_preserves_original_case() {
        assert_eq!(
            extract_otp_code(&message(
                "Your verification code",
                "Use Ab12C to sign in",
                ""
            )),
            Some("Ab12C".to_string())
        );
    }

    #[test]
    fn otp_code_extraction_filters_years_and_dates_before_fallback() {
        assert_eq!(
            extract_otp_code(&message(
                "Your verification code",
                "Expires on 2026-05-27. Use 123456 to sign in",
                ""
            )),
            Some("123456".to_string())
        );
        assert!(!looks_like_otp_message(&message(
            "Weekly report",
            "There were 123456 events",
            ""
        )));
    }

    #[test]
    fn recent_notification_cache_expires_and_caps_entries() {
        let mut recently_notified = HashMap::new();
        assert!(mark_recently_notified(
            &mut recently_notified,
            "message-1",
            100
        ));
        assert!(!mark_recently_notified(
            &mut recently_notified,
            "message-1",
            101
        ));
        assert!(mark_recently_notified(
            &mut recently_notified,
            "message-1",
            100 + RECENTLY_NOTIFIED_TTL_SECS
        ));

        let mut recently_notified = HashMap::new();
        assert!(mark_recently_notified(
            &mut recently_notified,
            "message-0",
            99,
        ));
        for index in 1..=RECENTLY_NOTIFIED_MAX {
            assert!(mark_recently_notified(
                &mut recently_notified,
                &format!("message-{index}"),
                100,
            ));
        }
        assert_eq!(recently_notified.len(), RECENTLY_NOTIFIED_MAX);
        assert!(!recently_notified.contains_key("message-0"));
    }
}
