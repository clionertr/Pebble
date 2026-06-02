// OpenAPI 文档：/api/docs 显示入口页，/api/docs/openapi.json 返回机器可读契约。

use axum::{response::Html, routing::get, Json, Router};
use serde_json::{json, Value};

pub fn docs_routes<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/docs", get(docs_page))
        .route("/api/docs/openapi.json", get(openapi_spec))
}

async fn docs_page() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>Pebble API Docs</title></head>
<body style="font-family: system-ui, sans-serif; max-width: 720px; margin: 48px auto; line-height: 1.6;">
  <h1>Pebble Webmail API</h1>
  <p>OpenAPI JSON is available at <a href="/api/docs/openapi.json">/api/docs/openapi.json</a>.</p>
</body>
</html>"#,
    )
}

async fn openapi_spec() -> Json<Value> {
    Json(build_spec())
}

fn build_spec() -> Value {
    let mut spec = json!({
        "openapi": "3.0.3",
        "info": {
            "title": "Pebble Webmail API",
            "version": "0.0.10",
            "description": "Single-user self-hosted Webmail REST API. Cookie-based auth via /api/auth/login."
        },
        "servers": [{ "url": "/", "description": "Same-origin" }],
        "paths": {}
    });

    let paths = spec["paths"].as_object_mut().unwrap();

    // Health
    paths.insert("/api/health".into(), json!({
        "get": { "summary": "Health check", "responses": { "200": { "description": "ok" }, "401": { "description": "Unauthorized" } } }
    }));

    paths.insert("/events".into(), json!({
        "get": { "summary": "Server-Sent Events stream", "responses": { "200": { "description": "SSE stream" }, "401": { "description": "Unauthorized" } } }
    }));

    paths.insert("/webhook/gmail".into(), json!({
        "post": { "summary": "Gmail Pub/Sub push webhook", "parameters": [
            { "name": "secret", "in": "query", "required": true, "schema": { "type": "string" } }
        ], "responses": { "200": { "description": "Accepted" }, "400": { "description": "Invalid Pub/Sub payload" }, "401": { "description": "Invalid secret" } } }
    }));

    // Shell
    paths.insert("/api/shell".into(), json!({
        "get": { "summary": "Shell: accounts + folders + unread counts", "responses": { "200": { "description": "Shell data" } } }
    }));

    // Messages
    paths.insert(
        "/api/inbox".into(),
        json!({
            "get": { "summary": "List inbox/thread messages", "parameters": [
                { "name": "accountId", "in": "query", "schema": { "type": "string" } },
                { "name": "folderId", "in": "query", "schema": { "type": "string" } },
                { "name": "limit", "in": "query", "schema": { "type": "integer" } },
                { "name": "offset", "in": "query", "schema": { "type": "integer" } },
                { "name": "folderIds", "in": "query", "schema": { "type": "string" }, "description": "Comma-separated folder IDs" }
            ], "responses": { "200": { "description": "Messages" } } }
        }),
    );

    paths.insert(
        "/api/starred".into(),
        json!({
            "get": { "summary": "List starred messages", "parameters": [
                { "name": "accountId", "in": "query", "schema": { "type": "string" } }
            ], "responses": { "200": { "description": "Starred messages" } } }
        }),
    );

    paths.insert("/api/messages/batch".into(), json!({
        "post": { "summary": "Batch fetch messages", "responses": { "200": { "description": "Messages array" } } }
    }));

    paths.insert(
        "/api/messages/send".into(),
        json!({
            "post": { "summary": "Send email", "responses": { "200": { "description": "Sent" } } }
        }),
    );

    paths.insert("/api/messages/{id}".into(), json!({
        "get": { "summary": "Get a message", "parameters": [
            { "name": "id", "in": "path", "required": true, "schema": { "type": "string" } }
        ], "responses": { "200": { "description": "Message" }, "404": { "description": "Not found" } } },
        "delete": { "summary": "Delete message" }
    }));

    paths.insert(
        "/api/messages/{id}/flags".into(),
        json!({
            "patch": { "summary": "Update message read/starred flags" }
        }),
    );

    paths.insert(
        "/api/messages/{id}/html".into(),
        json!({
            "get": { "summary": "Get rendered HTML for message", "parameters": [
                { "name": "privacyMode", "in": "query", "schema": { "type": "string" } }
            ] }
        }),
    );

    paths.insert(
        "/api/messages/{id}/full".into(),
        json!({
            "get": { "summary": "Get full message with HTML", "parameters": [
                { "name": "privacyMode", "in": "query", "schema": { "type": "string" } }
            ] }
        }),
    );

    paths.insert(
        "/api/messages/{id}/archive".into(),
        json!({
            "post": { "summary": "Archive a message" }
        }),
    );

    paths.insert(
        "/api/messages/{id}/move".into(),
        json!({
            "post": { "summary": "Move message to folder" }
        }),
    );

    paths.insert(
        "/api/messages/{id}/restore".into(),
        json!({
            "post": { "summary": "Restore deleted message" }
        }),
    );

    // Batch mutations
    for (path, summary) in [
        ("/api/messages/batch/archive", "Batch archive"),
        ("/api/messages/batch/delete", "Batch delete"),
        ("/api/messages/batch/read", "Batch mark read"),
        ("/api/messages/batch/star", "Batch star"),
    ] {
        paths.insert(path.into(), json!({ "post": { "summary": summary } }));
    }

    // Auth
    paths.insert("/api/auth/login".into(), json!({
        "post": { "summary": "Login", "responses": { "200": { "description": "Logged in" }, "401": { "description": "Wrong password" }, "429": { "description": "Rate limited" } } }
    }));
    paths.insert(
        "/api/auth/logout".into(),
        json!({
            "post": { "summary": "Logout" }
        }),
    );
    paths.insert(
        "/api/auth/status".into(),
        json!({
            "get": { "summary": "Check auth status" }
        }),
    );

    // Threads
    paths.insert("/api/threads".into(), json!({
        "get": { "summary": "List threads", "parameters": [
            { "name": "folderId", "in": "query", "required": true, "schema": { "type": "string" } },
            { "name": "limit", "in": "query", "schema": { "type": "integer" } },
            { "name": "offset", "in": "query", "schema": { "type": "integer" } },
            { "name": "folderIds", "in": "query", "schema": { "type": "string" }, "description": "Comma-separated folder IDs" }
        ] }
    }));
    paths.insert(
        "/api/threads/{id}/messages".into(),
        json!({
            "get": { "summary": "List thread messages" }
        }),
    );

    // Search
    paths.insert(
        "/api/search".into(),
        json!({
            "get": { "summary": "Search messages", "parameters": [
                { "name": "q", "in": "query", "schema": { "type": "string" } }
            ] }
        }),
    );
    paths.insert(
        "/api/search/advanced".into(),
        json!({
            "post": { "summary": "Advanced search with structured query" }
        }),
    );

    // Kanban
    paths.insert(
        "/api/kanban".into(),
        json!({
            "get": { "summary": "List kanban cards + notes" }
        }),
    );
    paths.insert(
        "/api/kanban/cards".into(),
        json!({
            "post": { "summary": "Move message to kanban" }
        }),
    );
    paths.insert(
        "/api/kanban/notes".into(),
        json!({
            "get": { "summary": "List kanban notes" },
            "patch": { "summary": "Merge kanban notes" }
        }),
    );
    paths.insert(
        "/api/kanban/notes/{id}".into(),
        json!({
            "put": { "summary": "Set kanban note" }
        }),
    );
    paths.insert(
        "/api/kanban/cards/{id}".into(),
        json!({
            "delete": { "summary": "Remove from kanban" }
        }),
    );

    // Snoozed
    paths.insert(
        "/api/snoozed".into(),
        json!({
            "get": { "summary": "List snoozed messages" },
            "post": { "summary": "Snooze a message" }
        }),
    );
    paths.insert(
        "/api/snoozed/{id}".into(),
        json!({
            "delete": { "summary": "Unsnooze a message" }
        }),
    );

    // Pending Ops
    paths.insert(
        "/api/pending-ops".into(),
        json!({
            "get": { "summary": "List pending operations" }
        }),
    );
    paths.insert(
        "/api/pending-ops/summary".into(),
        json!({
            "get": { "summary": "Pending operations summary" }
        }),
    );
    paths.insert(
        "/api/pending-ops/{id}/cancel".into(),
        json!({
            "post": { "summary": "Cancel pending operation" }
        }),
    );
    paths.insert(
        "/api/pending-ops/{id}".into(),
        json!({
            "delete": { "summary": "Delete pending operation" }
        }),
    );

    // Labels
    paths.insert(
        "/api/labels".into(),
        json!({
            "get": { "summary": "List labels" }
        }),
    );
    paths.insert(
        "/api/messages/{id}/labels".into(),
        json!({
            "get": { "summary": "Get message labels" },
            "post": { "summary": "Add label to message" }
        }),
    );
    paths.insert(
        "/api/messages/{id}/labels/{name}".into(),
        json!({
            "delete": { "summary": "Remove label from message" }
        }),
    );
    paths.insert(
        "/api/messages/batch/labels".into(),
        json!({
            "post": { "summary": "Batch get message labels" }
        }),
    );

    // Rules
    paths.insert(
        "/api/rules".into(),
        json!({
            "get": { "summary": "List rules" },
            "post": { "summary": "Create rule" }
        }),
    );
    paths.insert(
        "/api/rules/{id}".into(),
        json!({
            "put": { "summary": "Update rule" },
            "delete": { "summary": "Delete rule" }
        }),
    );

    // Translate
    paths.insert(
        "/api/translate".into(),
        json!({
            "post": { "summary": "Translate text" }
        }),
    );
    paths.insert(
        "/api/translate/stream".into(),
        json!({
            "post": { "summary": "Stream LLM translation deltas" }
        }),
    );
    paths.insert(
        "/api/translate/config".into(),
        json!({
            "get": { "summary": "Get translate config" },
            "put": { "summary": "Save translate config" }
        }),
    );
    paths.insert(
        "/api/translate/test".into(),
        json!({
            "post": { "summary": "Test translate connection" }
        }),
    );

    // Contacts
    paths.insert(
        "/api/contacts".into(),
        json!({
            "get": { "summary": "Search contacts" }
        }),
    );

    // Accounts
    paths.insert(
        "/api/accounts".into(),
        json!({
            "get": { "summary": "List accounts" },
            "post": { "summary": "Add IMAP account" }
        }),
    );
    paths.insert(
        "/api/accounts/{id}".into(),
        json!({
            "patch": { "summary": "Update account" },
            "delete": { "summary": "Delete account" }
        }),
    );
    paths.insert(
        "/api/accounts/{id}/folders".into(),
        json!({
            "get": { "summary": "List account folders" }
        }),
    );
    paths.insert(
        "/api/accounts/{id}/signature".into(),
        json!({
            "get": { "summary": "Get email signature" },
            "put": { "summary": "Set email signature" }
        }),
    );
    paths.insert(
        "/api/accounts/{id}/sync/start".into(),
        json!({
            "post": { "summary": "Start account sync" }
        }),
    );
    paths.insert(
        "/api/accounts/{id}/sync/trigger".into(),
        json!({
            "post": { "summary": "Trigger sync for account" }
        }),
    );
    paths.insert(
        "/api/sync/wake".into(),
        json!({
            "post": {
                "summary": "唤醒一个或多个账号的同步",
                "description": "按需确保选中账号的同步 worker 正在运行，然后为每个账号发送一次实时触发。省略 account_ids 表示全部账号。"
            }
        }),
    );
    paths.insert(
        "/api/accounts/{id}/sync/stop".into(),
        json!({
            "post": { "summary": "Stop account sync" }
        }),
    );
    paths.insert(
        "/api/accounts/{id}/test-connection".into(),
        json!({
            "post": { "summary": "Test account connection" }
        }),
    );
    paths.insert(
        "/api/imap/test-connection".into(),
        json!({
            "post": { "summary": "Test IMAP/SMTP connection before adding account" }
        }),
    );
    paths.insert(
        "/api/accounts/{id}/gmail-realtime".into(),
        json!({
            "get": { "summary": "Get Gmail realtime config" },
            "put": { "summary": "Update Gmail realtime fallback interval" }
        }),
    );
    paths.insert(
        "/api/accounts/{id}/gmail-realtime/enable".into(),
        json!({
            "post": { "summary": "Enable Gmail realtime" }
        }),
    );
    paths.insert(
        "/api/accounts/{id}/gmail-realtime/disable".into(),
        json!({
            "post": { "summary": "Disable Gmail realtime" }
        }),
    );
    paths.insert(
        "/api/accounts/{id}/proxy".into(),
        json!({
            "get": { "summary": "Get account proxy" },
            "put": { "summary": "Set account proxy" }
        }),
    );
    paths.insert(
        "/api/accounts/{id}/proxy-setting".into(),
        json!({
            "get": { "summary": "Get account proxy mode" },
            "put": { "summary": "Set account proxy mode" }
        }),
    );
    paths.insert(
        "/api/accounts/{id}/trash".into(),
        json!({
            "delete": { "summary": "Empty trash" }
        }),
    );

    // Drafts
    paths.insert(
        "/api/drafts".into(),
        json!({
            "post": { "summary": "Save draft" }
        }),
    );
    paths.insert(
        "/api/drafts/{id}".into(),
        json!({
            "delete": { "summary": "Delete draft" }
        }),
    );

    // Attachments
    paths.insert(
        "/api/attachments/stage".into(),
        json!({
            "post": { "summary": "Upload attachment (multipart/form-data)" }
        }),
    );
    paths.insert(
        "/api/attachments/{id}".into(),
        json!({
            "get": { "summary": "Download attachment (streaming)" }
        }),
    );
    paths.insert(
        "/api/messages/{id}/attachments".into(),
        json!({
            "get": { "summary": "List message attachments" }
        }),
    );

    // Templates
    paths.insert(
        "/api/templates".into(),
        json!({
            "get": { "summary": "List templates" },
            "post": { "summary": "Save template" }
        }),
    );
    paths.insert(
        "/api/templates/{id}".into(),
        json!({
            "delete": { "summary": "Delete template" }
        }),
    );

    // Trusted Senders
    paths.insert(
        "/api/trusted-senders".into(),
        json!({
            "get": { "summary": "List trusted senders" },
            "post": { "summary": "Trust a sender" },
            "delete": { "summary": "Remove trusted sender" }
        }),
    );
    paths.insert(
        "/api/trusted-senders/check".into(),
        json!({
            "get": { "summary": "Check if sender is trusted" }
        }),
    );

    // Preferences
    paths.insert(
        "/api/preferences/realtime".into(),
        json!({
            "put": { "summary": "Set realtime preference" }
        }),
    );
    paths.insert(
        "/api/preferences/notifications".into(),
        json!({
            "put": { "summary": "Set notifications preference" }
        }),
    );

    // Diagnostics
    paths.insert(
        "/api/logs".into(),
        json!({
            "get": { "summary": "Read app logs" }
        }),
    );
    paths.insert(
        "/api/diagnostics/mail-timing".into(),
        json!({
            "post": { "summary": "Record mail timing" }
        }),
    );

    // Proxy
    paths.insert(
        "/api/proxy".into(),
        json!({
            "get": { "summary": "Get global proxy config" },
            "put": { "summary": "Set global proxy config" }
        }),
    );

    // Cloud Sync
    for (path, summary) in [
        ("/api/cloud-sync/webdav/test", "Test WebDAV connection"),
        ("/api/cloud-sync/webdav/backup", "Backup to WebDAV"),
        ("/api/cloud-sync/webdav/preview", "Preview WebDAV backup"),
        ("/api/cloud-sync/webdav/restore", "Restore from WebDAV"),
    ] {
        paths.insert(path.into(), json!({ "post": { "summary": summary } }));
    }

    spec
}
