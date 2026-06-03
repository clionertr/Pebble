use tracing::warn;

use pebble_core::traits::OutgoingMessage;
use pebble_core::{new_id, DraftMessage, EmailAddress, PebbleError, Result};

pub(super) fn parse_email_header(raw: &str) -> (String, String) {
    // Parse "Display Name <email@example.com>" or just "email@example.com"
    if let Some(start) = raw.rfind('<') {
        if let Some(end) = raw.rfind('>') {
            let name = raw[..start].trim().trim_matches('"').to_string();
            let addr = raw[start + 1..end].trim().to_string();
            return (name, addr);
        }
    }
    (String::new(), raw.trim().to_string())
}

pub(super) fn parse_address_list(raw: &str) -> Vec<EmailAddress> {
    if raw.is_empty() {
        return vec![];
    }
    raw.split(',')
        .map(|s| {
            let (name, address) = parse_email_header(s.trim());
            EmailAddress {
                name: if name.is_empty() { None } else { Some(name) },
                address,
            }
        })
        .collect()
}

pub(super) fn format_address(addr: &EmailAddress) -> String {
    match &addr.name {
        Some(name) => format!("{name} <{}>", addr.address),
        None => addr.address.clone(),
    }
}

fn validate_header_value(label: &str, value: &str) -> Result<()> {
    if value.contains('\r') || value.contains('\n') {
        return Err(PebbleError::Validation(format!(
            "{label} contains invalid header characters"
        )));
    }
    Ok(())
}

fn validate_email_address(label: &str, addr: &EmailAddress) -> Result<()> {
    if let Some(name) = &addr.name {
        validate_header_value(&format!("{label} display name"), name)?;
    }
    validate_header_value(&format!("{label} address"), &addr.address)
}

fn validate_email_addresses(label: &str, addrs: &[EmailAddress]) -> Result<()> {
    for addr in addrs {
        validate_email_address(label, addr)?;
    }
    Ok(())
}

fn quote_mime_param(label: &str, value: &str) -> Result<String> {
    validate_header_value(label, value)?;
    Ok(value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn new_mime_boundary(prefix: &str) -> String {
    format!("{prefix}-{}", new_id())
}

fn write_common_headers(
    raw: &mut String,
    to: &[EmailAddress],
    cc: &[EmailAddress],
    bcc: &[EmailAddress],
    subject: &str,
    in_reply_to: Option<&str>,
) -> Result<()> {
    validate_email_addresses("To", to)?;
    validate_email_addresses("Cc", cc)?;
    validate_email_addresses("Bcc", bcc)?;
    validate_header_value("Subject", subject)?;
    if let Some(irt) = in_reply_to {
        validate_header_value("In-Reply-To", irt)?;
    }

    let to = to.iter().map(format_address).collect::<Vec<_>>().join(", ");
    let cc = cc.iter().map(format_address).collect::<Vec<_>>().join(", ");
    let bcc = bcc
        .iter()
        .map(format_address)
        .collect::<Vec<_>>()
        .join(", ");

    raw.push_str(&format!("To: {to}\r\n"));
    raw.push_str(&format!("Subject: {subject}\r\n"));
    if !cc.is_empty() {
        raw.push_str(&format!("Cc: {cc}\r\n"));
    }
    // Gmail's raw send API derives recipients from the RFC 5322 message. Bcc
    // must be present in the submission raw for Gmail to deliver to those
    // recipients; Gmail is expected to strip it from delivered recipient
    // copies. Do not remove this without replacing it with an envelope-level
    // recipient API.
    if !bcc.is_empty() {
        raw.push_str(&format!("Bcc: {bcc}\r\n"));
    }
    if let Some(irt) = in_reply_to {
        raw.push_str(&format!("In-Reply-To: {irt}\r\n"));
    }
    raw.push_str("MIME-Version: 1.0\r\n");
    Ok(())
}

fn append_body(raw: &mut String, body_text: &str, body_html: Option<&str>) {
    if let Some(body_html) = body_html {
        let boundary = new_mime_boundary("pebble-gmail-boundary");
        raw.push_str(&format!(
            "Content-Type: multipart/alternative; boundary=\"{boundary}\"\r\n\r\n"
        ));
        raw.push_str(&format!(
            "--{boundary}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{body_text}\r\n"
        ));
        raw.push_str(&format!(
            "--{boundary}\r\nContent-Type: text/html; charset=utf-8\r\n\r\n{body_html}\r\n"
        ));
        raw.push_str(&format!("--{boundary}--\r\n"));
    } else {
        raw.push_str("Content-Type: text/plain; charset=utf-8\r\n\r\n");
        raw.push_str(body_text);
        raw.push_str("\r\n");
    }
}

pub(super) fn build_raw_message(msg: &OutgoingMessage) -> Result<Vec<u8>> {
    let mut raw = String::new();
    write_common_headers(
        &mut raw,
        &msg.to,
        &msg.cc,
        &msg.bcc,
        &msg.subject,
        msg.in_reply_to.as_deref(),
    )?;

    if msg.attachment_paths.is_empty() {
        append_body(&mut raw, &msg.body_text, msg.body_html.as_deref());
    } else {
        // multipart/mixed: body + attachments
        let mixed_boundary = new_mime_boundary("pebble-mixed-boundary");
        raw.push_str(&format!(
            "Content-Type: multipart/mixed; boundary=\"{mixed_boundary}\"\r\n\r\n"
        ));

        // Body part
        raw.push_str(&format!("--{mixed_boundary}\r\n"));
        append_body(&mut raw, &msg.body_text, msg.body_html.as_deref());

        // Attachment parts
        for path_str in &msg.attachment_paths {
            let path = std::path::Path::new(path_str);
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("attachment");
            let filename = quote_mime_param("attachment filename", filename)?;
            let data = match std::fs::read(path) {
                Ok(d) => d,
                Err(e) => {
                    warn!("Failed to read attachment {path_str}: {e}, skipping");
                    continue;
                }
            };
            let encoded = base64_standard_encode(&data);
            let content_type = guess_mime_type(&filename);

            raw.push_str(&format!("--{mixed_boundary}\r\n"));
            raw.push_str(&format!(
                "Content-Type: {content_type}; name=\"{filename}\"\r\n"
            ));
            raw.push_str("Content-Transfer-Encoding: base64\r\n");
            raw.push_str(&format!(
                "Content-Disposition: attachment; filename=\"{filename}\"\r\n\r\n"
            ));
            // Wrap base64 at 76 chars per line per RFC 2045
            for chunk in encoded.as_bytes().chunks(76) {
                raw.push_str(std::str::from_utf8(chunk).unwrap_or(""));
                raw.push_str("\r\n");
            }
        }
        raw.push_str(&format!("--{mixed_boundary}--\r\n"));
    }

    Ok(raw.into_bytes())
}

fn base64_standard_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[(n & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn guess_mime_type(filename: &str) -> &'static str {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "txt" => "text/plain",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "zip" => "application/zip",
        "gz" | "gzip" => "application/gzip",
        "tar" => "application/x-tar",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "csv" => "text/csv",
        "mp3" => "audio/mpeg",
        "mp4" => "video/mp4",
        "wav" => "audio/wav",
        "eml" => "message/rfc822",
        _ => "application/octet-stream",
    }
}

pub(super) fn build_draft_raw(draft: &DraftMessage) -> Result<Vec<u8>> {
    let message = OutgoingMessage {
        to: draft.to.clone(),
        cc: draft.cc.clone(),
        bcc: draft.bcc.clone(),
        subject: draft.subject.clone(),
        body_text: draft.body_text.clone(),
        body_html: draft.body_html.clone(),
        in_reply_to: draft.in_reply_to.clone(),
        attachment_paths: draft.attachment_paths.clone(),
    };
    build_raw_message(&message)
}
