//! 线程、搜索、看板和延迟邮件 API 路由聚合。

mod kanban;
mod search;
mod snooze;
mod thread_reads;

use crate::state::AppState;
use axum::Router;
use std::sync::Arc;

pub(super) const MAX_PAGE_LIMIT: usize = 500;

pub fn thread_routes() -> Router<Arc<AppState>> {
    Router::new()
        .merge(thread_reads::routes())
        .merge(search::routes())
        .merge(kanban::routes())
        .merge(snooze::routes())
}
