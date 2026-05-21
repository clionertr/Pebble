use super::network::get_global_proxy_raw;
use crate::state::AppState;
use pebble_core::{now_timestamp, PebbleError, TranslateConfig};
use pebble_translate::types::{TranslateProviderConfig, TranslateResult};
use pebble_translate::TranslateService;

/// Decode a hex string to bytes.
fn hex_decode(s: &str) -> std::result::Result<Vec<u8>, PebbleError> {
    if !s.len().is_multiple_of(2) {
        return Err(PebbleError::Internal(
            "Invalid hex string length".to_string(),
        ));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|e| PebbleError::Internal(format!("Invalid hex: {e}")))
}

/// Encode bytes to a hex string.
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Decrypt the config field of a TranslateConfig using the app's crypto service.
/// If the stored value is legacy plaintext JSON, migrates it to encrypted form in-place.
fn decrypt_config(state: &AppState, stored: &str) -> std::result::Result<String, PebbleError> {
    if serde_json::from_str::<serde_json::Value>(stored).is_ok() {
        // Legacy plaintext config — migrate to encrypted form in-place.
        let encrypted = encrypt_config(state, stored)?;
        state.store.update_translate_config_blob(&encrypted)?;
        return Ok(stored.to_string());
    }
    let bytes = hex_decode(stored)?;
    let decrypted = state.crypto.decrypt(&bytes)?;
    String::from_utf8(decrypted)
        .map_err(|e| PebbleError::Internal(format!("Invalid UTF-8 in decrypted config: {e}")))
}

/// Encrypt a plaintext config string for storage.
fn encrypt_config(state: &AppState, plaintext: &str) -> std::result::Result<String, PebbleError> {
    let encrypted = state.crypto.encrypt(plaintext.as_bytes())?;
    Ok(hex_encode(&encrypted))
}

pub async fn translate_text(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    text: String,
    from_lang: String,
    to_lang: String,
) -> std::result::Result<TranslateResult, PebbleError> {
    let provider_config = load_active_provider_config(&state)?;
    let proxy = get_global_proxy_raw(&state.crypto, &state.store)?;

    TranslateService::translate_with_proxy(
        &provider_config,
        proxy.as_ref(),
        &text,
        &from_lang,
        &to_lang,
    )
    .await
}

pub async fn translate_text_stream(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    text: String,
    from_lang: String,
    to_lang: String,
) -> std::result::Result<reqwest::Response, PebbleError> {
    let provider_config = load_active_provider_config(&state)?;
    let TranslateProviderConfig::LLM {
        endpoint,
        api_key,
        model,
        mode,
    } = provider_config
    else {
        return Err(PebbleError::Validation(
            "Streaming translation is only supported for LLM providers".to_string(),
        ));
    };

    let proxy = get_global_proxy_raw(&state.crypto, &state.store)?;
    let client = TranslateService::http_client_with_proxy(proxy.as_ref())?;

    pebble_translate::llm::stream(
        &client,
        pebble_translate::llm::LlmTranslateRequest {
            endpoint: &endpoint,
            api_key: &api_key,
            model: &model,
            mode: &mode,
            text: &text,
            from: &from_lang,
            to: &to_lang,
        },
    )
    .await
}

fn load_active_provider_config(
    state: &AppState,
) -> std::result::Result<TranslateProviderConfig, PebbleError> {
    let config = state
        .store
        .get_translate_config()?
        .ok_or_else(|| PebbleError::Translate("No translate engine configured".to_string()))?;

    if !config.is_enabled {
        return Err(PebbleError::Translate(
            "Translation is disabled".to_string(),
        ));
    }

    let decrypted = decrypt_config(state, &config.config)?;
    let provider_config: TranslateProviderConfig = serde_json::from_str(&decrypted)
        .map_err(|e| PebbleError::Translate(format!("Invalid config: {e}")))?;

    validate_provider_config(&provider_config)?;
    Ok(provider_config)
}

pub async fn get_translate_config(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
) -> std::result::Result<Option<TranslateConfig>, PebbleError> {
    let config = state.store.get_translate_config()?;
    // Return config with decrypted config field so frontend can display/edit it
    match config {
        Some(mut tc) => {
            tc.config = decrypt_config(&state, &tc.config)?;
            Ok(Some(tc))
        }
        None => Ok(None),
    }
}

pub async fn save_translate_config(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    provider_type: String,
    config: String,
    is_enabled: bool,
) -> std::result::Result<(), PebbleError> {
    // Validate URL(s) in config before persisting
    let provider_config: TranslateProviderConfig = serde_json::from_str(&config)
        .map_err(|e| PebbleError::Translate(format!("Invalid config: {e}")))?;
    if provider_type != provider_type_name(&provider_config) {
        return Err(PebbleError::Translate(
            "Provider type does not match translate config".to_string(),
        ));
    }
    validate_provider_config(&provider_config)?;

    let now = now_timestamp();
    // Encrypt config before storing
    let encrypted = encrypt_config(&state, &config)?;
    let tc = TranslateConfig {
        id: "active".to_string(),
        provider_type,
        config: encrypted,
        is_enabled,
        created_at: now,
        updated_at: now,
    };
    state.store.save_translate_config(&tc)
}

/// Validate URL(s) in a TranslateProviderConfig.
fn validate_provider_config(
    provider_config: &TranslateProviderConfig,
) -> std::result::Result<(), PebbleError> {
    match provider_config {
        TranslateProviderConfig::DeepLX { endpoint } => validate_translate_url(endpoint),
        TranslateProviderConfig::GenericApi { endpoint, .. } => validate_translate_url(endpoint),
        TranslateProviderConfig::LLM { endpoint, .. } => validate_translate_url(endpoint),
        TranslateProviderConfig::DeepL { .. } => Ok(()), // uses official API, no custom URL
    }
}

fn provider_type_name(provider_config: &TranslateProviderConfig) -> &'static str {
    match provider_config {
        TranslateProviderConfig::DeepLX { .. } => "deeplx",
        TranslateProviderConfig::DeepL { .. } => "deepl",
        TranslateProviderConfig::GenericApi { .. } => "generic_api",
        TranslateProviderConfig::LLM { .. } => "llm",
    }
}

/// Validate that a translate endpoint URL is safe (HTTPS required, HTTP only for localhost).
fn validate_translate_url(url: &str) -> std::result::Result<(), PebbleError> {
    if url.starts_with("https://") {
        return Ok(());
    }
    if let Some(after_scheme) = url.strip_prefix("http://") {
        // Extract host from http://host[:port]/...
        let host = after_scheme
            .split('/')
            .next()
            .unwrap_or("")
            .split(':')
            .next()
            .unwrap_or("");
        if host == "localhost" || host == "127.0.0.1" || host == "::1" || host == "[::1]" {
            return Ok(());
        }
        return Err(PebbleError::Validation(
            "Only HTTPS URLs are allowed for remote services".into(),
        ));
    }
    Err(PebbleError::Validation("Unsupported URL scheme".into()))
}

pub async fn test_translate_connection(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    config: String,
) -> std::result::Result<String, PebbleError> {
    let provider_config: TranslateProviderConfig = serde_json::from_str(&config)
        .map_err(|e| PebbleError::Translate(format!("Invalid config: {e}")))?;

    // Validate endpoint URLs before making any requests
    validate_provider_config(&provider_config)?;

    let proxy = get_global_proxy_raw(&state.crypto, &state.store)?;
    let result = TranslateService::translate_with_proxy(
        &provider_config,
        proxy.as_ref(),
        "Hello",
        "en",
        "zh",
    )
    .await?;
    Ok(result.translated)
}
