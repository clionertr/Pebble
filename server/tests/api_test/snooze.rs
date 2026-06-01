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

fn seed_message(state: &std::sync::Arc<pebble::state::AppState>) -> String {
    let now = pebble_core::now_timestamp();
    let account = pebble_core::Account {
        id: pebble_core::new_id(),
        email: "test@example.com".to_string(),
        display_name: "Test".to_string(),
        color: None,
        provider: pebble_core::ProviderType::Imap,
        created_at: now,
        updated_at: now,
    };
    state.store.insert_account(&account).unwrap();

    let folder = pebble_core::Folder {
        id: pebble_core::new_id(),
        account_id: account.id.clone(),
        remote_id: "INBOX".to_string(),
        name: "Inbox".to_string(),
        folder_type: pebble_core::FolderType::Folder,
        role: Some(pebble_core::FolderRole::Inbox),
        parent_id: None,
        color: None,
        is_system: true,
        sort_order: 0,
    };
    state.store.insert_folder(&folder).unwrap();

    let msg_id = pebble_core::new_id();
    let msg = pebble_core::Message {
        id: msg_id.clone(),
        account_id: account.id,
        remote_id: "1".to_string(),
        message_id_header: None,
        in_reply_to: None,
        references_header: None,
        thread_id: None,
        subject: "Test".to_string(),
        snippet: "Snippet".to_string(),
        from_address: "sender@example.com".to_string(),
        from_name: "Sender".to_string(),
        to_list: vec![],
        cc_list: vec![],
        bcc_list: vec![],
        body_text: "body".to_string(),
        body_html_raw: "<p>body</p>".to_string(),
        has_attachments: false,
        is_read: false,
        is_starred: false,
        is_draft: false,
        date: now,
        remote_version: None,
        is_deleted: false,
        deleted_at: None,
        created_at: now,
        updated_at: now,
    };
    state
        .store
        .insert_message(&msg, std::slice::from_ref(&folder.id))
        .unwrap();
    msg_id
}

#[tokio::test]
async fn snooze_and_unsnooze_message() {
    let (app, _dir, state) = test_app().await;
    let cookie = login(&app).await;
    let msg_id = seed_message(&state);

    // POST snooze
    let body = json!({
        "messageId": msg_id,
        "until": 2000000000_i64,
        "returnTo": "inbox"
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/snoozed")
                .method("POST")
                .header(header::COOKIE, &cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // GET list — should contain our snoozed message
    let list = get_json(&app, &cookie, "/api/snoozed").await;
    let list = list.as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["message_id"], msg_id);
    assert_eq!(list[0]["return_to"], "inbox");

    // DELETE unsnooze
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/snoozed/{}", msg_id))
                .method(Method::DELETE)
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // GET list — should be empty now
    let list = get_json(&app, &cookie, "/api/snoozed").await;
    let list = list.as_array().unwrap();
    assert_eq!(list.len(), 0);
}
