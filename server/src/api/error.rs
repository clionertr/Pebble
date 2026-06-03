// API error types — maps PebbleError to HTTP responses.
// Every /api endpoint returns errors through this type.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::error;

/// Standard API error response body: `{ "error": "message" }`
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: msg.into(),
        }
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: msg.into(),
        }
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: msg.into(),
        }
    }

    pub fn too_many_requests(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::TOO_MANY_REQUESTS,
            message: msg.into(),
        }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: msg.into(),
        }
    }

    pub fn status(&self) -> StatusCode {
        self.status
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(json!({ "error": self.message }));
        (self.status, body).into_response()
    }
}

// PebbleError → ApiError conversion
impl From<pebble_core::PebbleError> for ApiError {
    fn from(e: pebble_core::PebbleError) -> Self {
        match e {
            pebble_core::PebbleError::TokenExpired(msg) | pebble_core::PebbleError::Auth(msg) => {
                Self::unauthorized(msg)
            }
            pebble_core::PebbleError::Validation(msg) => Self::bad_request(msg),
            other => {
                error!("Internal error: {other}");
                Self::internal("Internal server error")
            }
        }
    }
}

// serde_json::Error → ApiError：序列化失败属于内部错误，不向客户端暴露细节
impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        error!("JSON serialization error: {e}");
        Self::internal("Internal server error")
    }
}
