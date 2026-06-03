use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::parser::AttachmentData;
use pebble_core::new_id;
use pebble_store::Store;
use tracing::warn;

/// 清理附件文件名，避免路径穿越和 Windows 保留名。
pub(crate) fn sanitize_filename(name: &str) -> String {
    fn is_windows_reserved(stem: &str) -> bool {
        let upper = stem.trim().to_ascii_uppercase();
        matches!(
            upper.as_str(),
            "CON"
                | "PRN"
                | "AUX"
                | "NUL"
                | "COM1"
                | "COM2"
                | "COM3"
                | "COM4"
                | "COM5"
                | "COM6"
                | "COM7"
                | "COM8"
                | "COM9"
                | "LPT1"
                | "LPT2"
                | "LPT3"
                | "LPT4"
                | "LPT5"
                | "LPT6"
                | "LPT7"
                | "LPT8"
                | "LPT9"
        )
    }

    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    if base == ".." || base == "." {
        return "unnamed_attachment".to_string();
    }

    let mut cleaned = base.to_string();
    while cleaned.contains("..") {
        cleaned = cleaned.replace("..", ".");
    }
    let sanitized: String = cleaned
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '|' | '?' | '*' => '_',
            _ => c,
        })
        .filter(|c| !c.is_control())
        .collect();

    let trimmed = sanitized
        .trim()
        .trim_matches(|c: char| c == '.' || c == ' ');
    if trimmed.is_empty() {
        return "unnamed_attachment".to_string();
    }

    let stem = trimmed.split('.').next().unwrap_or(trimmed);
    if is_windows_reserved(stem) {
        return "unnamed_attachment".to_string();
    }

    trimmed.to_string()
}

/// 将附件写入磁盘并把元数据写入 store。
pub(crate) fn persist_message_attachments(
    store: &Store,
    attachments_root: &Path,
    message_id: &str,
    attachments: Vec<AttachmentData>,
) {
    use std::io::Write;
    const CHUNK_SIZE: usize = 64 * 1024;

    for att_data in attachments.into_iter() {
        let att_dir = attachments_root.join(message_id);
        if std::fs::create_dir_all(&att_dir).is_err() {
            warn!("Failed to create attachment dir for message {}", message_id);
            continue;
        }

        let safe_filename = sanitize_filename(&att_data.meta.filename);
        if safe_filename.is_empty() {
            warn!("Attachment has empty filename after sanitization, skipping");
            continue;
        }

        let mut file_path = att_dir.join(&safe_filename);
        let mut counter = 1u32;
        while file_path.exists() {
            let stem = std::path::Path::new(&safe_filename)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();
            let ext = std::path::Path::new(&safe_filename)
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default();
            file_path = att_dir.join(format!("{stem}_{counter}{ext}"));
            counter += 1;
        }
        let file = match std::fs::File::create(&file_path) {
            Ok(f) => f,
            Err(e) => {
                warn!(
                    "Failed to create attachment file {}: {}",
                    file_path.display(),
                    e
                );
                continue;
            }
        };
        let mut writer = std::io::BufWriter::with_capacity(CHUNK_SIZE, file);

        let AttachmentData { meta, data } = att_data;
        let mut write_ok = true;
        for chunk in data.chunks(CHUNK_SIZE) {
            if let Err(e) = writer.write_all(chunk) {
                warn!(
                    "Failed to write attachment file {}: {}",
                    file_path.display(),
                    e
                );
                write_ok = false;
                break;
            }
        }
        drop(data);

        if !write_ok {
            let _ = std::fs::remove_file(&file_path);
            continue;
        }
        if let Err(e) = writer.flush() {
            warn!(
                "Failed to flush attachment file {}: {}",
                file_path.display(),
                e
            );
            let _ = std::fs::remove_file(&file_path);
            continue;
        }

        let attachment = pebble_core::Attachment {
            id: new_id(),
            message_id: message_id.to_string(),
            filename: meta.filename,
            mime_type: meta.mime_type,
            size: meta.size as i64,
            local_path: Some(file_path.to_string_lossy().to_string()),
            content_id: meta.content_id,
            is_inline: meta.is_inline,
        };
        if let Err(e) = store.insert_attachment(&attachment) {
            warn!("Failed to store attachment record: {}", e);
        }
    }
}

/// 异步包装，把附件文件 I/O 放到 blocking 线程。
pub(crate) async fn persist_message_attachments_async(
    store: Arc<Store>,
    attachments_root: PathBuf,
    message_id: String,
    attachments: Vec<AttachmentData>,
) {
    if attachments.is_empty() {
        return;
    }
    let _ = tokio::task::spawn_blocking(move || {
        persist_message_attachments(&store, &attachments_root, &message_id, attachments);
    })
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename_rejects_windows_reserved_names() {
        assert_eq!(sanitize_filename("CON.txt"), "unnamed_attachment");
        assert_eq!(sanitize_filename("aux"), "unnamed_attachment");
        assert_eq!(sanitize_filename("LPT1.log"), "unnamed_attachment");
    }

    #[test]
    fn test_sanitize_filename_removes_windows_unsafe_characters() {
        assert_eq!(
            sanitize_filename("quarterly:report*final?.pdf"),
            "quarterly_report_final_.pdf",
        );
        assert_eq!(sanitize_filename("report. "), "report");
    }
}
