mod account_colors;
mod events;
mod realtime;
mod snooze_watcher;
mod state;
mod rpc;
mod auth;

use axum::{
    routing::{get, post},
    extract::State,
    response::sse::{Event, Sse},
    Router,
};
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use std::path::PathBuf;
use std::sync::Arc;
use state::AppState;

async fn sse_handler(
    State(state): State<Arc<AppState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| match msg {
        Ok(payload) => {
            let json = serde_json::to_string(&payload.payload).unwrap_or_default();
            Some(Ok(Event::default().event(payload.event).data(json)))
        }
        Err(_) => None,
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("keep-alive-text"),
    )
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Use a local ./data directory for VPS deployment
    let data_dir = PathBuf::from("./data");
    std::fs::create_dir_all(&data_dir).unwrap();

    let db_path = data_dir.join("pebble.db");
    let store = pebble_store::Store::open(&db_path).unwrap();

    let index_path = data_dir.join("index");
    let search = pebble_search::TantivySearch::open(&index_path).unwrap();

    let key_path = data_dir.join("pebble.key");
    let crypto = pebble_crypto::CryptoService::init(&key_path).unwrap();

    let attachments_dir = data_dir.join("attachments");
    std::fs::create_dir_all(&attachments_dir).unwrap();

    let (snooze_stop_tx, snooze_stop_rx) = std::sync::mpsc::channel::<()>();
    let state = Arc::new(AppState::new(store, search, crypto, snooze_stop_tx, attachments_dir));

    let store_clone = state.store.clone();
    let state_clone = state.clone();
    tokio::spawn(async move {
        snooze_watcher::run_snooze_watcher(store_clone, state_clone, snooze_stop_rx).await;
    });

    let app = Router::new()
        .route("/rpc", post(rpc::dispatch::handle_rpc))
        .route("/rpc/batch", post(rpc::dispatch::handle_rpc_batch))
        .route("/events", get(sse_handler))
        .route("/auth/login", get(auth::login_handler))
        .route("/auth/callback", get(auth::callback_handler))
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
        )
        .layer(tower_http::compression::CompressionLayer::new())
        .with_state(state);

    let host = std::env::var("PEBBLE_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PEBBLE_PORT")
        .map(|v| v.parse().unwrap_or(3000))
        .unwrap_or(3000);

    let addr_str = format!("{}:{}", host, port);
    let addr: SocketAddr = addr_str.parse().expect("Invalid address");
    tracing::info!("listening on {}", addr);
    
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}