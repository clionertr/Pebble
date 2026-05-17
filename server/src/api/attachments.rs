// Attachment endpoints: list, upload (multipart), download (streaming).

use axum::{
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use crate::state::AppState;

pub fn attachment_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/messages/{id}/attachments", get(list_attachments_handler))
        .route("/api/attachments/stage", post(stage_handler))
        .route("/api/attachments/{id}", get(download_handler))
}

// ── List ──────────────────────────────────────────────────────────────

async fn list_attachments_handler(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
) -> Result<axum::Json<Vec<pebble_core::Attachment>>, crate::api::error::ApiError> {
    let atts = crate::rpc::attachments::list_attachments(
        axum::extract::State(state), message_id,
    ).await?;
    Ok(axum::Json(atts))
}

// ── Upload (multipart) ────────────────────────────────────────────────

async fn stage_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<axum::Json<serde_json::Value>, crate::api::error::ApiError> {
    let mut uploaded = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e|
        crate::api::error::ApiError::bad_request(e.to_string())
    )? {
        let filename = field
            .file_name()
            .unwrap_or("attachment")
            .to_string();
        let data = field.bytes().await.map_err(|e|
            crate::api::error::ApiError::bad_request(e.to_string())
        )?;

        let path = crate::rpc::compose::stage_compose_attachment(
            axum::extract::State(state.clone()),
            filename.clone(),
            data.to_vec(),
        )
        .await?;

        uploaded.push(serde_json::json!({
            "filename": filename,
            "path": path,
            "size": data.len(),
        }));
    }

    Ok(axum::Json(serde_json::json!({ "attachments": uploaded })))
}

// ── Download (streaming) ──────────────────────────────────────────────

async fn download_handler(
    State(state): State<Arc<AppState>>,
    Path(attachment_id): Path<String>,
) -> Result<impl IntoResponse, crate::api::error::ApiError> {
    // Get the local path of the attachment
    let path = crate::rpc::attachments::get_attachment_path(
        axum::extract::State(state.clone()),
        attachment_id.clone(),
    )
    .await?
    .ok_or_else(|| crate::api::error::ApiError::not_found("Attachment not found"))?;

    // Get attachment metadata for filename and MIME
    let att = state.store.get_attachment(&attachment_id)
        .map_err(|e| crate::api::error::ApiError::internal(e.to_string()))?
        .ok_or_else(|| crate::api::error::ApiError::not_found("Attachment not found"))?;

    // Read the file
    let bytes = tokio::fs::read(&path).await
        .map_err(|e| crate::api::error::ApiError::internal(e.to_string()))?;

    let mime = att.mime_type;
    let filename = att.filename;

    let headers = [
        (header::CONTENT_TYPE, mime),
        (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", filename)),
    ];

    Ok((StatusCode::OK, headers, bytes))
}
