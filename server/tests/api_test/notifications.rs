use crate::test_app;
use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use serde_json::json;
use tower::ServiceExt;

async fn login(app: &Router) -> String {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/auth/login")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"password":"test-password"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    response
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

async fn get_json(app: &Router, cookie: &str, uri: &str) -> serde_json::Value {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(uri)
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 65536)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn vapid_public_key_returns_key() {
    let (app, _dir, _state) = test_app().await;
    let cookie = login(&app).await;

    let json = get_json(&app, &cookie, "/api/notifications/vapid-public-key").await;

    assert!(
        json["public_key"]
            .as_str()
            .is_some_and(|key| !key.is_empty()),
        "public key response: {json}"
    );
}

#[tokio::test]
async fn notification_subscription_registers_and_renames_device() {
    let (app, _dir, _state) = test_app().await;
    let cookie = login(&app).await;
    let body = json!({
        "device_id": "device-1",
        "subscription": {
            "endpoint": "https://push.example.test/subscription/device-1",
            "keys": {
                "p256dh": "p256dh-key",
                "auth": "auth-secret"
            }
        }
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/notifications/subscriptions")
                .method("POST")
                .header(header::COOKIE, &cookie)
                .header(header::USER_AGENT, "Mozilla/5.0 Chrome Linux")
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response_body = axum::body::to_bytes(response.into_body(), 65536)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&response_body).unwrap();
    assert_eq!(response_json["device"]["id"], "device-1");
    assert_eq!(response_json["device"]["device_name"], "Chrome on Linux");

    let list = get_json(&app, &cookie, "/api/notifications/devices").await;
    assert_eq!(list["devices"].as_array().unwrap().len(), 1);

    let rename_body = json!({ "device_name": "Laptop" });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/notifications/devices/device-1")
                .method("PATCH")
                .header(header::COOKIE, &cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(rename_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response_body = axum::body::to_bytes(response.into_body(), 65536)
        .await
        .unwrap();
    let response_json: serde_json::Value = serde_json::from_slice(&response_body).unwrap();
    assert_eq!(response_json["device_name"], "Laptop");
}
