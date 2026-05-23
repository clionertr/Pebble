use pebble_core::{PebbleError, Result};

use crate::deeplx::build_segments;
use crate::types::TranslateResult;

pub async fn translate(
    client: &reqwest::Client,
    api_key: &str,
    use_free_api: bool,
    text: &str,
    from: &str,
    to: &str,
) -> Result<TranslateResult> {
    let base = if use_free_api {
        "https://api-free.deepl.com/v2/translate"
    } else {
        "https://api.deepl.com/v2/translate"
    };

    let form = build_deepl_form(text, from, to);

    let resp = client
        .post(base)
        .header("Authorization", format!("DeepL-Auth-Key {api_key}"))
        .form(&form)
        .send()
        .await
        .map_err(|e| PebbleError::Translate(format!("DeepL request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(PebbleError::Translate(format!(
            "DeepL error {status}: {body}"
        )));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| PebbleError::Translate(format!("DeepL parse failed: {e}")))?;

    let translated = json
        .get("translations")
        .and_then(|t| t.get(0))
        .and_then(|t| t.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| {
            PebbleError::Translate("DeepL response missing translated text".to_string())
        })?;

    if translated.trim().is_empty() {
        return Err(PebbleError::Translate(
            "DeepL response contained empty translation".to_string(),
        ));
    }

    Ok(TranslateResult {
        segments: build_segments(text, translated),
        translated: translated.to_string(),
    })
}

fn build_deepl_form(text: &str, from: &str, to: &str) -> Vec<(&'static str, String)> {
    let mut form = vec![
        ("text", text.to_string()),
        ("target_lang", to.trim().to_uppercase()),
    ];
    let from_trimmed = from.trim();
    if !from_trimmed.is_empty() && !from_trimmed.eq_ignore_ascii_case("auto") {
        form.push(("source_lang", from_trimmed.to_uppercase()));
    }
    form
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deepl_form_omits_auto_source_lang() {
        let form = build_deepl_form("Hello", "AUTO", "zh");

        assert_eq!(
            form,
            vec![
                ("text", "Hello".to_string()),
                ("target_lang", "ZH".to_string()),
            ]
        );
    }
}
