use pebble_core::PebbleError;
use std::sync::atomic::Ordering;

pub async fn set_notifications_enabled(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    enabled: bool,
) -> std::result::Result<(), PebbleError> {
    state.notifications_enabled.store(enabled, Ordering::SeqCst);
    Ok(())
}
