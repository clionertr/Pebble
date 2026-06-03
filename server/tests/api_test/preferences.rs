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
async fn preference_routes_accept_valid_values_and_reject_bad_realtime_mode() {
    let (app, _dir, _state) = test_app().await;
    let cookie = login(&app).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/preferences/realtime")
                .method("PUT")
                .header(header::COOKIE, &cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "mode": "manual" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/preferences/notifications")
                .method("PUT")
                .header(header::COOKIE, &cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "enabled": true }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/preferences/realtime")
                .method("PUT")
                .header(header::COOKIE, &cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "mode": "turbo" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
