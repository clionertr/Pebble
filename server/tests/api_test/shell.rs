use crate::test_app;
use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use tower::ServiceExt;

/// Helper: login and return session cookie.
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
async fn shell_returns_401_without_auth() {
    let (app, _dir) = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/shell")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn shell_returns_200_with_auth() {
    let (app, _dir) = test_app().await;
    let cookie = login(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/shell")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn shell_returns_expected_structure() {
    let (app, _dir) = test_app().await;
    let cookie = login(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/shell")
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
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Response shape: { accounts, folders, unreadCounts, gmailRealtime }
    assert!(
        json["accounts"].is_array(),
        "accounts should be array: {json}"
    );
    assert!(
        json["folders"].is_object(),
        "folders should be object: {json}"
    );
    assert!(
        json["unreadCounts"].is_object(),
        "unreadCounts should be object: {json}"
    );
    assert!(
        json["gmailRealtime"].is_object(),
        "gmailRealtime should be object: {json}"
    );
}
