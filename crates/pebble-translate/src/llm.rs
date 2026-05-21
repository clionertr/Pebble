use pebble_core::{PebbleError, Result};

use crate::deeplx::build_segments;
use crate::types::{LLMMode, TranslateResult};

pub struct LlmTranslateRequest<'a> {
    pub endpoint: &'a str,
    pub api_key: &'a str,
    pub model: &'a str,
    pub mode: &'a LLMMode,
    pub text: &'a str,
    pub from: &'a str,
    pub to: &'a str,
}

pub async fn translate(
    client: &reqwest::Client,
    request: LlmTranslateRequest<'_>,
) -> Result<TranslateResult> {
    let LlmTranslateRequest {
        endpoint,
        api_key,
        model,
        mode,
        text,
        from,
        to,
    } = request;

    let (url, body) = build_llm_request(endpoint, model, mode, text, from, to, false);

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&body)
        .send()
        .await
        .map_err(|e| PebbleError::Translate(format!("LLM request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        return Err(PebbleError::Translate(format!(
            "LLM error {status}: {body_text}"
        )));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| PebbleError::Translate(format!("LLM parse failed: {e}")))?;

    let translated = match mode {
        LLMMode::Completions => extract_chat_completion_text(&json),
        LLMMode::Responses => extract_responses_text(&json),
    }
    .ok_or_else(|| PebbleError::Translate("LLM response missing translated text".to_string()))?;

    if translated.trim().is_empty() {
        return Err(PebbleError::Translate(
            "LLM response contained empty translation".to_string(),
        ));
    }

    Ok(TranslateResult {
        segments: build_segments(text, &translated),
        translated,
    })
}

pub async fn stream(
    client: &reqwest::Client,
    request: LlmTranslateRequest<'_>,
) -> Result<reqwest::Response> {
    let LlmTranslateRequest {
        endpoint,
        api_key,
        model,
        mode,
        text,
        from,
        to,
    } = request;

    let (url, body) = build_llm_request(endpoint, model, mode, text, from, to, true);

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .header(reqwest::header::ACCEPT, "text/event-stream")
        .json(&body)
        .send()
        .await
        .map_err(|e| PebbleError::Translate(format!("LLM request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        return Err(PebbleError::Translate(format!(
            "LLM error {status}: {body_text}"
        )));
    }

    Ok(resp)
}

fn build_llm_request(
    endpoint: &str,
    model: &str,
    mode: &LLMMode,
    text: &str,
    from: &str,
    to: &str,
    stream: bool,
) -> (String, serde_json::Value) {
    let system_prompt = format!(
        "You are a professional translator. Translate the following text from {from} to {to}. \
         Output ONLY the translation, nothing else. Preserve formatting and line breaks."
    );

    match mode {
        LLMMode::Completions => (
            format!("{}/v1/chat/completions", endpoint.trim_end_matches('/')),
            serde_json::json!({
                "model": model,
                "messages": [
                    { "role": "system", "content": system_prompt },
                    { "role": "user", "content": text }
                ],
                "temperature": 0.3,
                "stream": stream,
            }),
        ),
        LLMMode::Responses => (
            format!("{}/v1/responses", endpoint.trim_end_matches('/')),
            serde_json::json!({
                "model": model,
                "input": format!("{system_prompt}\n\n{text}"),
                "stream": stream,
            }),
        ),
    }
}

fn extract_chat_completion_text(json: &serde_json::Value) -> Option<String> {
    json.get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(extract_content_text)
}

fn extract_responses_text(json: &serde_json::Value) -> Option<String> {
    if let Some(text) = json.get("output_text").and_then(|text| text.as_str()) {
        if !text.trim().is_empty() {
            return Some(text.to_string());
        }
    }

    let output = json.get("output")?.as_array()?;
    let mut parts = Vec::new();
    for item in output {
        if let Some(text) = item.get("content").and_then(extract_content_text) {
            parts.push(text);
        }
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(""))
    }
}

fn extract_content_text(value: &serde_json::Value) -> Option<String> {
    if let Some(text) = value.as_str() {
        return Some(text.to_string());
    }

    let items = value.as_array()?;
    let mut parts = Vec::new();
    for item in items {
        if let Some(text) = item.get("text").and_then(|text| text.as_str()) {
            parts.push(text.to_string());
        } else if let Some(text) = item.get("content").and_then(|text| text.as_str()) {
            parts.push(text.to_string());
        }
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_responses_output_text() {
        let json = serde_json::json!({ "output_text": "你好" });

        assert_eq!(extract_responses_text(&json).unwrap(), "你好");
    }

    #[test]
    fn extracts_responses_nested_output_text() {
        let json = serde_json::json!({
            "output": [{
                "type": "message",
                "content": [{ "type": "output_text", "text": "你好" }]
            }]
        });

        assert_eq!(extract_responses_text(&json).unwrap(), "你好");
    }

    #[test]
    fn extracts_chat_completion_array_content() {
        let json = serde_json::json!({
            "choices": [{
                "message": {
                    "content": [{ "type": "text", "text": "你好" }]
                }
            }]
        });

        assert_eq!(extract_chat_completion_text(&json).unwrap(), "你好");
    }

    #[test]
    fn stream_request_sets_stream_true() {
        let (_, body) = build_llm_request(
            "https://api.openai.com/v1",
            "gpt-4o-mini",
            &LLMMode::Completions,
            "Hello",
            "en",
            "zh",
            true,
        );

        assert_eq!(body.get("stream").and_then(|value| value.as_bool()), Some(true));
    }
}
