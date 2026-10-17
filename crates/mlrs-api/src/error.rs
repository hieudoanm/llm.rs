//! Error envelope for the mlrs HTTP API.
//!
//! Every error response is the flat `{"error": {...}}` shape from the Python
//! service. The request id is carried in a task-local set by the request-id
//! middleware, so `IntoResponse` can attach it without every handler having to
//! thread it through.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Map, Value};

use mlrs_core::error::MlrsError;

tokio::task_local! {
    /// The correlation id of the request currently being served.
    pub static REQUEST_ID: String;
}

/// The correlation id of the current request, or an empty string outside one.
pub fn current_request_id() -> String {
    REQUEST_ID.try_with(|id| id.clone()).unwrap_or_default()
}

#[derive(Debug, Clone)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
}

impl ApiError {
    pub fn new(status: u16, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(400, "VALIDATION_ERROR", message)
    }

    pub fn not_found(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(404, code, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(500, "INTERNAL_ERROR", message)
    }

    pub fn body(&self) -> Value {
        let mut error = Map::new();
        error.insert("code".to_string(), Value::String(self.code.clone()));
        error.insert("message".to_string(), Value::String(self.message.clone()));
        error.insert(
            "requestId".to_string(),
            Value::String(current_request_id()),
        );
        if let Some(details) = &self.details {
            error.insert("details".to_string(), details.clone());
        }
        json!({ "error": Value::Object(error) })
    }
}

impl From<MlrsError> for ApiError {
    fn from(error: MlrsError) -> Self {
        Self {
            status: StatusCode::from_u16(error.status)
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            code: error.code,
            message: error.message,
            details: error.details,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body())).into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
