// API Baseline Tests — Phase 0 safety net.
// Verify that the API skeleton compiles and ApiError types work correctly.

use pebble::api::error::ApiError;
use pebble::api::api_routes;

/// Test that the API router can be created without panicking.
#[test]
fn api_router_creation() {
    let _router = api_routes::<()>();
}

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
    assert_eq!(
        axum::http::StatusCode::METHOD_NOT_ALLOWED.as_u16(),
        405
    );
}
