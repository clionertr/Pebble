use axum::{
    extract::State,
    http::{header, HeaderValue, Method},
    response::sse::{Event, Sse},
    routing::{get, post},
    Router,
};
use pebble::{api, auth, gmail_realtime, middleware, rpc, snooze_watcher, state::AppState};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_logging(data_dir: &std::path::Path) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let log_dir = rpc::diagnostics::app_log_dir(data_dir);
    let file_guard = std::fs::create_dir_all(&log_dir).ok().map(|_| {
        let file_appender =
            tracing_appender::rolling::never(&log_dir, rpc::diagnostics::LOG_FILE_NAME);
        let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(file_writer)
            .with_ansi(false);
        (file_layer, guard)
    });

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let stdout_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);

    if let Some((file_layer, guard)) = file_guard {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(stdout_layer)
            .with(file_layer)
            .init();
        Some(guard)
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(stdout_layer)
            .init();
        None
    }
}

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

fn cors_layer() -> CorsLayer {
    let mut layer = CorsLayer::new()
        .allow_headers([header::CONTENT_TYPE])
        .allow_methods([
            Method::DELETE,
            Method::GET,
            Method::OPTIONS,
            Method::PATCH,
            Method::POST,
            Method::PUT,
        ]);

    if let Ok(origin) = std::env::var("ALLOWED_ORIGIN") {
        let origin = origin
            .parse::<HeaderValue>()
            .expect("ALLOWED_ORIGIN must be a valid HTTP origin");
        layer = layer.allow_origin(origin).allow_credentials(true);
    }

    layer
}

fn maybe_spawn_search_reindex(state: Arc<AppState>) {
    let schema_needs_reindex = state.search.needs_reindex();
    let db_count = match state.store.count_all_messages() {
        Ok(count) => count,
        Err(error) => {
            tracing::warn!("Failed to count messages before search reindex check: {error}");
            return;
        }
    };
    let index_count = state.search.doc_count();

    let reason = if schema_needs_reindex {
        Some("search index schema changed")
    } else if db_count > 0 && index_count == 0 {
        Some("search index is empty while messages exist")
    } else if index_count != db_count {
        Some("search index document count differs from message count")
    } else {
        None
    };

    let Some(reason) = reason else {
        return;
    };

    tracing::info!(
        reason,
        index_count,
        db_count,
        "Scheduling background search reindex"
    );

    let store = state.store.clone();
    let search = state.search.clone();
    tokio::spawn(async move {
        match tokio::task::spawn_blocking(move || rpc::indexing::do_reindex(&store, &search)).await
        {
            Ok(Ok(count)) => tracing::info!(count, "Background search reindex completed"),
            Ok(Err(error)) => tracing::error!("Background search reindex failed: {error}"),
            Err(error) => tracing::error!("Background search reindex task failed: {error}"),
        }
    });
}

#[tokio::main]
async fn main() {
    // Use a local ./data directory for VPS deployment
    let data_dir = PathBuf::from("./data");
    std::fs::create_dir_all(&data_dir).unwrap();
    let _log_guard = init_logging(&data_dir);

    let db_path = data_dir.join("pebble.db");
    let store = pebble_store::Store::open(&db_path).unwrap();

    let index_path = data_dir.join("index");
    let search = pebble_search::TantivySearch::open(&index_path).unwrap();

    let key_path = data_dir.join("pebble.key");
    let crypto = pebble_crypto::CryptoService::init(&key_path).unwrap();

    let attachments_dir = data_dir.join("attachments");
    std::fs::create_dir_all(&attachments_dir).unwrap();

    let (snooze_stop_tx, snooze_stop_rx) = std::sync::mpsc::channel::<()>();
    let password_hash = std::env::var("PEBBLE_PASSWORD_HASH")
        .expect("PEBBLE_PASSWORD_HASH must be set to a bcrypt password hash");
    let state = Arc::new(AppState::new(
        store,
        search,
        crypto,
        snooze_stop_tx,
        attachments_dir,
        password_hash,
    ));

    maybe_spawn_search_reindex(state.clone());

    let store_clone = state.store.clone();
    let state_clone = state.clone();
    tokio::spawn(async move {
        snooze_watcher::run_snooze_watcher(store_clone, state_clone, snooze_stop_rx).await;
    });
    gmail_realtime::spawn_gmail_watch_renewal_task(state.clone());

    let app = Router::new()
        .route("/events", get(sse_handler))
        .route(
            "/webhook/gmail",
            post(gmail_realtime::gmail_webhook_handler),
        )
        .route("/auth/login", get(auth::login_handler))
        .route("/auth/callback", get(auth::callback_handler))
        .merge(api::api_routes())
        .merge(api::auth_api::auth_routes())
        .merge(api::docs::docs_routes())
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ))
        .layer(cors_layer())
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
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
