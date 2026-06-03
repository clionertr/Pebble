use pebble_core::{new_id, Folder, FolderRole, FolderType, PebbleError, ProviderType};
use pebble_mail::should_hide_outlook_folder;

fn provider_folders_have_arrived(folders: &[Folder]) -> bool {
    folders
        .iter()
        .any(|folder| !folder.remote_id.starts_with("__local_"))
}

fn should_seed_local_archive(folders: &[Folder]) -> bool {
    let has_archive = folders
        .iter()
        .any(|folder| folder.role == Some(FolderRole::Archive));

    provider_folders_have_arrived(folders) && !has_archive
}

fn should_hide_stored_outlook_folder(folder: &Folder) -> bool {
    folder.role.is_none()
        && !folder.remote_id.starts_with("__local_")
        && should_hide_outlook_folder(Some(&folder.name), None)
}

fn filter_display_folders(provider: Option<&ProviderType>, folders: Vec<Folder>) -> Vec<Folder> {
    if !matches!(provider, Some(ProviderType::Outlook)) {
        return folders;
    }

    folders
        .into_iter()
        .filter(|folder| !should_hide_stored_outlook_folder(folder))
        .collect()
}

pub(crate) async fn list_folders(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    account_id: String,
) -> std::result::Result<Vec<Folder>, PebbleError> {
    state
        .store
        .with_blocking_async(move |store| {
            let provider = store
                .get_account(&account_id)?
                .map(|account| account.provider);
            let folders = store.list_folders(&account_id)?;

            if !provider_folders_have_arrived(&folders) {
                return Ok(Vec::new());
            }

            // 仅在服务商文件夹已到达后补本地归档文件夹；首次 OAuth 登录时文件夹
            // 可能仍在同步，提前返回空列表可让侧边栏保留占位文件夹，避免缓存成
            // 误导性的“只有 Archive”状态。
            if should_seed_local_archive(&folders) {
                let archive = Folder {
                    id: new_id(),
                    account_id: account_id.clone(),
                    remote_id: "__local_archive__".to_string(),
                    name: "Archive".to_string(),
                    folder_type: FolderType::Folder,
                    role: Some(FolderRole::Archive),
                    parent_id: None,
                    color: None,
                    is_system: true,
                    sort_order: 3,
                };
                if let Err(e) = store.insert_folder(&archive) {
                    tracing::warn!(
                        "Failed to seed local archive folder for account {account_id}: {e}"
                    );
                }
                let folders = store.list_folders(&account_id)?;
                return Ok(filter_display_folders(provider.as_ref(), folders));
            }

            Ok(filter_display_folders(provider.as_ref(), folders))
        })
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(role: FolderRole, remote_id: &str) -> Folder {
        Folder {
            id: new_id(),
            account_id: "account-1".to_string(),
            remote_id: remote_id.to_string(),
            name: remote_id.to_string(),
            folder_type: FolderType::Folder,
            role: Some(role),
            parent_id: None,
            color: None,
            is_system: true,
            sort_order: 0,
        }
    }

    #[test]
    fn archive_seed_waits_until_provider_folders_exist() {
        assert!(!provider_folders_have_arrived(&[]));
        assert!(!provider_folders_have_arrived(&[folder(
            FolderRole::Archive,
            "__local_archive__"
        )]));
        assert!(provider_folders_have_arrived(&[folder(
            FolderRole::Inbox,
            "INBOX"
        )]));

        assert!(!should_seed_local_archive(&[]));
        assert!(!should_seed_local_archive(&[folder(
            FolderRole::Sent,
            "__local_outbox__"
        )]));
        assert!(should_seed_local_archive(&[folder(
            FolderRole::Inbox,
            "INBOX"
        )]));
        assert!(!should_seed_local_archive(&[folder(
            FolderRole::Archive,
            "__local_archive__"
        )]));
    }

    #[test]
    fn display_filter_hides_outlook_service_folders_but_keeps_local_outbox() {
        let mut conversation_history = folder(FolderRole::Inbox, "conversation-history-id");
        conversation_history.role = None;
        conversation_history.name = "对话历史记录".to_string();

        let mut remote_outbox = folder(FolderRole::Inbox, "remote-outbox-id");
        remote_outbox.role = None;
        remote_outbox.name = "发件箱".to_string();

        let mut local_outbox = folder(FolderRole::Inbox, "__local_outbox__");
        local_outbox.role = None;
        local_outbox.name = "Outbox".to_string();

        let inbox = folder(FolderRole::Inbox, "inbox-id");
        let filtered = filter_display_folders(
            Some(&pebble_core::ProviderType::Outlook),
            vec![
                conversation_history,
                remote_outbox,
                local_outbox.clone(),
                inbox.clone(),
            ],
        );

        assert_eq!(
            filtered
                .iter()
                .map(|folder| folder.remote_id.as_str())
                .collect::<Vec<_>>(),
            vec!["__local_outbox__", "inbox-id"]
        );
    }
}
