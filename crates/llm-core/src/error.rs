//! The llm error envelope shared by every layer.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A user-facing error carrying the llm error-envelope fields.
///
/// The HTTP layer converts this into the flat `{"error": {...}}` body and
/// keeps internal driver / SQL text out of the response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmError {
    pub status: u16,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

pub type LlmResult<T> = Result<T, LlmError>;

impl LlmError {
    pub fn new(status: u16, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status,
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

    pub fn conflict(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(409, code, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(500, "INTERNAL_ERROR", message)
    }

    pub fn database() -> Self {
        Self::new(
            500,
            "DATABASE_ERROR",
            "A database error occurred. Please retry.",
        )
    }

    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::new(503, "SERVICE_UNAVAILABLE", message)
    }
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for LlmError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_new_and_display() {
        let err = LlmError::new(404, "NOT_FOUND", "missing");
        assert_eq!(err.status, 404);
        assert_eq!(err.code, "NOT_FOUND");
        assert_eq!(err.message, "missing");
        let s = format!("{}", err);
        assert!(s.contains("NOT_FOUND"));
    }

    #[test]
    fn error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "nope");
        let err: LlmError = io_err.into();
        assert_eq!(err.status, 500);
        assert_eq!(err.code, "INTERNAL_ERROR");
    }

    #[test]
    fn error_from_serde_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("{bad").unwrap_err();
        let err: LlmError = json_err.into();
        assert_eq!(err.status, 500);
        assert_eq!(err.code, "INTERNAL_ERROR");
    }

    #[test]
    fn error_source() {
        let err = LlmError::new(400, "X", "y");
        let _ = format!("{:?}", err);
    }
}
impl From<serde_json::Error> for LlmError {
    fn from(value: serde_json::Error) -> Self {
        Self::internal(value.to_string())
    }
}

impl From<std::io::Error> for LlmError {
    fn from(value: std::io::Error) -> Self {
        Self::internal(value.to_string())
    }
}

/// Build the flat llm error envelope body.
pub fn error_body(error: &LlmError, request_id: &str) -> Value {
    let mut detail = serde_json::json!({
        "code": error.code,
        "message": error.message,
        "requestId": request_id,
    });
    if let Some(details) = &error.details {
        detail["details"] = details.clone();
    }
    serde_json::json!({ "error": detail })
}
