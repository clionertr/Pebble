use crate::realtime::SyncTrigger;
use pebble_crypto::CryptoService;
use pebble_search::TantivySearch;
use pebble_store::Store;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, watch, Mutex};

#[derive(Clone, Debug)]
pub struct EventPayload {
    pub event: String,
    pub payload: serde_json::Value,
}

pub struct SyncHandle {
    pub stop_tx: watch::Sender<bool>,
    pub trigger_tx: mpsc::UnboundedSender<SyncTrigger>,
    pub task: tokio::task::JoinHandle<()>,
}

pub struct AppState {
    pub store: Arc<Store>,
    pub search: Arc<TantivySearch>,
    pub crypto: Arc<CryptoService>,
    pub sync_handles: Mutex<HashMap<String, SyncHandle>>,
    pub oauth_states: Mutex<HashMap<String, crate::auth::OAuthSession>>,
    /// Kept alive so the snooze watcher's `stop_rx` remains open.
    #[allow(dead_code)]
    pub snooze_stop_tx: std::sync::mpsc::Sender<()>,
    pub attachments_dir: PathBuf,
    pub notifications_enabled: Arc<AtomicBool>,
    pub tx: broadcast::Sender<EventPayload>,
}

impl AppState {
    pub fn new(
        store: Store,
        search: TantivySearch,
        crypto: CryptoService,
        snooze_stop_tx: std::sync::mpsc::Sender<()>,
        attachments_dir: PathBuf,
    ) -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self {
            store: Arc::new(store),
            search: Arc::new(search),
            crypto: Arc::new(crypto),
            sync_handles: Mutex::new(HashMap::new()),
            oauth_states: Mutex::new(HashMap::new()),
            snooze_stop_tx,
            attachments_dir,
            notifications_enabled: Arc::new(AtomicBool::new(true)),
            tx,
        }
    }

    pub fn emit<S: Into<String>, P: Serialize>(&self, event: S, payload: P) {
        if let Ok(value) = serde_json::to_value(payload) {
            let _ = self.tx.send(EventPayload {
                event: event.into(),
                payload: value,
            });
        }
    }
}
