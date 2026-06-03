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
async fn search_rejects_overlong_query() {
    let (app, _dir, _state) = test_app().await;
    let cookie = login(&app).await;
    let query = "x".repeat(501);

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/search?q={query}"))
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn advanced_search_rejects_invalid_query_shape() {
    let (app, _dir, _state) = test_app().await;
    let cookie = login(&app).await;
    let body = json!({
        "query": "not-an-object",
        "limit": 10
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/search/advanced")
                .method("POST")
                .header(header::COOKIE, cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn search_returns_429_when_concurrency_limit_is_exhausted() {
    let (app, _dir, state) = test_app().await;
    let cookie = login(&app).await;
    let _permits = state
        .rpc_semaphore
        .clone()
        .acquire_many_owned(64)
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/search?q=test")
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}
