use crate::Store;
use pebble_core::Result;
use rusqlite::{params, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationDeviceStatus {
    Active,
    Paused,
}

impl NotificationDeviceStatus {
    fn from_str(value: &str) -> Self {
        match value {
            "active" => Self::Active,
            _ => Self::Paused,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationDevice {
    pub id: String,
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
    pub device_name: String,
    pub user_agent: Option<String>,
    pub status: NotificationDeviceStatus,
    pub session_id: Option<String>,
    pub session_expires_at: Option<i64>,
    pub last_active_at: i64,
    pub summary_sent_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone)]
pub struct UpsertNotificationDevice {
    pub id: String,
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
    pub device_name: String,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
    pub session_expires_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnreadInboxSummary {
    pub unread_count: u32,
    pub sample_senders: Vec<String>,
}

fn row_to_device(row: &Row<'_>) -> rusqlite::Result<NotificationDevice> {
    let status: String = row.get(6)?;
    Ok(NotificationDevice {
        id: row.get(0)?,
        endpoint: row.get(1)?,
        p256dh: row.get(2)?,
        auth: row.get(3)?,
        device_name: row.get(4)?,
        user_agent: row.get(5)?,
        status: NotificationDeviceStatus::from_str(&status),
        session_id: row.get(7)?,
        session_expires_at: row.get(8)?,
        last_active_at: row.get(9)?,
        summary_sent_at: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

const DEVICE_SELECT: &str = "id, endpoint, p256dh, auth, device_name, user_agent, status, \
    session_id, session_expires_at, last_active_at, summary_sent_at, created_at, updated_at";

impl Store {
    pub fn upsert_notification_device(
        &self,
        input: UpsertNotificationDevice,
    ) -> Result<NotificationDevice> {
        self.with_write(|conn| {
            let id = input.id.clone();
            let now = pebble_core::now_timestamp();
            conn.execute(
                "INSERT INTO notification_devices (
                     id, endpoint, p256dh, auth, device_name, user_agent, status,
                     session_id, session_expires_at, last_active_at, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'active', ?7, ?8, ?9, ?9, ?9)
                 ON CONFLICT(id) DO UPDATE SET
                     endpoint = excluded.endpoint,
                     p256dh = excluded.p256dh,
                     auth = excluded.auth,
                     device_name = excluded.device_name,
                     user_agent = excluded.user_agent,
                     status = 'active',
                     session_id = excluded.session_id,
                     session_expires_at = excluded.session_expires_at,
                     last_active_at = excluded.last_active_at,
                     updated_at = excluded.updated_at",
                params![
                    input.id,
                    input.endpoint,
                    input.p256dh,
                    input.auth,
                    input.device_name,
                    input.user_agent,
                    input.session_id,
                    input.session_expires_at,
                    now,
                ],
            )?;
            conn.query_row(
                &format!("SELECT {DEVICE_SELECT} FROM notification_devices WHERE id = ?1"),
                params![id],
                row_to_device,
            )
            .map_err(Into::into)
        })
    }

    pub fn list_notification_devices(&self) -> Result<Vec<NotificationDevice>> {
        self.with_read(|conn| {
            let mut stmt = conn.prepare(&format!(
                "SELECT {DEVICE_SELECT} FROM notification_devices ORDER BY last_active_at DESC, created_at DESC"
            ))?;
            let rows = stmt.query_map([], row_to_device)?;
            let mut devices = Vec::new();
            for row in rows {
                devices.push(row?);
            }
            Ok(devices)
        })
    }

    pub fn get_notification_device(&self, id: &str) -> Result<Option<NotificationDevice>> {
        self.with_read(|conn| {
            conn.query_row(
                &format!("SELECT {DEVICE_SELECT} FROM notification_devices WHERE id = ?1"),
                params![id],
                row_to_device,
            )
            .optional()
            .map_err(Into::into)
        })
    }

    pub fn list_active_notification_devices(&self, now: i64) -> Result<Vec<NotificationDevice>> {
        self.with_read(|conn| {
            let mut stmt = conn.prepare(&format!(
                "SELECT {DEVICE_SELECT} FROM notification_devices
                 WHERE status = 'active'
                   AND (session_expires_at IS NULL OR session_expires_at > ?1)
                 ORDER BY last_active_at DESC"
            ))?;
            let rows = stmt.query_map(params![now], row_to_device)?;
            let mut devices = Vec::new();
            for row in rows {
                devices.push(row?);
            }
            Ok(devices)
        })
    }

    pub fn rename_notification_device(&self, id: &str, device_name: &str) -> Result<()> {
        self.with_write(|conn| {
            conn.execute(
                "UPDATE notification_devices SET device_name = ?1, updated_at = ?2 WHERE id = ?3",
                params![device_name, pebble_core::now_timestamp(), id],
            )?;
            Ok(())
        })
    }

    pub fn delete_notification_device(&self, id: &str) -> Result<()> {
        self.with_write(|conn| {
            conn.execute(
                "DELETE FROM notification_devices WHERE id = ?1",
                params![id],
            )?;
            Ok(())
        })
    }

    pub fn delete_notification_devices_by_session(&self, session_id: &str) -> Result<()> {
        self.with_write(|conn| {
            conn.execute(
                "DELETE FROM notification_devices WHERE session_id = ?1",
                params![session_id],
            )?;
            Ok(())
        })
    }

    pub fn pause_expired_notification_devices(&self, now: i64) -> Result<()> {
        self.with_write(|conn| {
            conn.execute(
                "UPDATE notification_devices SET status = 'paused', updated_at = ?1
                 WHERE status = 'active' AND session_expires_at IS NOT NULL AND session_expires_at <= ?1",
                params![now],
            )?;
            Ok(())
        })
    }

    pub fn pause_all_notification_devices(&self) -> Result<()> {
        self.with_write(|conn| {
            conn.execute(
                "UPDATE notification_devices SET status = 'paused', updated_at = ?1 WHERE status = 'active'",
                params![pebble_core::now_timestamp()],
            )?;
            Ok(())
        })
    }

    pub fn mark_notification_summary_sent(&self, id: &str) -> Result<()> {
        self.with_write(|conn| {
            let now = pebble_core::now_timestamp();
            conn.execute(
                "UPDATE notification_devices SET summary_sent_at = ?1, updated_at = ?1 WHERE id = ?2",
                params![now, id],
            )?;
            Ok(())
        })
    }

    pub fn unread_inbox_summary(&self) -> Result<UnreadInboxSummary> {
        self.with_read(|conn| {
            let unread_count = conn.query_row(
                "SELECT COUNT(DISTINCT m.id)
                 FROM messages m
                 JOIN message_folders mf ON mf.message_id = m.id
                 JOIN folders f ON f.id = mf.folder_id
                 WHERE f.role = 'inbox' AND m.is_read = 0 AND m.is_deleted = 0",
                [],
                |row| row.get::<_, u32>(0),
            )?;

            let mut stmt = conn.prepare(
                "SELECT CASE WHEN m.from_name != '' THEN m.from_name ELSE m.from_address END
                 FROM messages m
                 JOIN message_folders mf ON mf.message_id = m.id
                 JOIN folders f ON f.id = mf.folder_id
                 WHERE f.role = 'inbox' AND m.is_read = 0 AND m.is_deleted = 0
                 ORDER BY m.date DESC, m.id ASC
                 LIMIT 3",
            )?;
            let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
            let mut sample_senders = Vec::new();
            for row in rows {
                let sender = row?;
                if !sender.trim().is_empty() {
                    sample_senders.push(sender);
                }
            }

            Ok(UnreadInboxSummary {
                unread_count,
                sample_senders,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pebble_core::{
        now_timestamp, Account, Folder, FolderRole, FolderType, Message, ProviderType,
    };

    fn upsert_input(id: &str) -> UpsertNotificationDevice {
        UpsertNotificationDevice {
            id: id.to_string(),
            endpoint: format!("https://push.example/{id}"),
            p256dh: "p256dh".to_string(),
            auth: "auth".to_string(),
            device_name: "Firefox on Linux".to_string(),
            user_agent: Some("agent".to_string()),
            session_id: Some("session".to_string()),
            session_expires_at: Some(now_timestamp() + 3600),
        }
    }

    #[test]
    fn notification_device_upsert_and_pause() {
        let store = Store::open_in_memory().unwrap();
        let device = store
            .upsert_notification_device(upsert_input("device-1"))
            .unwrap();
        assert_eq!(device.status, NotificationDeviceStatus::Active);

        store.pause_all_notification_devices().unwrap();
        let device = store.get_notification_device("device-1").unwrap().unwrap();
        assert_eq!(device.status, NotificationDeviceStatus::Paused);
    }

    #[test]
    fn unread_inbox_summary_counts_only_unread_inbox() {
        let store = Store::open_in_memory().unwrap();
        let now = now_timestamp();
        let account = Account {
            id: "account-1".to_string(),
            email: "me@example.com".to_string(),
            display_name: "Me".to_string(),
            color: None,
            provider: ProviderType::Imap,
            created_at: now,
            updated_at: now,
        };
        store.insert_account(&account).unwrap();
        let inbox = Folder {
            id: "inbox-1".to_string(),
            account_id: account.id.clone(),
            remote_id: "INBOX".to_string(),
            name: "Inbox".to_string(),
            folder_type: FolderType::Folder,
            role: Some(FolderRole::Inbox),
            parent_id: None,
            color: None,
            is_system: true,
            sort_order: 0,
        };
        store.insert_folder(&inbox).unwrap();
        let message = Message {
            id: "message-1".to_string(),
            account_id: account.id.clone(),
            remote_id: "remote-1".to_string(),
            message_id_header: None,
            in_reply_to: None,
            references_header: None,
            thread_id: None,
            subject: "Hello".to_string(),
            snippet: "Snippet".to_string(),
            from_address: "sender@example.com".to_string(),
            from_name: "Sender".to_string(),
            to_list: vec![],
            cc_list: vec![],
            bcc_list: vec![],
            body_text: "Body".to_string(),
            body_html_raw: String::new(),
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
        };
        store
            .insert_message(&message, std::slice::from_ref(&inbox.id))
            .unwrap();

        let summary = store.unread_inbox_summary().unwrap();
        assert_eq!(summary.unread_count, 1);
        assert_eq!(summary.sample_senders, vec!["Sender".to_string()]);
    }
}
