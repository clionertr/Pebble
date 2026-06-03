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

#[tokio::test]
async fn diagnostics_routes_read_logs_and_validate_mail_timing_payload() {
    let (app, _dir, _state) = test_app().await;
    let cookie = login(&app).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/logs?maxBytes=128")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 65536)
        .await
        .unwrap();
    let snapshot: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(snapshot["content"].is_string());
    assert!(snapshot["truncated"].is_boolean());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/diagnostics/mail-timing")
                .method("POST")
                .header(header::COOKIE, &cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "messageId": "message-1",
                        "frontendSseAtMs": 1000,
                        "displayedAtMs": 1200
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/diagnostics/mail-timing")
                .method("POST")
                .header(header::COOKIE, &cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "messageId": "missing-times" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
