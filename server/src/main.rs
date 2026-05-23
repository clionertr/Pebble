use axum::{
    extract::State,
    http::{header, HeaderValue, Method},
    response::sse::{Event, Sse},
    routing::{get, post},
    Router,
};
use pebble::{api, auth, gmail_realtime, middleware, rpc, snooze_watcher, state::AppState};
use std::convert::Infallible;
use std::io::Read;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
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

fn parse_dotenv_line(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }

    let (key, value) = line.split_once('=')?;
    let key = key.trim();
    if key.is_empty() || key.starts_with('#') {
        return None;
    }

    let value = value.trim();
    let value = if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        &value[1..value.len() - 1]
    } else {
        value
    };

    Some((key.to_string(), value.to_string()))
}

fn load_dotenv_if_present(path: &Path) {
    let Ok(contents) = std::fs::read_to_string(path) else {
        return;
    };

    for line in contents.lines() {
        let Some((key, value)) = parse_dotenv_line(line) else {
            continue;
        };
        if std::env::var_os(&key).is_none() {
            std::env::set_var(key, value);
        }
    }
}

fn password_hash_from_env_or_exit() -> String {
    let password_hash = match std::env::var("PEBBLE_PASSWORD_HASH") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => {
            eprintln!("PEBBLE_PASSWORD_HASH must be set to a bcrypt password hash.");
            eprintln!("Create one with: printf '%s' 'your-password' | pebble hash-password");
            eprintln!("For source runs, put it in .env or export it in the shell.");
            std::process::exit(2);
        }
    };

    if bcrypt::verify("", &password_hash).is_err() {
        eprintln!("PEBBLE_PASSWORD_HASH is not a valid bcrypt hash.");
        eprintln!("Create one with: printf '%s' 'your-password' | pebble hash-password");
        eprintln!(
            "Direct source runs use single $ characters; Docker Compose .env files use $$ escaping."
        );
        std::process::exit(2);
    }

    password_hash
}

fn open_search_or_exit(index_path: &Path) -> pebble_search::TantivySearch {
    match pebble_search::TantivySearch::open(index_path) {
        Ok(search) => search,
        Err(error) => {
            let message = error.to_string();
            eprintln!(
                "Failed to open search index at {}: {message}",
                index_path.display()
            );
            if message.contains("LockBusy") || message.contains("index lock") {
                eprintln!("Another Pebble process is probably using this data directory.");
                eprintln!("Stop the old backend before starting a new one, then retry.");
                eprintln!("If no Pebble process is running, reboot the server or remove the stale search index lock file under data/index after backing up data.");
            }
            std::process::exit(1);
        }
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

fn hash_password_from_args(args: &[String]) -> Result<String, String> {
    let password = match args {
        [] => {
            let mut input = String::new();
            std::io::stdin()
                .read_to_string(&mut input)
                .map_err(|error| format!("Failed to read password from stdin: {error}"))?;
            input.trim_end_matches(&['\r', '\n'][..]).to_string()
        }
        [password] => password.clone(),
        _ => {
            return Err(
                "Usage: pebble hash-password [password]\n       printf '%s' \"$PASSWORD\" | pebble hash-password"
                    .to_string(),
            )
        }
    };

    if password.is_empty() {
        return Err("Password cannot be empty".to_string());
    }

    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|error| format!("Failed to hash password: {error}"))
}

fn handle_cli_command() -> bool {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let Some(command) = args.first() else {
        return false;
    };

    if command != "hash-password" {
        return false;
    }

    match hash_password_from_args(&args[1..]) {
        Ok(hash) => {
            println!("{hash}");
            true
        }
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    }
}

#[tokio::main]
async fn main() {
    if handle_cli_command() {
        return;
    }

    load_dotenv_if_present(Path::new(".env"));

    // Use a local ./data directory for VPS deployment
    let data_dir = PathBuf::from("./data");
    std::fs::create_dir_all(&data_dir).unwrap();
    let _log_guard = init_logging(&data_dir);

    let db_path = data_dir.join("pebble.db");
    let store = pebble_store::Store::open(&db_path).unwrap();
    if let Err(error) = store.pause_all_notification_devices() {
        tracing::warn!("Failed to pause notification devices after restart: {error}");
    }

    let index_path = data_dir.join("index");
    let search = open_search_or_exit(&index_path);

    let key_path = data_dir.join("pebble.key");
    let crypto = pebble_crypto::CryptoService::init(&key_path).unwrap();

    let attachments_dir = data_dir.join("attachments");
    std::fs::create_dir_all(&attachments_dir).unwrap();

    let (snooze_stop_tx, snooze_stop_rx) = std::sync::mpsc::channel::<()>();
    let password_hash = password_hash_from_env_or_exit();
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

#[cfg(test)]
mod tests {
    use super::{hash_password_from_args, parse_dotenv_line};

    #[test]
    fn parse_dotenv_line_reads_plain_and_quoted_values() {
        assert_eq!(
            parse_dotenv_line("PEBBLE_PASSWORD_HASH='$2b$12$abc'"),
            Some(("PEBBLE_PASSWORD_HASH".to_string(), "$2b$12$abc".to_string()))
        );
        assert_eq!(
            parse_dotenv_line(" PEBBLE_HOST = 0.0.0.0 "),
            Some(("PEBBLE_HOST".to_string(), "0.0.0.0".to_string()))
        );
        assert_eq!(parse_dotenv_line("# PEBBLE_PORT=3000"), None);
    }

    #[test]
    fn hash_password_from_argument_verifies_with_bcrypt() {
        let hash = hash_password_from_args(&["test-password".to_string()]).unwrap();

        assert!(bcrypt::verify("test-password", &hash).unwrap());
    }

    #[test]
    fn hash_password_rejects_empty_password() {
        let error = hash_password_from_args(&["".to_string()]).unwrap_err();

        assert_eq!(error, "Password cannot be empty");
    }

    #[test]
    fn hash_password_rejects_extra_arguments() {
        let error = hash_password_from_args(&["one".to_string(), "two".to_string()]).unwrap_err();

        assert!(error.starts_with("Usage: pebble hash-password"));
    }
}
