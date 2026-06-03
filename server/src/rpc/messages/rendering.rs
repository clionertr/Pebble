use axum::extract::State;
use pebble_core::{Message, PebbleError, PrivacyMode, RenderedHtml, TrustType};
use pebble_privacy::PrivacyGuard;
use pebble_store::Store;

pub async fn get_rendered_html(
    state: State<std::sync::Arc<crate::state::AppState>>,
    message_id: String,
    privacy_mode: PrivacyMode,
) -> std::result::Result<RenderedHtml, PebbleError> {
    let store = state.store.clone();
    store
        .with_blocking_async(move |store| {
            let message = store
                .get_message(&message_id)?
                .ok_or_else(|| PebbleError::Internal(format!("Message not found: {message_id}")))?;

            let effective_mode = resolve_privacy_mode(store, &message, privacy_mode)?;
            let guard = PrivacyGuard::new();
            Ok(guard.render_message_html(
                &message.body_html_raw,
                &message.body_text,
                &effective_mode,
            ))
        })
        .await
}

pub async fn get_message_with_html(
    state: State<std::sync::Arc<crate::state::AppState>>,
    message_id: String,
    privacy_mode: PrivacyMode,
) -> std::result::Result<Option<(Message, RenderedHtml)>, PebbleError> {
    let store = state.store.clone();
    store
        .with_blocking_async(move |store| {
            let message = match store.get_message(&message_id)? {
                Some(m) => m,
                None => return Ok(None),
            };

            let effective_mode = resolve_privacy_mode(store, &message, privacy_mode)?;
            let guard = PrivacyGuard::new();
            let rendered = guard.render_message_html(
                &message.body_html_raw,
                &message.body_text,
                &effective_mode,
            );
            Ok(Some((message, rendered)))
        })
        .await
}

pub async fn is_trusted_sender(
    state: State<std::sync::Arc<crate::state::AppState>>,
    account_id: String,
    email: String,
) -> std::result::Result<bool, PebbleError> {
    let store = state.store.clone();
    store
        .with_blocking_async(move |store| {
            Ok(store.is_trusted_sender(&account_id, &email)?.is_some())
        })
        .await
}

fn resolve_privacy_mode(
    store: &Store,
    message: &Message,
    privacy_mode: PrivacyMode,
) -> std::result::Result<PrivacyMode, PebbleError> {
    match privacy_mode {
        PrivacyMode::Strict | PrivacyMode::LoadOnce => {
            match store.is_trusted_sender(&message.account_id, &message.from_address)? {
                Some(TrustType::All) => Ok(PrivacyMode::TrustSender(message.from_address.clone())),
                Some(TrustType::Images) => Ok(PrivacyMode::LoadOnce),
                None => Ok(privacy_mode),
            }
        }
        PrivacyMode::TrustSender(sender) => {
            if sender.eq_ignore_ascii_case(&message.from_address) {
                Ok(PrivacyMode::TrustSender(message.from_address.clone()))
            } else {
                Ok(PrivacyMode::Strict)
            }
        }
        PrivacyMode::Off => Ok(privacy_mode),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pebble_core::{
        new_id, now_timestamp, Account, EmailAddress, Folder, FolderRole, FolderType, Message,
        ProviderType, TrustType, TrustedSender,
    };

    fn make_account(id: &str) -> Account {
        let now = now_timestamp();
        Account {
            id: id.to_string(),
            email: "me@example.com".to_string(),
            display_name: "Me".to_string(),
            color: None,
            provider: ProviderType::Imap,
            created_at: now,
            updated_at: now,
        }
    }

    fn make_message(account_id: &str, from_address: &str) -> Message {
        let now = now_timestamp();
        Message {
            id: new_id(),
            account_id: account_id.to_string(),
            remote_id: "remote-1".to_string(),
            message_id_header: None,
            in_reply_to: None,
            references_header: None,
            thread_id: Some("thread-1".to_string()),
            subject: "Subject".to_string(),
            snippet: "Snippet".to_string(),
            from_address: from_address.to_string(),
            from_name: "Trusted".to_string(),
            to_list: vec![EmailAddress {
                name: None,
                address: "me@example.com".to_string(),
            }],
            cc_list: vec![],
            bcc_list: vec![],
            body_text: "Body".to_string(),
            body_html_raw: "<p>Body</p>".to_string(),
            has_attachments: false,
            is_read: false,
            is_starred: false,
            is_draft: false,
            date: now,
            remote_version: None,
            is_deleted: false,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn make_folder(account_id: &str) -> Folder {
        Folder {
            id: new_id(),
            account_id: account_id.to_string(),
            remote_id: "INBOX".to_string(),
            name: "Inbox".to_string(),
            role: Some(FolderRole::Inbox),
            folder_type: FolderType::Folder,
            parent_id: None,
            color: None,
            is_system: true,
            sort_order: 0,
        }
    }

    fn store_with_trusted_sender(trust_type: TrustType) -> (Store, Message) {
        let store = Store::open_in_memory().unwrap();
        let account = make_account("account-1");
        store.insert_account(&account).unwrap();
        let folder = make_folder(&account.id);
        store.insert_folder(&folder).unwrap();
        let message = make_message(&account.id, "trusted@example.com");
        store.insert_message(&message, &[folder.id]).unwrap();
        store
            .trust_sender(&TrustedSender {
                account_id: account.id,
                email: "trusted@example.com".to_string(),
                trust_type,
                created_at: now_timestamp(),
            })
            .unwrap();
        (store, message)
    }

    #[test]
    fn all_trusted_sender_overrides_relaxed_mode() {
        let (store, message) = store_with_trusted_sender(TrustType::All);

        let mode = resolve_privacy_mode(&store, &message, PrivacyMode::LoadOnce).unwrap();

        assert!(
            matches!(mode, PrivacyMode::TrustSender(sender) if sender == "trusted@example.com")
        );
    }

    #[test]
    fn explicit_trust_sender_must_match_message_sender() {
        let store = Store::open_in_memory().unwrap();
        let message = make_message("account-1", "trusted@example.com");

        let mode = resolve_privacy_mode(
            &store,
            &message,
            PrivacyMode::TrustSender("other@example.com".to_string()),
        )
        .unwrap();

        assert!(matches!(mode, PrivacyMode::Strict));
    }

    #[test]
    fn explicit_trust_sender_allows_matching_message_sender() {
        let store = Store::open_in_memory().unwrap();
        let message = make_message("account-1", "Trusted@Example.com");

        let mode = resolve_privacy_mode(
            &store,
            &message,
            PrivacyMode::TrustSender("trusted@example.com".to_string()),
        )
        .unwrap();

        assert!(
            matches!(mode, PrivacyMode::TrustSender(sender) if sender == "Trusted@Example.com")
        );
    }
}
