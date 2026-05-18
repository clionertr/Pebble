use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use pebble_core::PebbleError;
use pebble_mail::ImapProvider;
use tokio::sync::Mutex;

use crate::rpc::messages::load_imap_config;
use crate::state::AppState;

const DEFAULT_TTL: Duration = Duration::from_secs(120);

struct PoolEntry {
    provider: Arc<ImapProvider>,
    created_at: Instant,
}

pub struct ImapConnectionPool {
    entries: Mutex<HashMap<String, PoolEntry>>,
    ttl: Duration,
}

impl Default for ImapConnectionPool {
    fn default() -> Self {
        Self::new()
    }
}

impl ImapConnectionPool {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            ttl: DEFAULT_TTL,
        }
    }

    pub async fn get(
        &self,
        state: &AppState,
        account_id: &str,
    ) -> Result<Arc<ImapProvider>, PebbleError> {
        {
            let entries = self.entries.lock().await;
            if let Some(entry) = entries.get(account_id) {
                if entry.created_at.elapsed() < self.ttl {
                    return Ok(entry.provider.clone());
                }
            }
        }

        let config = load_imap_config(&state.store, &state.crypto, account_id)?;
        let provider = ImapProvider::new(config);
        provider.connect().await?;
        let entry = PoolEntry {
            provider: Arc::new(provider),
            created_at: Instant::now(),
        };

        let mut entries = self.entries.lock().await;
        entries.insert(account_id.to_string(), entry);
        Ok(entries.get(account_id).unwrap().provider.clone())
    }

    pub async fn evict(&self, account_id: &str) {
        let mut entries = self.entries.lock().await;
        entries.remove(account_id);
    }
}
