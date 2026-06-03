use crate::api::error::ApiError;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    routing::{delete, get, patch, post},
    Json, Router,
};
use axum_extra::extract::cookie::CookieJar;
use pebble_store::notification_devices::NotificationDevice;
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
    Ok(Json(DeviceListResponse {
        devices: crate::rpc::notifications::list_notification_devices(&state).await?,
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
    let session_id = jar
        .get(SESSION_COOKIE)
        .map(|cookie| cookie.value().to_string());
    let device = crate::rpc::notifications::upsert_subscription(
        &state,
        crate::rpc::notifications::UpsertSubscriptionInput {
            device_id: body.device_id,
            device_name: body.device_name,
            subscription: crate::rpc::notifications::BrowserSubscription {
                endpoint: body.subscription.endpoint,
                keys: crate::rpc::notifications::BrowserSubscriptionKeys {
                    p256dh: body.subscription.keys.p256dh,
                    auth: body.subscription.keys.auth,
                },
            },
            session_id,
            user_agent: crate::rpc::notifications::user_agent_from_headers(&headers),
        },
    )
    .await?;
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
    let device = crate::rpc::notifications::rename_notification_device(
        &state,
        &device_id,
        &body.device_name,
    )?
    .ok_or_else(|| ApiError::not_found("Notification device not found"))?;
    Ok(Json(device))
}

async fn delete_device(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<String>,
) -> Result<Json<()>, ApiError> {
    crate::rpc::notifications::delete_notification_device(&state, &device_id)?;
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
    crate::rpc::notifications::send_test_notification(&state, &body.device_id).await?;
    Ok(Json(()))
}
