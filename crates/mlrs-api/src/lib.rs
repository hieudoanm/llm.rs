//! The mlrs HTTP API: routers, middleware, and application factory.

pub mod bundle;
pub mod error;
pub mod middleware;
pub mod query;
pub mod routes;
pub mod schemas;
pub mod state;
pub mod validation;
pub mod worker;

pub use error::{ApiError, ApiResult};
pub use state::AppState;

use axum::extract::State;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::{delete, get, patch, post};
use axum::Router;

/// Build the mlrs router: health probes, `/`, and the versioned API.
pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/", get(routes::root))
        .route("/healthz", get(routes::health::healthz))
        .route("/readyz", get(routes::health::readyz))
        .route("/health/worker", get(routes::health::worker_status))
        .route("/api/v1/algorithms", get(routes::algorithms::list_algorithms))
        .route("/api/v1/datasets", post(routes::datasets::upload_dataset))
        .route(
            "/api/v1/datasets/{dataset_id}",
            get(routes::datasets::get_dataset),
        )
        .route(
            "/api/v1/datasets/{dataset_id}/insights",
            get(routes::datasets::get_dataset_insights),
        )
        .route(
            "/api/v1/jobs",
            get(routes::jobs::list_jobs).post(routes::jobs::create_job),
        )
        .route("/api/v1/jobs/compare", get(routes::jobs::compare))
        .route(
            "/api/v1/jobs/artefacts.zip",
            get(routes::jobs::download_jobs_bundle),
        )
        .route("/api/v1/jobs/{job_id}", get(routes::jobs::get_job))
        .route(
            "/api/v1/jobs/{job_id}",
            patch(routes::jobs::update_job_metadata),
        )
        .route(
            "/api/v1/jobs/{job_id}",
            delete(routes::jobs::delete_job),
        )
        .route(
            "/api/v1/jobs/{job_id}/artefacts.zip",
            get(routes::jobs::download_job_bundle),
        )
        .route(
            "/api/v1/jobs/{job_id}/artefacts/{artefact_id}",
            get(routes::jobs::download_artefact),
        );

    let router = api.with_state(state.clone());

    let router = router.layer(axum::extract::DefaultBodyLimit::max(
        state.settings.max_upload_bytes() + 1024 * 1024,
    ));

    // Layer order (last added runs first): rate limit -> timeout -> compression
    // -> cors -> request id -> handler.
    let router = router
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            timeout_middleware,
        ))
        .layer(crate::middleware::compression(&state.settings))
        .layer(crate::middleware::cors(&state.settings))
        .layer(axum::middleware::from_fn(request_id_middleware));

    router
}

async fn rate_limit_middleware(
    state: State<AppState>,
    connect_info: axum::extract::ConnectInfo<std::net::SocketAddr>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    middleware::rate_limit(state, connect_info, request, next).await
}

async fn timeout_middleware(
    state: State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    middleware::timeout(state, request, next).await
}

async fn request_id_middleware(request: Request<axum::body::Body>, next: Next) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let header_value = axum::http::HeaderValue::from_str(&request_id)
        .unwrap_or_else(|_| axum::http::HeaderValue::from_static("invalid-request-id"));

    let mut response = crate::error::REQUEST_ID
        .scope(request_id, async move { next.run(request).await })
        .await;
    response
        .headers_mut()
        .insert("x-request-id", header_value);
    response
}
