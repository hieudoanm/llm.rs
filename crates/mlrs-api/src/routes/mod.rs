//! Health probes, algorithm catalogue, datasets, and job endpoints.

pub mod algorithms;
pub mod datasets;
pub mod health;
pub mod jobs;

/// Identify the service and its version.
pub async fn root() -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::OK,
        axum::Json(serde_json::json!({
            "service": "mlrs-backend",
            "version": "0.1.0",
        })),
    )
}
