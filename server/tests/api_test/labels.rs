use crate::test_app;
use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use pebble_core::{
    new_id, now_timestamp, Account, Folder, FolderRole, FolderType, Message, ProviderType,
};
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

fn seed_message(state: &pebble::state::AppState) -> String {
    let now = now_timestamp();
    let account = Account {
        id: new_id(),
        email: "labels@example.com".to_string(),
        display_name: "Labels".to_string(),
        color: None,
        provider: ProviderType::Imap,
        created_at: now,
        updated_at: now,
    };
    state.store.insert_account(&account).unwrap();

    let folder = Folder {
        id: new_id(),
        account_id: account.id.clone(),
        remote_id: "INBOX".to_string(),
        name: "Inbox".to_string(),
        folder_type: FolderType::Folder,
        role: Some(FolderRole::Inbox),
        parent_id: None,
        color: None,
        is_system: true,
        sort_order: 0,
    };
    state.store.insert_folder(&folder).unwrap();

    let message_id = new_id();
    let message = Message {
        id: message_id.clone(),
        account_id: account.id,
        remote_id: "remote-1".to_string(),
        message_id_header: None,
        in_reply_to: None,
        references_header: None,
        thread_id: None,
        subject: "Label me".to_string(),
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
        .insert_message(&message, std::slice::from_ref(&folder.id))
        .unwrap();
    message_id
}

#[tokio::test]
async fn label_routes_add_list_batch_and_remove_message_labels() {
    let (app, _dir, state) = test_app().await;
    let cookie = login(&app).await;
    let message_id = seed_message(&state);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/messages/{message_id}/labels"))
                .method("POST")
                .header(header::COOKIE, &cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "labelName": "Work" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/messages/{message_id}/labels"))
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
    let labels: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(labels[0]["name"], "Work");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/messages/batch/labels")
                .method("POST")
                .header(header::COOKIE, &cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "messageIds": [message_id.clone()] }).to_string(),
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
                .uri("/api/labels")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/messages/{message_id}/labels/Work"))
                .method("DELETE")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(state
        .store
        .get_message_labels(&message_id)
        .unwrap()
        .is_empty());
}
