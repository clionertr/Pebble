use crate::test_app;
use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use pebble_core::{new_id, now_timestamp, Account, ProviderType};
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

fn test_account() -> Account {
    let now = now_timestamp();
    Account {
        id: new_id(),
        email: "drafts@example.com".to_string(),
        display_name: "Draft Tester".to_string(),
        color: None,
        provider: ProviderType::Imap,
        created_at: now,
        updated_at: now,
    }
}

#[tokio::test]
async fn draft_save_and_delete_routes_persist_local_imap_draft() {
    let (app, _dir, state) = test_app().await;
    let cookie = login(&app).await;
    let account = test_account();
    state.store.insert_account(&account).unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/drafts")
                .method("POST")
                .header(header::COOKIE, &cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "accountId": account.id,
                        "to": ["recipient@example.com"],
                        "subject": "Saved draft",
                        "bodyText": "Draft body",
                        "bodyHtml": "<p>Draft body</p>"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 65536)
        .await
        .unwrap();
    let draft_id: String = serde_json::from_slice(&body).unwrap();
    let stored = state
        .store
        .get_message(&draft_id)
        .unwrap()
        .expect("draft should be stored locally");
    assert!(stored.is_draft);
    assert_eq!(stored.subject, "Saved draft");

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/drafts/{draft_id}?accountId={}", account.id))
                .method("DELETE")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(state.store.get_message(&draft_id).unwrap().is_none());
}
