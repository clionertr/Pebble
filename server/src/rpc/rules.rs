use pebble_core::{new_id, now_timestamp, PebbleError, Rule};

pub(crate) async fn create_rule(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    name: String,
    priority: i32,
    conditions: String,
    actions: String,
) -> std::result::Result<Rule, PebbleError> {
    let now = now_timestamp();
    let rule = Rule {
        id: new_id(),
        name,
        priority,
        conditions,
        actions,
        is_enabled: true,
        created_at: now,
        updated_at: now,
    };
    state.store.insert_rule(&rule)?;
    Ok(rule)
}

pub(crate) async fn list_rules(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
) -> std::result::Result<Vec<Rule>, PebbleError> {
    state.store.list_rules()
}

pub(crate) async fn update_rule(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    rule: Rule,
) -> std::result::Result<(), PebbleError> {
    state.store.update_rule(&rule)
}

pub(crate) async fn delete_rule(
    state: axum::extract::State<std::sync::Arc<crate::state::AppState>>,
    rule_id: String,
) -> std::result::Result<(), PebbleError> {
    state.store.delete_rule(&rule_id)
}
