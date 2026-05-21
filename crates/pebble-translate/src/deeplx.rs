use pebble_core::{PebbleError, Result};
use reqwest::StatusCode;
use serde_json::Value;

use crate::types::{BilingualSegment, TranslateResult};

pub async fn translate(
    client: &reqwest::Client,
    endpoint: &str,
    text: &str,
    from: &str,
    to: &str,
) -> Result<TranslateResult> {
    let body = serde_json::json!({
        "text": text,
        "source_lang": deeplx_lang(from),
        "target_lang": deeplx_lang(to),
    });

    let resp = client
        .post(endpoint)
        .json(&body)
        .send()
        .await
        .map_err(|e| PebbleError::Translate(format!("DeepLX request failed: {e}")))?;

    let status = resp.status();
    let body_text = resp
        .text()
        .await
        .map_err(|e| PebbleError::Translate(format!("DeepLX response read failed: {e}")))?;

    if !status.is_success() {
        return Err(PebbleError::Translate(format_deeplx_http_error(
            status, &body_text,
        )));
    }

    let json: Value = serde_json::from_str(&body_text)
        .map_err(|e| PebbleError::Translate(format!("DeepLX response parse failed: {e}")))?;
    let translated = parse_deeplx_translation(&json)?;

    Ok(TranslateResult {
        segments: build_segments(text, &translated),
        translated,
    })
}

fn deeplx_lang(lang: &str) -> String {
    let trimmed = lang.trim();
    if trimmed.eq_ignore_ascii_case("auto") {
        "auto".to_string()
    } else {
        trimmed.to_uppercase()
    }
}

fn format_deeplx_http_error(status: StatusCode, body: &str) -> String {
    let remote_message = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|json| {
            json.get("message")
                .and_then(|message| message.as_str())
                .map(str::to_string)
        })
        .filter(|message| !message.trim().is_empty())
        .unwrap_or_else(|| body.trim().to_string());

    if status == StatusCode::UNAUTHORIZED {
        return format!(
            "DeepLX unauthorized (401): endpoint token is invalid or expired. Remote response: {remote_message}"
        );
    }

    format!("DeepLX error {status}: {remote_message}")
}

fn parse_deeplx_translation(json: &Value) -> Result<String> {
    if let Some(code) = json.get("code").and_then(|code| code.as_i64()) {
        if !(200..300).contains(&code) {
            let message = json
                .get("message")
                .and_then(|message| message.as_str())
                .unwrap_or("unknown error");
            return Err(PebbleError::Translate(format!(
                "DeepLX error {code}: {message}"
            )));
        }
    }

    let translated = json
        .get("data")
        .and_then(|data| data.as_str())
        .ok_or_else(|| PebbleError::Translate("DeepLX response missing data field".to_string()))?;

    if translated.trim().is_empty() {
        return Err(PebbleError::Translate(
            "DeepLX response contained empty translation".to_string(),
        ));
    }

    Ok(translated.to_string())
}

pub fn build_segments(source: &str, target: &str) -> Vec<BilingualSegment> {
    source
        .split('\n')
        .zip(target.split('\n'))
        .filter(|(s, _)| !s.trim().is_empty())
        .map(|(s, t)| BilingualSegment {
            source: s.to_string(),
            target: t.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_segments() {
        let segments = build_segments("Hello\nWorld\n\nFoo", "你好\n世界\n\nFoo翻译");
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].source, "Hello");
        assert_eq!(segments[0].target, "你好");
        assert_eq!(segments[1].source, "World");
        assert_eq!(segments[1].target, "世界");
    }

    #[test]
    fn test_build_segments_uneven() {
        let segments = build_segments("Line1\nLine2\nLine3", "译1\n译2");
        // zip stops at shorter
        assert_eq!(segments.len(), 2);
    }

    #[test]
    fn deeplx_lang_keeps_auto_lowercase() {
        assert_eq!(deeplx_lang("auto"), "auto");
        assert_eq!(deeplx_lang("AUTO"), "auto");
        assert_eq!(deeplx_lang("en"), "EN");
    }

    #[test]
    fn deeplx_unauthorized_error_explains_token_problem() {
        let message = format_deeplx_http_error(
            StatusCode::UNAUTHORIZED,
            r#"{"code":401,"message":"status code: 401"}"#,
        );

        assert!(message.contains("endpoint token is invalid or expired"));
        assert!(message.contains("status code: 401"));
    }

    #[test]
    fn parse_deeplx_translation_rejects_empty_data() {
        let err = parse_deeplx_translation(&serde_json::json!({ "code": 200, "data": "" }))
            .unwrap_err();

        assert!(err.to_string().contains("empty translation"));
    }
}
