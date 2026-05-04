use std::sync::Arc;
use axum::{extract::State, Json, response::IntoResponse};
use serde_json::{Value, json};
use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct RpcRequest {
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

pub async fn handle_rpc_batch(
    state: State<Arc<AppState>>,
    Json(reqs): Json<Vec<RpcRequest>>,
) -> Result<Json<Vec<Value>>, Json<Value>> {
    let mut responses = Vec::with_capacity(reqs.len());
    for req in reqs {
        let res = handle_rpc(state.clone(), Json(req)).await;
        match res {
            Ok(Json(val)) => responses.push(val),
            Err(Json(err)) => responses.push(err),
        }
    }
    Ok(Json(responses))
}

pub async fn handle_rpc(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RpcRequest>,
) -> Result<Json<Value>, Json<Value>> {
    match req.method.as_str() {
        "get_pending_mail_ops_summary" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::pending_mail_ops::get_pending_mail_ops_summary(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?);
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "list_pending_mail_ops" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            #[serde(default)]
            pub limit: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::pending_mail_ops::list_pending_mail_ops(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?, serde_json::from_value(args.limit).map_err(|e| Json(json!({ "error": format!("Invalid arg 'limit': {}", e) })))?);
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "save_draft" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            #[serde(default)]
            pub to: serde_json::Value,
            #[serde(default)]
            pub cc: serde_json::Value,
            #[serde(default)]
            pub bcc: serde_json::Value,
            #[serde(default)]
            pub subject: serde_json::Value,
            #[serde(default)]
            pub body_text: serde_json::Value,
            #[serde(default)]
            pub body_html: serde_json::Value,
            #[serde(default)]
            pub in_reply_to: serde_json::Value,
            #[serde(default)]
            pub attachment_paths: serde_json::Value,
            #[serde(default)]
            pub existing_draft_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::drafts::save_draft(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?, serde_json::from_value(args.to).map_err(|e| Json(json!({ "error": format!("Invalid arg 'to': {}", e) })))?, serde_json::from_value(args.cc).map_err(|e| Json(json!({ "error": format!("Invalid arg 'cc': {}", e) })))?, serde_json::from_value(args.bcc).map_err(|e| Json(json!({ "error": format!("Invalid arg 'bcc': {}", e) })))?, serde_json::from_value(args.subject).map_err(|e| Json(json!({ "error": format!("Invalid arg 'subject': {}", e) })))?, serde_json::from_value(args.body_text).map_err(|e| Json(json!({ "error": format!("Invalid arg 'body_text': {}", e) })))?, serde_json::from_value(args.body_html).map_err(|e| Json(json!({ "error": format!("Invalid arg 'body_html': {}", e) })))?, serde_json::from_value(args.in_reply_to).map_err(|e| Json(json!({ "error": format!("Invalid arg 'in_reply_to': {}", e) })))?, serde_json::from_value(args.attachment_paths).map_err(|e| Json(json!({ "error": format!("Invalid arg 'attachment_paths': {}", e) })))?, serde_json::from_value(args.existing_draft_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'existing_draft_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "delete_draft" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            #[serde(default)]
            pub draft_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::drafts::delete_draft(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?, serde_json::from_value(args.draft_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'draft_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "get_message_labels" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::labels::get_message_labels(axum::extract::State(state.clone()), serde_json::from_value(args.message_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "get_message_labels_batch" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_ids: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::labels::get_message_labels_batch(axum::extract::State(state.clone()), serde_json::from_value(args.message_ids).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_ids': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "add_message_label" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_id: serde_json::Value,
            #[serde(default)]
            pub label_name: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::labels::add_message_label(axum::extract::State(state.clone()), serde_json::from_value(args.message_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_id': {}", e) })))?, serde_json::from_value(args.label_name).map_err(|e| Json(json!({ "error": format!("Invalid arg 'label_name': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "remove_message_label" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_id: serde_json::Value,
            #[serde(default)]
            pub label_name: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::labels::remove_message_label(axum::extract::State(state.clone()), serde_json::from_value(args.message_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_id': {}", e) })))?, serde_json::from_value(args.label_name).map_err(|e| Json(json!({ "error": format!("Invalid arg 'label_name': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "list_labels" => {
            let res = crate::rpc::labels::list_labels(axum::extract::State(state.clone())).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "translate_text" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub text: serde_json::Value,
            #[serde(default)]
            pub from_lang: serde_json::Value,
            #[serde(default)]
            pub to_lang: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::translate::translate_text(axum::extract::State(state.clone()), serde_json::from_value(args.text).map_err(|e| Json(json!({ "error": format!("Invalid arg 'text': {}", e) })))?, serde_json::from_value(args.from_lang).map_err(|e| Json(json!({ "error": format!("Invalid arg 'from_lang': {}", e) })))?, serde_json::from_value(args.to_lang).map_err(|e| Json(json!({ "error": format!("Invalid arg 'to_lang': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "get_translate_config" => {
            let res = crate::rpc::translate::get_translate_config(axum::extract::State(state.clone())).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "save_translate_config" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub provider_type: serde_json::Value,
            #[serde(default)]
            pub config: serde_json::Value,
            #[serde(default)]
            pub is_enabled: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::translate::save_translate_config(axum::extract::State(state.clone()), serde_json::from_value(args.provider_type).map_err(|e| Json(json!({ "error": format!("Invalid arg 'provider_type': {}", e) })))?, serde_json::from_value(args.config).map_err(|e| Json(json!({ "error": format!("Invalid arg 'config': {}", e) })))?, serde_json::from_value(args.is_enabled).map_err(|e| Json(json!({ "error": format!("Invalid arg 'is_enabled': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "test_translate_connection" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub config: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::translate::test_translate_connection(serde_json::from_value(args.config).map_err(|e| Json(json!({ "error": format!("Invalid arg 'config': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "test_webdav_connection" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub url: serde_json::Value,
            #[serde(default)]
            pub username: serde_json::Value,
            #[serde(default)]
            pub password: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::cloud_sync::test_webdav_connection(serde_json::from_value(args.url).map_err(|e| Json(json!({ "error": format!("Invalid arg 'url': {}", e) })))?, serde_json::from_value(args.username).map_err(|e| Json(json!({ "error": format!("Invalid arg 'username': {}", e) })))?, serde_json::from_value(args.password).map_err(|e| Json(json!({ "error": format!("Invalid arg 'password': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "backup_to_webdav" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub url: serde_json::Value,
            #[serde(default)]
            pub username: serde_json::Value,
            #[serde(default)]
            pub password: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::cloud_sync::backup_to_webdav(axum::extract::State(state.clone()), serde_json::from_value(args.url).map_err(|e| Json(json!({ "error": format!("Invalid arg 'url': {}", e) })))?, serde_json::from_value(args.username).map_err(|e| Json(json!({ "error": format!("Invalid arg 'username': {}", e) })))?, serde_json::from_value(args.password).map_err(|e| Json(json!({ "error": format!("Invalid arg 'password': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "preview_webdav_backup" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub url: serde_json::Value,
            #[serde(default)]
            pub username: serde_json::Value,
            #[serde(default)]
            pub password: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::cloud_sync::preview_webdav_backup(serde_json::from_value(args.url).map_err(|e| Json(json!({ "error": format!("Invalid arg 'url': {}", e) })))?, serde_json::from_value(args.username).map_err(|e| Json(json!({ "error": format!("Invalid arg 'username': {}", e) })))?, serde_json::from_value(args.password).map_err(|e| Json(json!({ "error": format!("Invalid arg 'password': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "restore_from_webdav" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub url: serde_json::Value,
            #[serde(default)]
            pub username: serde_json::Value,
            #[serde(default)]
            pub password: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::cloud_sync::restore_from_webdav(axum::extract::State(state.clone()), serde_json::from_value(args.url).map_err(|e| Json(json!({ "error": format!("Invalid arg 'url': {}", e) })))?, serde_json::from_value(args.username).map_err(|e| Json(json!({ "error": format!("Invalid arg 'username': {}", e) })))?, serde_json::from_value(args.password).map_err(|e| Json(json!({ "error": format!("Invalid arg 'password': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "send_email" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            #[serde(default)]
            pub to: serde_json::Value,
            #[serde(default)]
            pub cc: serde_json::Value,
            #[serde(default)]
            pub bcc: serde_json::Value,
            #[serde(default)]
            pub subject: serde_json::Value,
            #[serde(default)]
            pub body_text: serde_json::Value,
            #[serde(default)]
            pub body_html: serde_json::Value,
            #[serde(default)]
            pub in_reply_to: serde_json::Value,
            #[serde(default)]
            pub attachment_paths: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::compose::send_email(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?, serde_json::from_value(args.to).map_err(|e| Json(json!({ "error": format!("Invalid arg 'to': {}", e) })))?, serde_json::from_value(args.cc).map_err(|e| Json(json!({ "error": format!("Invalid arg 'cc': {}", e) })))?, serde_json::from_value(args.bcc).map_err(|e| Json(json!({ "error": format!("Invalid arg 'bcc': {}", e) })))?, serde_json::from_value(args.subject).map_err(|e| Json(json!({ "error": format!("Invalid arg 'subject': {}", e) })))?, serde_json::from_value(args.body_text).map_err(|e| Json(json!({ "error": format!("Invalid arg 'body_text': {}", e) })))?, serde_json::from_value(args.body_html).map_err(|e| Json(json!({ "error": format!("Invalid arg 'body_html': {}", e) })))?, serde_json::from_value(args.in_reply_to).map_err(|e| Json(json!({ "error": format!("Invalid arg 'in_reply_to': {}", e) })))?, serde_json::from_value(args.attachment_paths).map_err(|e| Json(json!({ "error": format!("Invalid arg 'attachment_paths': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "stage_compose_attachment" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub filename: serde_json::Value,
            #[serde(default)]
            pub bytes: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::compose::stage_compose_attachment(axum::extract::State(state.clone()), serde_json::from_value(args.filename).map_err(|e| Json(json!({ "error": format!("Invalid arg 'filename': {}", e) })))?, serde_json::from_value(args.bytes).map_err(|e| Json(json!({ "error": format!("Invalid arg 'bytes': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "search_contacts" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            #[serde(default)]
            pub query: serde_json::Value,
            #[serde(default)]
            pub limit: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::contacts::search_contacts(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?, serde_json::from_value(args.query).map_err(|e| Json(json!({ "error": format!("Invalid arg 'query': {}", e) })))?, serde_json::from_value(args.limit).map_err(|e| Json(json!({ "error": format!("Invalid arg 'limit': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "batch_archive" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_ids: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::batch::batch_archive(axum::extract::State(state.clone()), serde_json::from_value(args.message_ids).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_ids': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "batch_delete" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_ids: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::batch::batch_delete(axum::extract::State(state.clone()), serde_json::from_value(args.message_ids).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_ids': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "batch_mark_read" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_ids: serde_json::Value,
            #[serde(default)]
            pub is_read: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::batch::batch_mark_read(axum::extract::State(state.clone()), serde_json::from_value(args.message_ids).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_ids': {}", e) })))?, serde_json::from_value(args.is_read).map_err(|e| Json(json!({ "error": format!("Invalid arg 'is_read': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "batch_star" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_ids: serde_json::Value,
            #[serde(default)]
            pub starred: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::batch::batch_star(axum::extract::State(state.clone()), serde_json::from_value(args.message_ids).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_ids': {}", e) })))?, serde_json::from_value(args.starred).map_err(|e| Json(json!({ "error": format!("Invalid arg 'starred': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "add_account" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub request: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::accounts::add_account(axum::extract::State(state.clone()), serde_json::from_value(args.request).map_err(|e| Json(json!({ "error": format!("Invalid arg 'request': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "update_account" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            #[serde(default)]
            pub email: serde_json::Value,
            #[serde(default)]
            pub display_name: serde_json::Value,
            #[serde(default)]
            pub password: serde_json::Value,
            #[serde(default)]
            pub imap_host: serde_json::Value,
            #[serde(default)]
            pub imap_port: serde_json::Value,
            #[serde(default)]
            pub smtp_host: serde_json::Value,
            #[serde(default)]
            pub smtp_port: serde_json::Value,
            #[serde(default)]
            pub imap_security: serde_json::Value,
            #[serde(default)]
            pub smtp_security: serde_json::Value,
            #[serde(default)]
            pub proxy_host: serde_json::Value,
            #[serde(default)]
            pub proxy_port: serde_json::Value,
            #[serde(default)]
            pub account_color: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::accounts::update_account(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?, serde_json::from_value(args.email).map_err(|e| Json(json!({ "error": format!("Invalid arg 'email': {}", e) })))?, serde_json::from_value(args.display_name).map_err(|e| Json(json!({ "error": format!("Invalid arg 'display_name': {}", e) })))?, serde_json::from_value(args.password).map_err(|e| Json(json!({ "error": format!("Invalid arg 'password': {}", e) })))?, serde_json::from_value(args.imap_host).map_err(|e| Json(json!({ "error": format!("Invalid arg 'imap_host': {}", e) })))?, serde_json::from_value(args.imap_port).map_err(|e| Json(json!({ "error": format!("Invalid arg 'imap_port': {}", e) })))?, serde_json::from_value(args.smtp_host).map_err(|e| Json(json!({ "error": format!("Invalid arg 'smtp_host': {}", e) })))?, serde_json::from_value(args.smtp_port).map_err(|e| Json(json!({ "error": format!("Invalid arg 'smtp_port': {}", e) })))?, serde_json::from_value(args.imap_security).map_err(|e| Json(json!({ "error": format!("Invalid arg 'imap_security': {}", e) })))?, serde_json::from_value(args.smtp_security).map_err(|e| Json(json!({ "error": format!("Invalid arg 'smtp_security': {}", e) })))?, serde_json::from_value(args.proxy_host).map_err(|e| Json(json!({ "error": format!("Invalid arg 'proxy_host': {}", e) })))?, serde_json::from_value(args.proxy_port).map_err(|e| Json(json!({ "error": format!("Invalid arg 'proxy_port': {}", e) })))?, serde_json::from_value(args.account_color).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_color': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "get_account_proxy" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::accounts::get_account_proxy(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "get_account_proxy_setting" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::accounts::get_account_proxy_setting(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "update_account_proxy" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            #[serde(default)]
            pub proxy_host: serde_json::Value,
            #[serde(default)]
            pub proxy_port: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::accounts::update_account_proxy(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?, serde_json::from_value(args.proxy_host).map_err(|e| Json(json!({ "error": format!("Invalid arg 'proxy_host': {}", e) })))?, serde_json::from_value(args.proxy_port).map_err(|e| Json(json!({ "error": format!("Invalid arg 'proxy_port': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "update_account_proxy_setting" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            #[serde(default)]
            pub mode: serde_json::Value,
            #[serde(default)]
            pub proxy_host: serde_json::Value,
            #[serde(default)]
            pub proxy_port: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::accounts::update_account_proxy_setting(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?, serde_json::from_value(args.mode).map_err(|e| Json(json!({ "error": format!("Invalid arg 'mode': {}", e) })))?, serde_json::from_value(args.proxy_host).map_err(|e| Json(json!({ "error": format!("Invalid arg 'proxy_host': {}", e) })))?, serde_json::from_value(args.proxy_port).map_err(|e| Json(json!({ "error": format!("Invalid arg 'proxy_port': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "test_imap_connection" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub request: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::accounts::test_imap_connection(axum::extract::State(state.clone()), serde_json::from_value(args.request).map_err(|e| Json(json!({ "error": format!("Invalid arg 'request': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "test_account_connection" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::accounts::test_account_connection(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "list_accounts" => {
            let res = crate::rpc::accounts::list_accounts(axum::extract::State(state.clone())).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "delete_account" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::accounts::delete_account(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "get_folder_unread_counts" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::folder_counts::get_folder_unread_counts(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "start_sync" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            #[serde(default)]
            pub poll_interval_secs: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::sync_cmd::start_sync(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?, serde_json::from_value(args.poll_interval_secs).map_err(|e| Json(json!({ "error": format!("Invalid arg 'poll_interval_secs': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "trigger_sync" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            #[serde(default)]
            pub reason: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::sync_cmd::trigger_sync(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?, serde_json::from_value(args.reason).map_err(|e| Json(json!({ "error": format!("Invalid arg 'reason': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "stop_sync" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::sync_cmd::stop_sync(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "set_realtime_preference" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub mode: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::sync_cmd::set_realtime_preference(axum::extract::State(state.clone()), serde_json::from_value(args.mode).map_err(|e| Json(json!({ "error": format!("Invalid arg 'mode': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "reindex_search" => {
            let res = crate::rpc::sync_cmd::reindex_search(axum::extract::State(state.clone())).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "set_notifications_enabled" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub enabled: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::notifications::set_notifications_enabled(axum::extract::State(state.clone()), serde_json::from_value(args.enabled).map_err(|e| Json(json!({ "error": format!("Invalid arg 'enabled': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "trust_sender" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            #[serde(default)]
            pub email: serde_json::Value,
            #[serde(default)]
            pub trust_type: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::trusted_senders::trust_sender(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?, serde_json::from_value(args.email).map_err(|e| Json(json!({ "error": format!("Invalid arg 'email': {}", e) })))?, serde_json::from_value(args.trust_type).map_err(|e| Json(json!({ "error": format!("Invalid arg 'trust_type': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "list_trusted_senders" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::trusted_senders::list_trusted_senders(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "remove_trusted_sender" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            #[serde(default)]
            pub email: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::trusted_senders::remove_trusted_sender(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?, serde_json::from_value(args.email).map_err(|e| Json(json!({ "error": format!("Invalid arg 'email': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "complete_oauth_flow" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub provider: serde_json::Value,
            #[serde(default)]
            pub email: serde_json::Value,
            #[serde(default)]
            pub display_name: serde_json::Value,
            #[serde(default)]
            pub proxy_host: serde_json::Value,
            #[serde(default)]
            pub proxy_port: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::oauth::complete_oauth_flow(axum::extract::State(state.clone()), serde_json::from_value(args.provider).map_err(|e| Json(json!({ "error": format!("Invalid arg 'provider': {}", e) })))?, serde_json::from_value(args.email).map_err(|e| Json(json!({ "error": format!("Invalid arg 'email': {}", e) })))?, serde_json::from_value(args.display_name).map_err(|e| Json(json!({ "error": format!("Invalid arg 'display_name': {}", e) })))?, serde_json::from_value(args.proxy_host).map_err(|e| Json(json!({ "error": format!("Invalid arg 'proxy_host': {}", e) })))?, serde_json::from_value(args.proxy_port).map_err(|e| Json(json!({ "error": format!("Invalid arg 'proxy_port': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "get_oauth_account_proxy" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::oauth::get_oauth_account_proxy(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "get_oauth_account_proxy_setting" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::oauth::get_oauth_account_proxy_setting(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "update_oauth_account_proxy" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            #[serde(default)]
            pub proxy_host: serde_json::Value,
            #[serde(default)]
            pub proxy_port: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::oauth::update_oauth_account_proxy(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?, serde_json::from_value(args.proxy_host).map_err(|e| Json(json!({ "error": format!("Invalid arg 'proxy_host': {}", e) })))?, serde_json::from_value(args.proxy_port).map_err(|e| Json(json!({ "error": format!("Invalid arg 'proxy_port': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "update_oauth_account_proxy_setting" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            #[serde(default)]
            pub mode: serde_json::Value,
            #[serde(default)]
            pub proxy_host: serde_json::Value,
            #[serde(default)]
            pub proxy_port: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::oauth::update_oauth_account_proxy_setting(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?, serde_json::from_value(args.mode).map_err(|e| Json(json!({ "error": format!("Invalid arg 'mode': {}", e) })))?, serde_json::from_value(args.proxy_host).map_err(|e| Json(json!({ "error": format!("Invalid arg 'proxy_host': {}", e) })))?, serde_json::from_value(args.proxy_port).map_err(|e| Json(json!({ "error": format!("Invalid arg 'proxy_port': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "search_messages" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub query: serde_json::Value,
            #[serde(default)]
            pub limit: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::search::search_messages(axum::extract::State(state.clone()), serde_json::from_value(args.query).map_err(|e| Json(json!({ "error": format!("Invalid arg 'query': {}", e) })))?, serde_json::from_value(args.limit).map_err(|e| Json(json!({ "error": format!("Invalid arg 'limit': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "create_rule" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub name: serde_json::Value,
            #[serde(default)]
            pub priority: serde_json::Value,
            #[serde(default)]
            pub conditions: serde_json::Value,
            #[serde(default)]
            pub actions: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::rules::create_rule(axum::extract::State(state.clone()), serde_json::from_value(args.name).map_err(|e| Json(json!({ "error": format!("Invalid arg 'name': {}", e) })))?, serde_json::from_value(args.priority).map_err(|e| Json(json!({ "error": format!("Invalid arg 'priority': {}", e) })))?, serde_json::from_value(args.conditions).map_err(|e| Json(json!({ "error": format!("Invalid arg 'conditions': {}", e) })))?, serde_json::from_value(args.actions).map_err(|e| Json(json!({ "error": format!("Invalid arg 'actions': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "list_rules" => {
            let res = crate::rpc::rules::list_rules(axum::extract::State(state.clone())).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "update_rule" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub rule: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::rules::update_rule(axum::extract::State(state.clone()), serde_json::from_value(args.rule).map_err(|e| Json(json!({ "error": format!("Invalid arg 'rule': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "delete_rule" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub rule_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::rules::delete_rule(axum::extract::State(state.clone()), serde_json::from_value(args.rule_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'rule_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "advanced_search" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub query: serde_json::Value,
            #[serde(default)]
            pub limit: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::advanced_search::advanced_search(axum::extract::State(state.clone()), serde_json::from_value(args.query).map_err(|e| Json(json!({ "error": format!("Invalid arg 'query': {}", e) })))?, serde_json::from_value(args.limit).map_err(|e| Json(json!({ "error": format!("Invalid arg 'limit': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "move_to_kanban" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_id: serde_json::Value,
            #[serde(default)]
            pub column: serde_json::Value,
            #[serde(default)]
            pub position: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::kanban::move_to_kanban(axum::extract::State(state.clone()), serde_json::from_value(args.message_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_id': {}", e) })))?, serde_json::from_value(args.column).map_err(|e| Json(json!({ "error": format!("Invalid arg 'column': {}", e) })))?, serde_json::from_value(args.position).map_err(|e| Json(json!({ "error": format!("Invalid arg 'position': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "list_kanban_cards" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub column: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::kanban::list_kanban_cards(axum::extract::State(state.clone()), serde_json::from_value(args.column).map_err(|e| Json(json!({ "error": format!("Invalid arg 'column': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "remove_from_kanban" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::kanban::remove_from_kanban(axum::extract::State(state.clone()), serde_json::from_value(args.message_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "list_kanban_context_notes" => {
            let res = crate::rpc::kanban::list_kanban_context_notes(axum::extract::State(state.clone())).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "set_kanban_context_note" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_id: serde_json::Value,
            #[serde(default)]
            pub note: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::kanban::set_kanban_context_note(axum::extract::State(state.clone()), serde_json::from_value(args.message_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_id': {}", e) })))?, serde_json::from_value(args.note).map_err(|e| Json(json!({ "error": format!("Invalid arg 'note': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "merge_kanban_context_notes" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub notes: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::kanban::merge_kanban_context_notes(axum::extract::State(state.clone()), serde_json::from_value(args.notes).map_err(|e| Json(json!({ "error": format!("Invalid arg 'notes': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "read_app_log" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub max_bytes: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::diagnostics::read_app_log(serde_json::from_value(args.max_bytes).map_err(|e| Json(json!({ "error": format!("Invalid arg 'max_bytes': {}", e) })))?);
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "list_thread_messages" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub thread_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::threads::list_thread_messages(axum::extract::State(state.clone()), serde_json::from_value(args.thread_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'thread_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "list_threads" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub folder_id: serde_json::Value,
            #[serde(default)]
            pub folder_ids: serde_json::Value,
            #[serde(default)]
            pub limit: serde_json::Value,
            #[serde(default)]
            pub offset: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::threads::list_threads(axum::extract::State(state.clone()), serde_json::from_value(args.folder_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'folder_id': {}", e) })))?, serde_json::from_value(args.folder_ids).map_err(|e| Json(json!({ "error": format!("Invalid arg 'folder_ids': {}", e) })))?, serde_json::from_value(args.limit).map_err(|e| Json(json!({ "error": format!("Invalid arg 'limit': {}", e) })))?, serde_json::from_value(args.offset).map_err(|e| Json(json!({ "error": format!("Invalid arg 'offset': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "list_email_templates" => {
            let res = crate::rpc::user_data::list_email_templates(axum::extract::State(state.clone())).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "save_email_template" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub template: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::user_data::save_email_template(axum::extract::State(state.clone()), serde_json::from_value(args.template).map_err(|e| Json(json!({ "error": format!("Invalid arg 'template': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "delete_email_template" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::user_data::delete_email_template(axum::extract::State(state.clone()), serde_json::from_value(args.id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "get_email_signature" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::user_data::get_email_signature(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "set_email_signature" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            #[serde(default)]
            pub signature: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::user_data::set_email_signature(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?, serde_json::from_value(args.signature).map_err(|e| Json(json!({ "error": format!("Invalid arg 'signature': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "list_folders" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::folders::list_folders(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "health_check" => {
            let res = crate::rpc::health::health_check(axum::extract::State(state.clone()));
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "snooze_message" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_id: serde_json::Value,
            #[serde(default)]
            pub until: serde_json::Value,
            #[serde(default)]
            pub return_to: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::snooze::snooze_message(axum::extract::State(state.clone()), serde_json::from_value(args.message_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_id': {}", e) })))?, serde_json::from_value(args.until).map_err(|e| Json(json!({ "error": format!("Invalid arg 'until': {}", e) })))?, serde_json::from_value(args.return_to).map_err(|e| Json(json!({ "error": format!("Invalid arg 'return_to': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "unsnooze_message" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::snooze::unsnooze_message(axum::extract::State(state.clone()), serde_json::from_value(args.message_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "list_snoozed" => {
            let res = crate::rpc::snooze::list_snoozed(axum::extract::State(state.clone())).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "list_attachments" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::attachments::list_attachments(axum::extract::State(state.clone()), serde_json::from_value(args.message_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "get_attachment_path" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub attachment_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::attachments::get_attachment_path(axum::extract::State(state.clone()), serde_json::from_value(args.attachment_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'attachment_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "download_attachment" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub attachment_id: serde_json::Value,
            #[serde(default)]
            pub save_to: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::attachments::download_attachment(axum::extract::State(state.clone()), serde_json::from_value(args.attachment_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'attachment_id': {}", e) })))?, serde_json::from_value(args.save_to).map_err(|e| Json(json!({ "error": format!("Invalid arg 'save_to': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "get_global_proxy" => {
            let res = crate::rpc::network::get_global_proxy(axum::extract::State(state.clone())).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "update_global_proxy" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub proxy_host: serde_json::Value,
            #[serde(default)]
            pub proxy_port: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::network::update_global_proxy(axum::extract::State(state.clone()), serde_json::from_value(args.proxy_host).map_err(|e| Json(json!({ "error": format!("Invalid arg 'proxy_host': {}", e) })))?, serde_json::from_value(args.proxy_port).map_err(|e| Json(json!({ "error": format!("Invalid arg 'proxy_port': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "get_rendered_html" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_id: serde_json::Value,
            #[serde(default)]
            pub privacy_mode: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::messages::rendering::get_rendered_html(axum::extract::State(state.clone()), serde_json::from_value(args.message_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_id': {}", e) })))?, serde_json::from_value(args.privacy_mode).map_err(|e| Json(json!({ "error": format!("Invalid arg 'privacy_mode': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "get_message_with_html" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_id: serde_json::Value,
            #[serde(default)]
            pub privacy_mode: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::messages::rendering::get_message_with_html(axum::extract::State(state.clone()), serde_json::from_value(args.message_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_id': {}", e) })))?, serde_json::from_value(args.privacy_mode).map_err(|e| Json(json!({ "error": format!("Invalid arg 'privacy_mode': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "is_trusted_sender" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            #[serde(default)]
            pub email: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::messages::rendering::is_trusted_sender(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?, serde_json::from_value(args.email).map_err(|e| Json(json!({ "error": format!("Invalid arg 'email': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "list_messages" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub folder_id: serde_json::Value,
            #[serde(default)]
            pub folder_ids: serde_json::Value,
            #[serde(default)]
            pub limit: serde_json::Value,
            #[serde(default)]
            pub offset: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::messages::query::list_messages(axum::extract::State(state.clone()), serde_json::from_value(args.folder_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'folder_id': {}", e) })))?, serde_json::from_value(args.folder_ids).map_err(|e| Json(json!({ "error": format!("Invalid arg 'folder_ids': {}", e) })))?, serde_json::from_value(args.limit).map_err(|e| Json(json!({ "error": format!("Invalid arg 'limit': {}", e) })))?, serde_json::from_value(args.offset).map_err(|e| Json(json!({ "error": format!("Invalid arg 'offset': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "list_starred_messages" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            #[serde(default)]
            pub limit: serde_json::Value,
            #[serde(default)]
            pub offset: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::messages::query::list_starred_messages(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?, serde_json::from_value(args.limit).map_err(|e| Json(json!({ "error": format!("Invalid arg 'limit': {}", e) })))?, serde_json::from_value(args.offset).map_err(|e| Json(json!({ "error": format!("Invalid arg 'offset': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "get_message" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::messages::query::get_message(axum::extract::State(state.clone()), serde_json::from_value(args.message_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "get_messages_batch" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_ids: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::messages::query::get_messages_batch(axum::extract::State(state.clone()), serde_json::from_value(args.message_ids).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_ids': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "archive_message" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::messages::lifecycle::archive_message(axum::extract::State(state.clone()), serde_json::from_value(args.message_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "delete_message" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::messages::lifecycle::delete_message(axum::extract::State(state.clone()), serde_json::from_value(args.message_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "restore_message" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::messages::lifecycle::restore_message(axum::extract::State(state.clone()), serde_json::from_value(args.message_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "move_to_folder" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_id: serde_json::Value,
            #[serde(default)]
            pub target_folder_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::messages::lifecycle::move_to_folder(axum::extract::State(state.clone()), serde_json::from_value(args.message_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_id': {}", e) })))?, serde_json::from_value(args.target_folder_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'target_folder_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "empty_trash" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub account_id: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::messages::lifecycle::empty_trash(axum::extract::State(state.clone()), serde_json::from_value(args.account_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'account_id': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        "update_message_flags" => {
            #[derive(serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
            #[serde(default)]
            pub message_id: serde_json::Value,
            #[serde(default)]
            pub is_read: serde_json::Value,
            #[serde(default)]
            pub is_starred: serde_json::Value,
            }
            let args: Args = serde_json::from_value(req.params).map_err(|e| Json(json!({ "error": e.to_string() })))?;
            let res = crate::rpc::messages::flags::update_message_flags(axum::extract::State(state.clone()), serde_json::from_value(args.message_id).map_err(|e| Json(json!({ "error": format!("Invalid arg 'message_id': {}", e) })))?, serde_json::from_value(args.is_read).map_err(|e| Json(json!({ "error": format!("Invalid arg 'is_read': {}", e) })))?, serde_json::from_value(args.is_starred).map_err(|e| Json(json!({ "error": format!("Invalid arg 'is_starred': {}", e) })))?).await;
            match res {
                Ok(val) => Ok(Json(serde_json::to_value(val).unwrap_or(Value::Null))),
                Err(err) => Err(Json(json!({ "error": err.to_string() }))),
            }
        }
        _ => Err(Json(json!({ "error": format!("Unknown method: {}", req.method) }))),
    }
}