use pebble_core::{now_timestamp, PebbleError, SnoozedMessage};

pub(crate) async fn snooze_message(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    message_id: String,
    until: i64,
    return_to: String,
) -> std::result::Result<(), PebbleError> {
    let snooze = SnoozedMessage {
        message_id,
        snoozed_at: now_timestamp(),
        unsnoozed_at: until,
        return_to,
    };
    state.store.snooze_message(&snooze)
}

pub(crate) async fn unsnooze_message(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    message_id: String,
) -> std::result::Result<(), PebbleError> {
    // Look up return_to before deleting so we can emit it in the event.
    let return_to = state
        .store
        .get_snoozed_message(&message_id)?
        .map(|s| s.return_to);
    state.store.unsnooze_message(&message_id)?;
    state.emit(
        "mail:unsnoozed",
        serde_json::json!({ "message_id": message_id, "return_to": return_to }),
    );
    Ok(())
}

pub(crate) async fn list_snoozed(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
) -> std::result::Result<Vec<SnoozedMessage>, PebbleError> {
    state.store.list_snoozed_messages()
}
