use crate::api::error::ApiError;
use crate::push_notifications::NOTIFICATION_SESSION_TTL_SECS;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    routing::{delete, get, patch, post},
    Json, Router,
};
use axum_extra::extract::cookie::CookieJar;
use pebble_store::notification_devices::{NotificationDevice, UpsertNotificationDevice};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const SESSION_COOKIE: &str = "pebble_session";

pub fn notification_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/notifications/vapid-public-key", get(vapid_public_key))
        .route("/api/notifications/devices", get(list_devices))
        .route(
            "/api/notifications/devices/:device_id",
            patch(rename_device).delete(delete_device),
        )
        .route(
            "/api/notifications/subscriptions",
            post(upsert_subscription),
        )
        .route(
            "/api/notifications/subscriptions/:device_id",
            delete(delete_device),
        )
        .route("/api/notifications/test", post(send_test_notification))
}

#[derive(Serialize)]
struct PublicKeyResponse {
    public_key: String,
}

async fn vapid_public_key(State(state): State<Arc<AppState>>) -> Json<PublicKeyResponse> {
    Json(PublicKeyResponse {
        public_key: state.push_notifications.public_key().to_string(),
    })
}

#[derive(Serialize)]
struct DeviceListResponse {
    devices: Vec<NotificationDevice>,
}

async fn list_devices(
    State(state): State<Arc<AppState>>,
) -> Result<Json<DeviceListResponse>, ApiError> {
    state
        .store
        .pause_expired_notification_devices(pebble_core::now_timestamp())?;
    Ok(Json(DeviceListResponse {
        devices: state.store.list_notification_devices()?,
    }))
}

#[derive(Deserialize)]
struct BrowserSubscriptionKeys {
    p256dh: String,
    auth: String,
}

#[derive(Deserialize)]
struct BrowserSubscription {
    endpoint: String,
    keys: BrowserSubscriptionKeys,
}

#[derive(Deserialize)]
struct UpsertSubscriptionRequest {
    device_id: String,
    device_name: Option<String>,
    subscription: BrowserSubscription,
}

#[derive(Serialize)]
struct UpsertSubscriptionResponse {
    device: NotificationDevice,
}

async fn upsert_subscription(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    headers: HeaderMap,
    Json(body): Json<UpsertSubscriptionRequest>,
) -> Result<Json<UpsertSubscriptionResponse>, ApiError> {
    let device_id = body.device_id.trim();
    if device_id.is_empty() {
        return Err(ApiError::bad_request("device_id is required"));
    }
    if body.subscription.endpoint.trim().is_empty()
        || body.subscription.keys.p256dh.trim().is_empty()
        || body.subscription.keys.auth.trim().is_empty()
    {
        return Err(ApiError::bad_request(
            "subscription endpoint and keys are required",
        ));
    }

    let session_id = jar
        .get(SESSION_COOKIE)
        .map(|cookie| cookie.value().to_string());
    let now = pebble_core::now_timestamp();
    let device_name = body
        .device_name
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| default_device_name(&headers));
    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let device = state
        .store
        .upsert_notification_device(UpsertNotificationDevice {
            id: device_id.to_string(),
            endpoint: body.subscription.endpoint,
            p256dh: body.subscription.keys.p256dh,
            auth: body.subscription.keys.auth,
            device_name,
            user_agent,
            session_id,
            session_expires_at: Some(now + NOTIFICATION_SESSION_TTL_SECS),
        })?;

    if device.summary_sent_at.is_none() {
        state
            .push_notifications
            .send_unread_summary_to_device(&state.store, &device)
            .await?;
    }

    let device = state
        .store
        .get_notification_device(device_id)?
        .unwrap_or(device);
    Ok(Json(UpsertSubscriptionResponse { device }))
}

#[derive(Deserialize)]
struct RenameDeviceRequest {
    device_name: String,
}

async fn rename_device(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
    Json(body): Json<RenameDeviceRequest>,
) -> Result<Json<NotificationDevice>, ApiError> {
    let device_name = body.device_name.trim();
    if device_name.is_empty() {
        return Err(ApiError::bad_request("device_name is required"));
    }
    state
        .store
        .rename_notification_device(&device_id, device_name)?;
    let device = state
        .store
        .get_notification_device(&device_id)?
        .ok_or_else(|| ApiError::not_found("Notification device not found"))?;
    Ok(Json(device))
}

async fn delete_device(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
) -> Result<Json<()>, ApiError> {
    state.store.delete_notification_device(&device_id)?;
    Ok(Json(()))
}

#[derive(Deserialize)]
struct TestNotificationRequest {
    device_id: String,
}

async fn send_test_notification(
    State(state): State<Arc<AppState>>,
    Json(body): Json<TestNotificationRequest>,
) -> Result<Json<()>, ApiError> {
    if body.device_id.trim().is_empty() {
        return Err(ApiError::bad_request("device_id is required"));
    }
    state
        .push_notifications
        .send_test_to_device(&state.store, &body.device_id)
        .await?;
    Ok(Json(()))
}

fn default_device_name(headers: &HeaderMap) -> String {
    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
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
