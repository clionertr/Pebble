use crate::test_app;
use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
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

async fn add_account(app: &Router, cookie: &str, email: &str) -> String {
    let body = json!({
        "email": email,
        "display_name": email,
        "provider": "imap",
        "imap_host": "localhost",
        "imap_port": 993,
        "smtp_host": "localhost",
        "smtp_port": 587,
        "username": email,
        "password": "password",
        "imap_security": "plain",
        "smtp_security": "plain"
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/accounts")
                .method("POST")
                .header(header::COOKIE, cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 65536)
        .await
        .unwrap();
    serde_json::from_slice::<serde_json::Value>(&body).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn trust_sender(app: &Router, cookie: &str, account_id: &str, email: &str, trust_type: &str) {
    let body = json!({
        "accountId": account_id,
        "email": email,
        "trustType": trust_type
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/trusted-senders")
                .method("POST")
                .header(header::COOKIE, cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
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
async fn trusted_senders_list_all_and_delete_by_account_email() {
    let (app, _dir, _state) = test_app().await;
    let cookie = login(&app).await;
    let account1 = add_account(&app, &cookie, "me@example.com").await;
    let account2 = add_account(&app, &cookie, "work@example.com").await;

    trust_sender(&app, &cookie, &account1, "shared@example.com", "all").await;
    trust_sender(&app, &cookie, &account2, "shared@example.com", "images").await;

    let all = get_json(&app, &cookie, "/api/trusted-senders").await;
    assert_eq!(all.as_array().unwrap().len(), 2);

    let account1_list = get_json(
        &app,
        &cookie,
        &format!("/api/trusted-senders?accountId={account1}"),
    )
    .await;
    assert_eq!(account1_list.as_array().unwrap().len(), 1);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/trusted-senders?accountId={account1}&email=shared%40example.com"
                ))
                .method(Method::DELETE)
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let account1_list = get_json(
        &app,
        &cookie,
        &format!("/api/trusted-senders?accountId={account1}"),
    )
    .await;
    assert_eq!(account1_list.as_array().unwrap().len(), 0);

    let all = get_json(&app, &cookie, "/api/trusted-senders").await;
    let all = all.as_array().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0]["account_id"].as_str().unwrap(), account2);
}
