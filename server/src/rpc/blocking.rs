use pebble_core::{PebbleError, Result};

/// 在阻塞线程运行无法直接异步化的任务，并统一 join-error 文案。
pub(crate) async fn run_blocking<F, T>(f: F) -> Result<T>
where
    F: FnOnce() -> Result<T> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| PebbleError::Internal(format!("spawn_blocking: {e}")))?
}
