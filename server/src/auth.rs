use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect},
};
use pebble_core::{new_id, now_timestamp, Account, HttpProxyConfig, OAuthTokens};
use pebble_oauth::{OAuthManager, OAuthNetworkConfig, PkceState};
use serde::Deserialize;
use std::sync::Arc;

pub struct OAuthSession {
    pub provider: String,
    pub pkce_state: PkceState,
    pub proxy: Option<HttpProxyConfig>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginQuery {
    provider: String,
    proxy_host: Option<String>,
    proxy_port: Option<u16>,
}

fn account_color_for_existing_oauth_account(
    existing: &Account,
    existing_accounts: &[Account],
    email: &str,
) -> String {
    existing
        .color
        .clone()
        .unwrap_or_else(|| crate::account_colors::default_account_color(existing_accounts, email))
}

pub async fn login_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LoginQuery>,
) -> impl IntoResponse {
    let LoginQuery {
        provider,
        proxy_host,
        proxy_port,
    } = query;
    let proxy =
        match crate::rpc::network::proxy_config_from_parts(proxy_host, proxy_port, "OAuth proxy") {
            Ok(proxy) => proxy,
            Err(e) => return Html(format!("<h1>Error</h1><p>{}</p>", e)).into_response(),
        };
    let config = match crate::rpc::oauth::config_for_provider(&provider) {
        Ok(c) => c,
        Err(e) => {
            return Html(format!("<h1>Error</h1><p>Invalid provider: {}</p>", e)).into_response()
        }
    };

    let manager = OAuthManager::new(config);
    let (auth_url, pkce_state) = match manager.start_auth().await {
        Ok(res) => res,
        Err(e) => {
            return Html(format!(
                "<h1>Error</h1><p>Failed to start OAuth flow: {}</p>",
                e
            ))
            .into_response()
        }
    };

    let state_str = pkce_state.csrf_token.secret().clone();
    state.oauth_states.lock().await.insert(
        state_str,
        OAuthSession {
            provider,
            pkce_state,
            proxy,
        },
    );

    Redirect::to(&auth_url).into_response()
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

pub async fn callback_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CallbackQuery>,
) -> impl IntoResponse {
    if let Some(err) = query.error {
        return Html(format!("<h1>OAuth Error</h1><p>{}</p>", err)).into_response();
    }

    let code = match query.code {
        Some(c) => c,
        None => return Html("<h1>Error</h1><p>Missing code</p>".to_string()).into_response(),
    };

    let state_str = match query.state {
        Some(s) => s,
        None => return Html("<h1>Error</h1><p>Missing state</p>".to_string()).into_response(),
    };

    let session = match state.oauth_states.lock().await.remove(&state_str) {
        Some(sess) => sess,
        None => {
            return Html("<h1>Error</h1><p>Invalid or expired session</p>".to_string())
                .into_response()
        }
    };

    let config = match crate::rpc::oauth::config_for_provider(&session.provider) {
        Ok(c) => c,
        Err(e) => return Html(format!("<h1>Error</h1><p>{}</p>", e)).into_response(),
    };

    let global_proxy =
        crate::rpc::network::get_global_proxy_raw(&state.crypto, &state.store).unwrap_or_default();
    let effective_proxy = session.proxy.clone().or(global_proxy);
    let network = OAuthNetworkConfig {
        proxy: effective_proxy,
    };
    let manager = OAuthManager::new_with_network(config, network.clone());

    let token_pair = match manager.complete_auth(&code, session.pkce_state).await {
        Ok(tp) => tp,
        Err(e) => {
            let message = crate::rpc::oauth::token_exchange_error_message(&session.provider, &e);
            return Html(format!("<h1>Error</h1><p>{}</p>", message)).into_response();
        }
    };

    // Note: We've decoupled fetch_userinfo and account creation into a background task
    // or we can do it inline. In a real app we might want to do it inline to show errors.
    // For now we'll do it inline, using a dummy fetch_userinfo if we don't want to copy the whole thing,
    // but actually we can make fetch_userinfo in oauth.rs pub(crate). Let's assume we will.

    match complete_account_creation(
        &state,
        &session.provider,
        token_pair,
        &network,
        session.proxy.clone(),
    )
    .await
    {
        Ok(_) => Html(
            "
            <html>
                <head><title>Success</title></head>
                <body style='font-family:sans-serif;text-align:center;padding:50px;'>
                    <h2>Account Added Successfully!</h2>
                    <p>You can close this tab and return to the application.</p>
                    <script>
                        setTimeout(() => {
                            window.location.href = '/';
                        }, 2000);
                    </script>
                </body>
            </html>
        "
            .to_string(),
        )
        .into_response(),
        Err(e) => Html(format!(
            "<h1>Error</h1><p>Failed to create account: {}</p>",
            e
        ))
        .into_response(),
    }
}

async fn complete_account_creation(
    state: &AppState,
    provider: &str,
    token_pair: pebble_oauth::TokenPair,
    network: &OAuthNetworkConfig,
    account_proxy: Option<HttpProxyConfig>,
) -> Result<(), pebble_core::PebbleError> {
    let (real_email, real_name) =
        crate::rpc::oauth::fetch_userinfo(provider, &token_pair.access_token, network).await?;

    let now = now_timestamp();
    let existing_accounts = state.store.list_accounts()?;
    let provider_type = crate::rpc::oauth::provider_type(provider)?;

    if let Some(existing) = existing_accounts
        .iter()
        .find(|a| a.email == real_email && a.provider == provider_type)
    {
        let color =
            account_color_for_existing_oauth_account(existing, &existing_accounts, &real_email);
        state
            .store
            .update_account(&existing.id, &real_email, &real_name, Some(&color))?;

        let tokens = OAuthTokens {
            access_token: token_pair.access_token,
            refresh_token: token_pair.refresh_token,
            expires_at: token_pair.expires_at,
            scopes: token_pair.scopes,
        };

        if let Some(proxy) = account_proxy.clone() {
            crate::rpc::oauth::persist_oauth_tokens_with_custom_proxy_raw(
                &state.crypto,
                &state.store,
                &existing.id,
                &tokens,
                proxy,
            )?;
        } else {
            crate::rpc::oauth::persist_oauth_tokens_raw(
                &state.crypto,
                &state.store,
                &existing.id,
                &tokens,
            )?;
        }

        state.store.update_sync_state(&existing.id, |s| {
            s.last_sync_cursor = None;
        })?;
    } else {
        let account_color = Some(crate::account_colors::default_account_color(
            &existing_accounts,
            &real_email,
        ));

        let account = Account {
            id: new_id(),
            email: real_email,
            display_name: real_name,
            color: account_color,
            provider: provider_type,
            created_at: now,
            updated_at: now,
        };

        state.store.insert_account(&account)?;

        let tokens = OAuthTokens {
            access_token: token_pair.access_token,
            refresh_token: token_pair.refresh_token,
            expires_at: token_pair.expires_at,
            scopes: token_pair.scopes,
        };

        let stored = crate::rpc::oauth::StoredOAuthAuthData::from_tokens(tokens, account_proxy);
        crate::rpc::oauth::persist_stored_oauth_auth_data_raw(
            &state.crypto,
            &state.store,
            &account.id,
            &stored,
        )?;

        let slug = crate::rpc::oauth::provider_slug(&account.provider).to_string();
        state.store.update_sync_state(&account.id, |s| {
            s.provider = Some(slug);
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pebble_core::ProviderType;

    fn account(id: &str, color: Option<&str>) -> Account {
        let now = now_timestamp();
        Account {
            id: id.to_string(),
            email: format!("{id}@example.com"),
            display_name: id.to_string(),
            color: color.map(ToOwned::to_owned),
            provider: ProviderType::Gmail,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn existing_oauth_account_color_keeps_saved_color_on_relogin() {
        let existing = account("gmail", Some("#f43f5e"));
        let accounts = vec![account("other", Some("#0ea5e9")), existing.clone()];

        let color =
            account_color_for_existing_oauth_account(&existing, &accounts, "gmail@example.com");

        assert_eq!(color, "#f43f5e");
    }

    #[test]
    fn existing_oauth_account_color_assigns_default_when_missing() {
        let existing = account("gmail", None);
        let accounts = vec![account("other", Some("#0ea5e9")), existing.clone()];

        let color =
            account_color_for_existing_oauth_account(&existing, &accounts, "gmail@example.com");

        assert_eq!(color, "#22c55e");
    }
}
