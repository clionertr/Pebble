use crate::test_app;
use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use pebble_core::{new_id, now_timestamp, Account, ProviderType};
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

fn test_imap_account() -> Account {
    let now = now_timestamp();
    Account {
        id: new_id(),
        email: "sender@example.com".to_string(),
        display_name: "Sender".to_string(),
        color: None,
        provider: ProviderType::Imap,
        created_at: now,
        updated_at: now,
    }
}

#[tokio::test]
async fn compose_send_queues_imap_smtp_network_failure() {
    let (app, _dir, state) = test_app().await;
    let cookie = login(&app).await;
    let account = test_imap_account();
    state.store.insert_account(&account).unwrap();

    let auth_config = serde_json::json!({
        "smtp": {
            "host": "127.0.0.1",
            "port": 9,
            "username": "sender@example.com",
            "password": "secret",
            "security": "plain"
        }
    });
    let encrypted = state
        .crypto
        .encrypt(serde_json::to_vec(&auth_config).unwrap().as_slice())
        .unwrap();
    state.store.set_auth_data(&account.id, &encrypted).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/messages/send")
                .method("POST")
                .header(header::COOKIE, &cookie)
                .header("Content-Type", "application/json")
                .body(Body::from(format!(
                    r#"{{
                        "accountId": "{}",
                        "to": ["recipient@example.com"],
                        "subject": "Queued from API",
                        "bodyText": "Plain body from API",
                        "bodyHtml": "<p>HTML body from API</p>"
                    }}"#,
                    account.id
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let pending_ops = state.store.list_pending_mail_ops(&account.id).unwrap();
    assert_eq!(pending_ops.len(), 1);
    assert_eq!(pending_ops[0].op_type, "send");

    let message = state
        .store
        .get_message(&pending_ops[0].message_id)
        .unwrap()
        .expect("queued outgoing message should exist");
    assert_eq!(message.subject, "Queued from API");
    assert_eq!(message.from_address, account.email);
    assert_eq!(message.to_list[0].address, "recipient@example.com");
    assert!(message.remote_id.starts_with("local-outbox-"));

    let outbox = state
        .store
        .find_folder_by_name(&account.id, "Outbox")
        .unwrap()
        .expect("outbox should be created");
    assert_eq!(outbox.role, None);
    let folder_ids = state.store.get_message_folder_ids(&message.id).unwrap();
    assert_eq!(folder_ids, vec![outbox.id]);
}
