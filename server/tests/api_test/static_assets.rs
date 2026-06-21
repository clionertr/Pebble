use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use pebble::{middleware, static_assets};
use tower::ServiceExt;

fn static_app(dist_dir: std::path::PathBuf) -> axum::Router {
    static_assets::static_assets_router(dist_dir).layer(axum::middleware::from_fn(
        middleware::security_headers_middleware,
    ))
}

async fn response_text(response: axum::response::Response) -> String {
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    String::from_utf8(body.to_vec()).unwrap()
}

#[tokio::test]
async fn serves_index_with_security_and_no_cache_headers() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("index.html"), "<h1>Pebble</h1>").unwrap();

    let response = static_app(dir.path().to_path_buf())
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CACHE_CONTROL).unwrap(),
        "no-cache"
    );
    assert_eq!(
        response
            .headers()
            .get(header::X_CONTENT_TYPE_OPTIONS)
            .unwrap(),
        "nosniff"
    );
    assert_eq!(response.headers().get("x-frame-options").unwrap(), "DENY");
    assert_eq!(
        response.headers().get("referrer-policy").unwrap(),
        "no-referrer"
    );
    assert!(response.headers().get("content-security-policy").is_some());

    assert!(response_text(response).await.contains("Pebble"));
}

#[tokio::test]
async fn spa_routes_fall_back_to_index_with_no_cache() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("index.html"), "<main>SPA shell</main>").unwrap();

    let response = static_app(dir.path().to_path_buf())
        .oneshot(
            Request::builder()
                .uri("/settings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CACHE_CONTROL).unwrap(),
        "no-cache"
    );
    assert!(response_text(response).await.contains("SPA shell"));
}

#[tokio::test]
async fn hashed_assets_use_immutable_cache() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("assets")).unwrap();
    std::fs::write(dir.path().join("index.html"), "<main>SPA shell</main>").unwrap();
    std::fs::write(
        dir.path().join("assets").join("index-abcd1234.js"),
        "console.log('ok');",
    )
    .unwrap();

    let response = static_app(dir.path().to_path_buf())
        .oneshot(
            Request::builder()
                .uri("/assets/index-abcd1234.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CACHE_CONTROL).unwrap(),
        "public, max-age=31536000, immutable"
    );
}

#[tokio::test]
async fn service_worker_is_not_cached_immutably() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("index.html"), "<main>SPA shell</main>").unwrap();
    std::fs::write(
        dir.path().join("pebble-sw.js"),
        "self.addEventListener('push', () => {})",
    )
    .unwrap();

    let response = static_app(dir.path().to_path_buf())
        .oneshot(
            Request::builder()
                .uri("/pebble-sw.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CACHE_CONTROL).unwrap(),
        "no-cache"
    );
}

#[tokio::test]
async fn missing_dist_keeps_backend_alive_with_clear_frontend_message() {
    let dir = tempfile::tempdir().unwrap();

    let response = static_app(dir.path().to_path_buf())
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = response_text(response).await;
    assert!(body.contains("dist/index.html"));
    assert!(body.contains("pnpm build:frontend"));
}

#[tokio::test]
async fn static_fallback_does_not_handle_non_get_requests() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("index.html"), "<main>SPA shell</main>").unwrap();

    let response = static_app(dir.path().to_path_buf())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/settings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn backend_reserved_paths_do_not_fall_back_to_index() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("index.html"), "<main>SPA shell</main>").unwrap();

    for path in ["/api/health", "/events", "/auth/unknown", "/webhook/gmail"] {
        let response = static_app(dir.path().to_path_buf())
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
    }
}
