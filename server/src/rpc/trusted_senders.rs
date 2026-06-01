use pebble_core::{now_timestamp, PebbleError, TrustType, TrustedSender};

pub async fn trust_sender(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    account_id: String,
    email: String,
    trust_type: TrustType,
) -> std::result::Result<(), PebbleError> {
    let sender = TrustedSender {
        account_id,
        email,
        trust_type,
        created_at: now_timestamp(),
    };
    state.store.trust_sender(&sender)
}

pub async fn list_trusted_senders(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    account_id: Option<String>,
) -> std::result::Result<Vec<TrustedSender>, PebbleError> {
    match account_id {
        Some(account_id) if !account_id.is_empty() => state.store.list_trusted_senders(&account_id),
        _ => state.store.list_all_trusted_senders(),
    }
}

pub async fn remove_trusted_sender(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    account_id: String,
    email: String,
) -> std::result::Result<(), PebbleError> {
    state.store.remove_trusted_sender(&account_id, &email)
}
