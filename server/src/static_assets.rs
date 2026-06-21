use axum::{
    body::Body,
    http::{header, HeaderMap, HeaderValue, Method, Request, Response, StatusCode},
    response::{Html, IntoResponse},
    routing::any,
    Router,
};
use std::path::PathBuf;
use tower::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};

const FRONTEND_MISSING_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Pebble frontend files are missing</title>
</head>
<body>
  <h1>Pebble frontend files are missing</h1>
  <p>The backend is running, but <code>dist/index.html</code> was not found.</p>
  <p>For source runs, execute <code>pnpm build:frontend</code> before starting Pebble, or use the Docker image.</p>
</body>
</html>"#;

const CACHE_NO_CACHE: &str = "no-cache";
const CACHE_IMMUTABLE: &str = "public, max-age=31536000, immutable";

/// 默认前端构建产物目录。Docker 镜像会把 Vite 产物复制到这个相对路径。
pub fn default_frontend_dist_dir() -> PathBuf {
    PathBuf::from("dist")
}

pub fn static_assets_router<S>(dist_dir: PathBuf) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new().fallback(any(move |request| {
        serve_frontend_asset(request, dist_dir.clone())
    }))
}

pub async fn serve_frontend_asset(request: Request<Body>, dist_dir: PathBuf) -> Response<Body> {
    let method = request.method().clone();
    if method != Method::GET && method != Method::HEAD {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }

    let path = request.uri().path().to_string();
    if is_reserved_backend_path(&path) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let index_path = dist_dir.join("index.html");
    if !index_path.is_file() {
        return frontend_missing_response();
    }

    let service = ServeDir::new(&dist_dir).fallback(ServeFile::new(index_path));

    match service.oneshot(request).await {
        Ok(response) => {
            let mut response = response.map(Body::new);
            apply_static_cache_header(response.headers_mut(), &path);
            response
        }
        Err(error) => {
            tracing::error!(
                dist_dir = %dist_dir.display(),
                "Failed to serve frontend static asset: {error}"
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn frontend_missing_response() -> Response<Body> {
    let mut response =
        (StatusCode::SERVICE_UNAVAILABLE, Html(FRONTEND_MISSING_HTML)).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(CACHE_NO_CACHE),
    );
    response
}

fn is_reserved_backend_path(path: &str) -> bool {
    path == "/events"
        || path == "/api"
        || path.starts_with("/api/")
        || path == "/auth"
        || path.starts_with("/auth/")
        || path == "/webhook"
        || path.starts_with("/webhook/")
}

fn apply_static_cache_header(headers: &mut HeaderMap, path: &str) {
    let value = if path.starts_with("/assets/") {
        CACHE_IMMUTABLE
    } else {
        CACHE_NO_CACHE
    };
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(value));
}

#[cfg(test)]
mod tests {
    use super::{
        apply_static_cache_header, is_reserved_backend_path, CACHE_IMMUTABLE, CACHE_NO_CACHE,
    };
    use axum::http::{header, HeaderMap};

    #[test]
    fn reserved_backend_paths_do_not_fall_back_to_spa() {
        for path in [
            "/api",
            "/api/health",
            "/events",
            "/auth/login",
            "/webhook/gmail",
        ] {
            assert!(is_reserved_backend_path(path), "{path} should be reserved");
        }

        for path in ["/", "/inbox", "/settings", "/assets/index.js"] {
            assert!(!is_reserved_backend_path(path), "{path} should be static");
        }
    }

    #[test]
    fn cache_header_distinguishes_hashed_assets_from_entrypoints() {
        let mut headers = HeaderMap::new();

        apply_static_cache_header(&mut headers, "/assets/index-abcd.js");
        assert_eq!(headers.get(header::CACHE_CONTROL).unwrap(), CACHE_IMMUTABLE);

        apply_static_cache_header(&mut headers, "/settings");
        assert_eq!(headers.get(header::CACHE_CONTROL).unwrap(), CACHE_NO_CACHE);

        apply_static_cache_header(&mut headers, "/pebble-sw.js");
        assert_eq!(headers.get(header::CACHE_CONTROL).unwrap(), CACHE_NO_CACHE);
    }
}
