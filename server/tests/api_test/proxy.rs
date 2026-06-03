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
async fn global_proxy_route_persists_and_returns_proxy_config() {
    let (app, _dir, _state) = test_app().await;
    let cookie = login(&app).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/proxy")
                .method("PUT")
                .header(header::COOKIE, &cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "proxy_host": " 127.0.0.1 ", "proxy_port": 7890 }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/proxy")
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
    let proxy: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(proxy["host"], "127.0.0.1");
    assert_eq!(proxy["port"], 7890);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/proxy")
                .method("PUT")
                .header(header::COOKIE, &cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "proxy_host": "127.0.0.1" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
