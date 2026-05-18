use crate::test_app;
use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use tower::ServiceExt;

/// Helper: login and return the session cookie value.
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
async fn health_returns_200() {
    let (app, _dir) = test_app().await;
    let cookie = login(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .method("GET")
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn health_returns_text_ok() {
    let (app, _dir) = test_app().await;
    let cookie = login(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    assert_eq!(body, "ok");
}

#[tokio::test]
async fn nonexistent_route_returns_404() {
    let (app, _dir) = test_app().await;
    let cookie = login(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/nonexistent-route-xyz")
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn wrong_method_returns_405() {
    let (app, _dir) = test_app().await;
    let cookie = login(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .method("POST")
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn api_error_not_found_json_shape() {
    let (app, _dir) = test_app().await;
    let cookie = login(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/nonexistent-route-xyz")
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
