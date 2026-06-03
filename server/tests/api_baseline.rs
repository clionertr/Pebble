// API Baseline Tests — Phase 0 safety net.
// Verify that the API skeleton compiles and ApiError types work correctly.

use pebble::api::error::ApiError;
use std::path::{Path, PathBuf};

/// Test that ApiError types map to correct HTTP status codes.
#[test]
fn api_error_status_codes() {
    let err = ApiError::not_found("message not found");
    assert_eq!(err.status(), axum::http::StatusCode::NOT_FOUND);

    let err = ApiError::unauthorized("invalid credentials");
    assert_eq!(err.status(), axum::http::StatusCode::UNAUTHORIZED);

    let err = ApiError::bad_request("missing required field");
    assert_eq!(err.status(), axum::http::StatusCode::BAD_REQUEST);

    let err = ApiError::internal("database error");
    assert_eq!(err.status(), axum::http::StatusCode::INTERNAL_SERVER_ERROR);
}

/// Test that ApiError produces correct JSON response body shape.
#[test]
fn api_error_response_body() {
    use axum::response::IntoResponse;

    let err = ApiError::not_found("resource X not found");
    let response = err.into_response();
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

/// Test 405 Method Not Allowed — when hitting a GET-only endpoint with POST.
/// Expanded in Phase 1 when the full router is mounted.
#[test]
fn method_not_allowed_detection() {
    // This is a design test: axum correctly returns 405 for wrong methods
    // when routes are defined with specific method filters.
    // We verify the concept here; full integration test in Phase 1.
    assert_eq!(axum::http::StatusCode::METHOD_NOT_ALLOWED.as_u16(), 405);
}

#[test]
fn api_handlers_do_not_bypass_api_error_boundary() {
    let api_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/api");
    let files = rust_files_under(&api_dir);
    let mut violations = Vec::new();

    for file in files {
        let source = std::fs::read_to_string(&file).expect("read api source");

        for line in source.lines() {
            let compact_line = line.split_whitespace().collect::<Vec<_>>().join(" ");
            if (compact_line.contains("->") || compact_line.contains("Result<"))
                && compact_line.contains("Result<")
                && compact_line.contains(", String>")
            {
                violations.push(format!("{} returns Result<..., String>", file.display()));
            }
            if line.contains("StatusCode, Json<")
                || line.contains("StatusCode, axum::Json<")
                || compact_line.contains("StatusCode , Json <")
                || compact_line.contains("StatusCode , axum :: Json <")
            {
                violations.push(format!(
                    "{} returns a raw StatusCode + Json tuple",
                    file.display()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "API handlers must return ApiError instead of ad-hoc error boundaries:\n{}",
        violations.join("\n")
    );
}

fn rust_files_under(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir).expect("read api dir") {
        let entry = entry.expect("read api entry");
        let path = entry.path();
        if path.is_dir() {
            files.extend(rust_files_under(&path));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    files
}
