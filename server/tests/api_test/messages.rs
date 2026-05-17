use crate::test_app;
use axum::http::{Request, StatusCode, header};
use axum::body::Body;
use axum::Router;
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
async fn inbox_returns_401_without_auth() {
    let (app, _dir) = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/inbox?accountId=test&folderId=inbox")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn inbox_returns_200_with_auth() {
    let (app, _dir) = test_app().await;
    let cookie = login(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/inbox?accountId=test&folderId=inbox")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn inbox_returns_expected_structure() {
    let (app, _dir) = test_app().await;
    let cookie = login(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/inbox?accountId=test&folderId=inbox")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 65536).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["messages"].is_array());
    assert!(json["total"].is_number());
    assert!(json["hasMore"].is_boolean());
}

#[tokio::test]
async fn starred_returns_200() {
    let (app, _dir) = test_app().await;
    let cookie = login(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/starred?accountId=test")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // 200 even when account doesn't exist — empty list
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn get_message_returns_404_for_nonexistent() {
    let (app, _dir) = test_app().await;
    let cookie = login(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/messages/nonexistent-id")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
