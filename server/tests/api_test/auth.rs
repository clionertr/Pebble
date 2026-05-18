use crate::test_app;
use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn unauthenticated_api_returns_401() {
    let (app, _dir) = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn authenticated_api_request_returns_200() {
    let (app, _dir) = test_app().await;

    // Step 1: login to get a session cookie
    let login_response = app
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

    assert_eq!(login_response.status(), StatusCode::OK);
    let cookie = login_response
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Step 2: use cookie to access protected endpoint
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

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn invalid_cookie_returns_401() {
    let (app, _dir) = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .header(header::COOKIE, "pebble_session=invalid-session-id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_correct_password_returns_200_and_cookie() {
    let (app, _dir) = test_app().await;

    let response = app
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

    assert_eq!(response.status(), StatusCode::OK);
    let set_cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        set_cookie.starts_with("pebble_session="),
        "cookie name: {set_cookie}"
    );
    assert!(
        set_cookie.contains("HttpOnly"),
        "HttpOnly flag: {set_cookie}"
    );
    assert!(set_cookie.contains("Secure"), "Secure flag: {set_cookie}");
    assert!(
        set_cookie.contains("SameSite=Strict"),
        "SameSite flag: {set_cookie}"
    );
    assert!(
        set_cookie.contains("Max-Age=604800"),
        "Max-Age: {set_cookie}"
    );
    assert!(set_cookie.contains("Path=/"), "Path: {set_cookie}");
}

#[tokio::test]
async fn login_wrong_password_returns_401() {
    let (app, _dir) = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/auth/login")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"password":"wrong-password"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert!(response.headers().get(header::SET_COOKIE).is_none());
}

#[tokio::test]
async fn login_missing_password_returns_400() {
    let (app, _dir) = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/auth/login")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    // Serde returns 422 for missing required fields; axum returns 400 for malformed bodies
    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn auth_status_without_cookie_returns_false() {
    let (app, _dir) = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/auth/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["authenticated"], false);
}

#[tokio::test]
async fn auth_status_with_valid_cookie_returns_true() {
    let (app, _dir) = test_app().await;

    // Login
    let login_response = app
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
    let cookie = login_response
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Check status
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/auth/status")
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["authenticated"], true);
}

#[tokio::test]
async fn logout_clears_cookie() {
    let (app, _dir) = test_app().await;

    // Login first
    let login_response = app
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
    let cookie = login_response
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Logout
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/auth/logout")
                .method("POST")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let set_cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        set_cookie.contains("Max-Age=0"),
        "cookie should be expired: {set_cookie}"
    );

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

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rate_limit_blocks_after_5_failures() {
    let (app, _dir) = test_app().await;

    // First 5 attempts return 401 (wrong password)
    for i in 0..5 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/auth/login")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"password":"wrong-password"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "attempt {}",
            i + 1
        );
    }

    // 6th attempt is rate limited
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/auth/login")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"password":"wrong-password"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn auth_exempt_routes_are_accessible() {
    let (app, _dir) = test_app().await;

    // /webhook/gmail is exempt from auth (has its own secret verification)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/webhook/gmail")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    // 400 or 401 depending on secret validation, but NOT 401 from auth middleware
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);

    // /auth/login is exempt
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/auth/login?provider=gmail")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Redirect or error from missing config, but NOT 401 from auth middleware
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn events_requires_session_cookie() {
    let (app, _dir) = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
