// GET /api/shell：一次返回账号、各账号文件夹和未读计数，减少首屏请求数。

use axum::{extract::State, response::Json, routing::get, Router};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::state::AppState;

#[derive(Serialize)]
pub struct ShellResponse {
    pub accounts: Vec<pebble_core::Account>,
    pub folders: HashMap<String, Vec<pebble_core::Folder>>,
    #[serde(rename = "unreadCounts")]
    pub unread_counts: HashMap<String, HashMap<String, u32>>,
}

pub fn shell_routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/shell", get(shell_handler))
}

async fn shell_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ShellResponse>, crate::api::error::ApiError> {
    let accounts = crate::rpc::accounts::list_accounts(axum::extract::State(state.clone())).await?;

    let mut folders: HashMap<String, Vec<pebble_core::Folder>> = HashMap::new();
    let mut unread_counts: HashMap<String, HashMap<String, u32>> = HashMap::new();

    for account in &accounts {
        let account_folders = crate::rpc::folders::list_folders(
            axum::extract::State(state.clone()),
            account.id.clone(),
        )
        .await
        .unwrap_or_default();

        let counts = crate::rpc::folder_counts::get_folder_unread_counts(
            axum::extract::State(state.clone()),
            account.id.clone(),
        )
        .await
        .unwrap_or_default();

        folders.insert(account.id.clone(), account_folders);
        unread_counts.insert(account.id.clone(), counts);
    }

    Ok(Json(ShellResponse {
        accounts,
        folders,
        unread_counts,
    }))
}
