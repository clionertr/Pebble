use semver::Version;
use serde::Serialize;

use pebble_core::PebbleError;

#[derive(Serialize)]
pub struct UpdateInfo {
    pub latest_version: String,
    pub release_url: String,
    pub is_newer: bool,
}

pub async fn check_for_update(current_version: String) -> Result<UpdateInfo, PebbleError> {
    let client = reqwest::Client::builder()
        .user_agent("Pebble-Email-Client")
        .build()
        .map_err(|e| PebbleError::Network(format!("Failed to create HTTP client: {e}")))?;

    let resp = client
        .get("https://api.github.com/repos/clionertr/Pebble/releases/latest")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| PebbleError::Network(format!("Failed to check for updates: {e}")))?;

    if !resp.status().is_success() {
        return Err(PebbleError::Network(format!(
            "GitHub API returned status {}",
            resp.status()
        )));
    }

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| PebbleError::Network(format!("Failed to parse response: {e}")))?;

    let tag = data["tag_name"]
        .as_str()
        .ok_or_else(|| PebbleError::Network("Missing tag_name in response".to_string()))?;
    let latest = tag.trim_start_matches('v').to_string();
    let release_url = data["html_url"]
        .as_str()
        .unwrap_or("https://github.com/clionertr/Pebble/releases")
        .to_string();

    let has_update = match (Version::parse(&latest), Version::parse(&current_version)) {
        (Ok(latest_ver), Ok(current_ver)) => latest_ver > current_ver,
        _ => latest != current_version,
    };

    Ok(UpdateInfo {
        is_newer: has_update,
        latest_version: latest,
        release_url,
    })
}

pub fn open_external_url(url: String) -> Result<(), PebbleError> {
    // 只允许安全 URL scheme，避免 opener::open / ShellExecuteW 被注入危险命令。
    if !url.starts_with("https://") && !url.starts_with("http://") && !url.starts_with("mailto:") {
        return Err(PebbleError::Validation(
            "Only https://, http://, and mailto: URLs are permitted".to_string(),
        ));
    }
    opener::open(&url).map_err(|e| PebbleError::Internal(format!("Failed to open URL: {e}")))
}

pub fn health_check(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
) -> Result<String, PebbleError> {
    match state.store.list_accounts() {
        Ok(accounts) => Ok(format!(
            "Pebble is healthy. {} account(s) configured.",
            accounts.len()
        )),
        Err(e) => Err(PebbleError::Storage(format!("Health check failed: {e}"))),
    }
}
