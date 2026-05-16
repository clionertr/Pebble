use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub const LOG_FILE_NAME: &str = "pebble.log";
const DEFAULT_LOG_MAX_BYTES: u64 = 64 * 1024;
const MAX_LOG_MAX_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppLogSnapshot {
    pub path: String,
    pub content: String,
    pub truncated: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MailDisplayTiming {
    pub account_id: Option<String>,
    pub message_id: String,
    pub source: Option<String>,
    pub active_folder_id: Option<String>,
    pub backend_received_at_ms: Option<i64>,
    pub backend_sse_at_ms: Option<i64>,
    pub message_received_at_ms: Option<i64>,
    pub frontend_sse_at_ms: i64,
    pub displayed_at_ms: i64,
    pub frontend_sse_to_display_ms: Option<i64>,
}

pub fn app_log_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("logs")
}

fn app_log_path(app_data_dir: &Path) -> PathBuf {
    app_log_dir(app_data_dir).join(LOG_FILE_NAME)
}

fn read_log_tail(path: &Path, max_bytes: u64) -> Result<AppLogSnapshot, String> {
    let path_display = path.display().to_string();
    let Ok(metadata) = fs::metadata(path) else {
        return Ok(AppLogSnapshot {
            path: path_display,
            content: String::new(),
            truncated: false,
        });
    };

    let file_len = metadata.len();
    let truncated = file_len > max_bytes;
    let start = if truncated { file_len - max_bytes } else { 0 };
    let mut file = fs::File::open(path).map_err(|e| format!("Failed to open app log: {e}"))?;
    file.seek(SeekFrom::Start(start))
        .map_err(|e| format!("Failed to seek app log: {e}"))?;

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|e| format!("Failed to read app log: {e}"))?;

    Ok(AppLogSnapshot {
        path: path_display,
        content: String::from_utf8_lossy(&bytes).into_owned(),
        truncated,
    })
}

pub fn read_app_log(max_bytes: Option<u64>) -> Result<AppLogSnapshot, String> {
    let app_data_dir = std::path::PathBuf::from("./data");
    let max_bytes = max_bytes
        .unwrap_or(DEFAULT_LOG_MAX_BYTES)
        .clamp(1, MAX_LOG_MAX_BYTES);
    read_log_tail(&app_log_path(&app_data_dir), max_bytes)
}

pub fn record_mail_display_timing(timing: MailDisplayTiming) -> Result<(), String> {
    if !crate::mail_latency::debug_enabled() {
        return Ok(());
    }

    let display_report_received_at_ms = crate::mail_latency::now_ms();
    let backend_sse_to_display_report_ms = timing
        .backend_sse_at_ms
        .map(|server_ms| display_report_received_at_ms.saturating_sub(server_ms));
    let frontend_sse_to_display_ms = timing.frontend_sse_to_display_ms.unwrap_or_else(|| {
        timing
            .displayed_at_ms
            .saturating_sub(timing.frontend_sse_at_ms)
    });
    let push_to_display_report_ms = timing
        .backend_received_at_ms
        .map(|push_ms| display_report_received_at_ms.saturating_sub(push_ms));
    let message_to_display_report_ms = timing
        .message_received_at_ms
        .map(|message_ms| display_report_received_at_ms.saturating_sub(message_ms));
    let client_clock_offset_at_report_ms =
        display_report_received_at_ms.saturating_sub(timing.displayed_at_ms);

    tracing::debug!(
        target: "pebble::mail_latency",
        stage = "frontend_message_displayed",
        account_id = timing.account_id.as_deref().unwrap_or(""),
        message_id = timing.message_id.as_str(),
        source = timing.source.as_deref().unwrap_or(""),
        active_folder_id = timing.active_folder_id.as_deref().unwrap_or(""),
        backend_received_at_ms = ?timing.backend_received_at_ms,
        backend_sse_at_ms = ?timing.backend_sse_at_ms,
        frontend_sse_at_ms = timing.frontend_sse_at_ms,
        displayed_at_ms = timing.displayed_at_ms,
        display_report_received_at_ms,
        client_clock_offset_at_report_ms,
        backend_sse_to_display_report_ms = ?backend_sse_to_display_report_ms,
        frontend_sse_to_display_ms,
        push_to_display_report_ms = ?push_to_display_report_ms,
        message_to_display_report_ms = ?message_to_display_report_ms,
        "mail latency event"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_log_path() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be available")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "pebble-diagnostics-test-{}-{nonce}.log",
            std::process::id()
        ))
    }

    #[test]
    fn read_log_tail_returns_only_recent_bytes_when_file_is_large() {
        let path = temp_log_path();
        fs::write(&path, "alpha\nbeta\ngamma\n").expect("test log should be writable");

        let snapshot = super::read_log_tail(&path, 11).expect("tail should be readable");

        assert_eq!(snapshot.content, "beta\ngamma\n");
        assert_eq!(snapshot.path, path.display().to_string());
        assert!(snapshot.truncated);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn read_log_tail_returns_empty_snapshot_when_file_is_missing() {
        let path = temp_log_path();

        let snapshot = super::read_log_tail(&path, 128).expect("missing log should not error");

        assert_eq!(snapshot.content, "");
        assert_eq!(snapshot.path, path.display().to_string());
        assert!(!snapshot.truncated);
    }
}
