//! Liveness and readiness probes.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(serde::Serialize)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(serde::Serialize)]
pub struct ReadyResponse {
    pub status: String,
}

#[derive(serde::Serialize, Default)]
pub struct WorkerStatusResponse {
    #[serde(rename = "maxConcurrentJobs")]
    pub max_concurrent_jobs: i64,
    #[serde(rename = "runningJobs")]
    pub running_jobs: i64,
    #[serde(rename = "queuedJobs")]
    pub queued_jobs: i64,
    #[serde(rename = "retryingJobs")]
    pub retrying_jobs: i64,
}

pub async fn healthz() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "ok".to_string(),
        }),
    )
}

pub async fn readyz(state: State<AppState>) -> Response {
    if state.db.ping().await.is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiError::new(
                503,
                "SERVICE_UNAVAILABLE",
                "Database not reachable",
            )
            .body()),
        )
            .into_response();
    }
    (
        StatusCode::OK,
        Json(ReadyResponse {
            status: "ok".to_string(),
        }),
    )
    .into_response()
}

pub async fn worker_status(state: State<AppState>) -> impl IntoResponse {
    let worker = match mlrs_db::jobs::worker_counts(state.db.pool()).await {
        Ok(worker) => worker,
        Err(_) => mlrs_db::jobs::WorkerCounts {
            running: 0,
            queued: 0,
            retrying: 0,
        },
    };
    (
        StatusCode::OK,
        Json(WorkerStatusResponse {
            max_concurrent_jobs: state.settings.max_concurrent_jobs,
            running_jobs: worker.running,
            queued_jobs: worker.queued,
            retrying_jobs: worker.retrying,
        }),
    )
}
