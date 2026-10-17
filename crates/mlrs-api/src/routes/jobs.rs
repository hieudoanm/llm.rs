//! Job endpoints: create, poll, metadata, delete, list, compare, and bundles.

use axum::body::Body;
use axum::extract::{Path, RawQuery, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};

use crate::bundle;
use crate::error::{ApiError, ApiResult};
use crate::schemas;
use crate::state::AppState;
use crate::validation;
use mlrs_db::artefacts;
use mlrs_db::datasets;
use mlrs_db::jobs;

/// Look up a single query parameter in the raw query string.
fn query_value(raw: Option<&str>, name: &str) -> Option<String> {
    raw?.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        if key == name {
            Some(value.to_string())
        } else {
            None
        }
    })
}

pub const LIST_QUERY_PARAMS: [&str; 10] = [
    "limit",
    "cursor",
    "status",
    "taskType",
    "algorithmId",
    "tag",
    "q",
    "createdFrom",
    "createdTo",
    "sort",
];

#[derive(Debug, serde::Deserialize, Default)]
pub struct ListJobsQuery {
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub cursor: Option<String>,
}

pub async fn root() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "service": "mlrs-backend",
            "version": "0.1.0",
        })),
    )
}

// ──────────────────────────────────────────────────────────────────────────────
// POST /api/v1/jobs
// ─────────────────────────────────────────────────────────────────────────────────

pub async fn create_job(
    state: State<AppState>,
    headers: axum::http::HeaderMap,
    body: Result<Json<schemas::CreateJobRequest>, axum::extract::rejection::JsonRejection>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let Json(body) = body
        .map_err(|err| ApiError::new(400, "VALIDATION_ERROR", err.to_string()))?;

    let idempotency_key = headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    if let Some(key) = &idempotency_key {
        if key.chars().count() > 255
            || !key
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        {
            return Err(ApiError::new(
                400,
                "VALIDATION_ERROR",
                "Idempotency-Key must be 1-255 characters of [a-zA-Z0-9_-]",
            ));
        }
    }

    let canonical = body.canonical();
    let body_hash = mlrs_db::idempotency::request_hash(&canonical);

    if let Some(key) = &idempotency_key {
        let cached = mlrs_db::idempotency::lookup(state.db.pool(), key, &body_hash)
            .await
            .map_err(ApiError::from)?;
        if let Some(cached) = cached {
            return Ok((StatusCode::ACCEPTED, Json(cached)));
        }
    }

    let validated = validation::validate_job_request(&state.db, &body).await?;
    let job_id = uuid::Uuid::new_v4();
    let now = mlrs_core::domain::utc_now();
    let job = mlrs_core::domain::Job {
        id: job_id,
        dataset_id: validated.dataset.id,
        task_type: body.task_type.as_str().to_string(),
        algorithm_id: body.algorithm_id.clone(),
        feature_columns: validated.feature_columns.clone(),
        label_column: validated.label_column.clone(),
        rl_columns: validated.rl_columns.clone(),
        hyperparameters: validated.hyperparameters.clone(),
        split_ratio: validated.split_ratio,
        random_seed: validated.random_seed,
        export_pkl: validated.export_pkl,
        export_joblib: validated.export_joblib,
        status: "queued".to_string(),
        error_code: None,
        error_message: None,
        library_versions: None,
        name: None,
        tags: Vec::new(),
        attempts: 0,
        max_attempts: state.settings.job_max_attempts,
        next_attempt_at: None,
        created_at: now,
        updated_at: now,
    };

    mlrs_db::jobs::insert_job(state.db.pool(), &job)
        .await
        .map_err(ApiError::from)?;

    let response = json!({
        "jobId": job.id,
        "status": job.status,
        "datasetId": job.dataset_id,
        "taskType": job.task_type,
        "algorithmId": job.algorithm_id,
        "createdAt": schemas::timestamp(job.created_at),
        "name": job.name,
        "tags": job.tags,
        "attempts": job.attempts,
        "maxAttempts": job.max_attempts,
        "nextAttemptAt": job.next_attempt_at.map(schemas::timestamp),
        "queuePosition": Value::Null,
    });

    if let Some(key) = &idempotency_key {
        mlrs_db::idempotency::record(state.db.pool(), key, &body_hash, job_id, &response)
            .await
            .map_err(ApiError::from)?;
    }

    let positions = jobs::queue_positions(state.db.pool(), &[job_id])
        .await
        .map_err(ApiError::from)?;
    let final_response = json!({
        "jobId": job.id,
        "status": job.status,
        "datasetId": job.dataset_id,
        "taskType": job.task_type,
        "algorithmId": job.algorithm_id,
        "createdAt": schemas::timestamp(job.created_at),
        "name": job.name,
        "tags": job.tags,
        "attempts": job.attempts,
        "maxAttempts": job.max_attempts,
        "nextAttemptAt": job.next_attempt_at.map(schemas::timestamp),
        "queuePosition": positions.get(&job.id).copied(),
    });

    state.worker.enqueue(job_id);
    Ok((StatusCode::ACCEPTED, Json(final_response)))
}

// ──────────────────────────────────────────────────────────────────────────────
// GET /api/v1/jobs/{job_id}
// ─────────────────────────────────────────────────────────────────────────────────

pub async fn get_job(
    state: State<AppState>,
    Path(job_id): Path<uuid::Uuid>,
) -> ApiResult<Json<Value>> {
    let job = jobs::get_job(state.db.pool(), job_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::new(404, "JOB_NOT_FOUND", "Job not found"))?;

    let artefacts = artefacts::list_artefacts(state.db.pool(), job_id)
        .await
        .map_err(ApiError::from)?;
    let metric = jobs::get_metric(state.db.pool(), job_id)
        .await
        .map_err(ApiError::from)?;
    let queue_position = if job.status == "queued" {
        jobs::queue_positions(state.db.pool(), &[job.id])
            .await
            .map_err(ApiError::from)?
            .get(&job.id)
            .copied()
    } else {
        None
    };

    Ok(Json(schemas::job_detail_value(
        &job,
        &artefacts,
        metric.as_ref(),
        queue_position,
    )))
}

// ──────────────────────────────────────────────────────────────────────────────
// PATCH /api/v1/jobs/{job_id}
// ─────────────────────────────────────────────────────────────────────────────────

pub async fn update_job_metadata(
    state: State<AppState>,
    Path(job_id): Path<uuid::Uuid>,
    body: Result<Json<schemas::UpdateJobMetadataRequest>, axum::extract::rejection::JsonRejection>,
) -> ApiResult<Json<Value>> {
    let Json(body) = body
        .map_err(|err| ApiError::new(400, "VALIDATION_ERROR", err.to_string()))?;

    let mut job = jobs::get_job(state.db.pool(), job_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::new(404, "JOB_NOT_FOUND", "Job not found"))?;

    let name = match body.name {
        Some(value) => validation::normalise_name(value)?,
        None => job.name.clone(),
    };
    let tags = match body.tags {
        Some(Some(tags)) => validation::normalise_tags(tags)?,
        Some(None) => Vec::new(),
        None => job.tags.clone(),
    };

    job.name = name.clone();
    job.tags = tags.clone();
    job.updated_at = mlrs_core::domain::utc_now();

    mlrs_db::jobs::update_metadata(state.db.pool(), job_id, name.as_deref(), &tags)
        .await
        .map_err(ApiError::from)?;

    let artefacts = artefacts::list_artefacts(state.db.pool(), job_id)
        .await
        .map_err(ApiError::from)?;
    let metric = jobs::get_metric(state.db.pool(), job_id)
        .await
        .map_err(ApiError::from)?;
    let queue_position = if job.status == "queued" {
        jobs::queue_positions(state.db.pool(), &[job.id])
            .await
            .map_err(ApiError::from)?
            .get(&job.id)
            .copied()
    } else {
        None
    };

    Ok(Json(schemas::job_detail_value(
        &job,
        &artefacts,
        metric.as_ref(),
        queue_position,
    )))
}

// ──────────────────────────────────────────────────────────────────────────────
// DELETE /api/v1/jobs/{job_id}
// ─────────────────────────────────────────────────────────────────────────────────

pub async fn delete_job(
    state: State<AppState>,
    Path(job_id): Path<uuid::Uuid>,
) -> ApiResult<Response> {
    let job = jobs::get_job(state.db.pool(), job_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::new(404, "JOB_NOT_FOUND", "Job not found"))?;

    mlrs_core::storage::remove_job_files(job_id);
    state.worker.cancel(job_id);

    mlrs_db::jobs::delete_metrics(state.db.pool(), job_id)
        .await
        .map_err(ApiError::from)?;
    mlrs_db::artefacts::delete_artefacts(state.db.pool(), job_id)
        .await
        .map_err(ApiError::from)?;
    mlrs_db::jobs::delete_job(state.db.pool(), job_id)
        .await
        .map_err(ApiError::from)?;

    let _ = job;
    Ok((StatusCode::NO_CONTENT, Body::empty()).into_response())
}

// ──────────────────────────────────────────────────────────────────────────────
// GET /api/v1/jobs/{job_id}/artefacts.zip
// ─────────────────────────────────────────────────────────────────────────────────

pub async fn download_job_bundle(
    state: State<AppState>,
    Path(job_id): Path<uuid::Uuid>,
) -> ApiResult<Response> {
    let job = jobs::get_job(state.db.pool(), job_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::new(404, "JOB_NOT_FOUND", "Job not found"))?;

    let artefacts = artefacts::list_artefacts(state.db.pool(), job_id)
        .await
        .map_err(ApiError::from)?;
    if artefacts.is_empty() {
        return Err(
            ApiError::new(404, "ARTEFACT_NOT_FOUND", "Job has no artefacts")
                .with_details(json!({ "jobId": job_id.to_string() })),
        );
    }

    let metric = jobs::get_metric(state.db.pool(), job_id)
        .await
        .map_err(ApiError::from)?;
    let bundle = bundle::build_bundle(
        &[bundle::BundleJob {
            job,
            artefacts,
            metrics: metric.map(|metric| metric.metrics.clone()),
        }],
        Vec::new(),
    )
    .map_err(ApiError::from)?;

    Ok(zip_response(bundle.bytes, &format!("{}.zip", job_id)))
}

// ──────────────────────────────────────────────────────────────────────────────
// GET /api/v1/jobs/{job_id}/artefacts/{artefact_id}
// ─────────────────────────────────────────────────────────────────────────────────

pub async fn download_artefact(
    state: State<AppState>,
    Path(job_id): Path<uuid::Uuid>,
    Path(artefact_id): Path<uuid::Uuid>,
) -> ApiResult<Response> {
    let artefact = artefacts::get_artefact(state.db.pool(), job_id, artefact_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::new(404, "ARTEFACT_NOT_FOUND", "Artefact not found"))?;

    let job = jobs::get_job(state.db.pool(), job_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::new(404, "JOB_NOT_FOUND", "Job not found"))?;

    if job.status != "completed" {
        return Err(ApiError::new(404, "ARTEFACT_NOT_FOUND", "Artefact not available"));
    }

    let path = mlrs_core::storage::artefact_path(&artefact.storage_key, job_id);
    if !path.exists() {
        return Err(ApiError::new(404, "ARTEFACT_NOT_FOUND", "Artefact file missing"));
    }

    let bytes = std::fs::read(&path)
        .map_err(|error| ApiError::new(404, "ARTEFACT_NOT_FOUND", format!("Artefact file missing: {error}")))?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        )
        .header(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!("attachment; filename=\"{}\"", artefact.filename))
                .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
        )
        .body(Body::from(bytes))
        .map_err(|error| ApiError::new(500, "INTERNAL_ERROR", error.to_string()))?;
    Ok(response)
}

// ──────────────────────────────────────────────────────────────────────────────
// GET /api/v1/jobs
// ─────────────────────────────────────────────────────────────────────────────────

pub async fn list_jobs(
    state: State<AppState>,
    query: RawQuery,
) -> ApiResult<Json<Value>> {
    let pairs = crate::query::QueryPairs::parse(query.0.as_deref());
    if let Some(repeated) = pairs.repeated(&LIST_QUERY_PARAMS) {
        return Err(ApiError::new(
            400,
            "VALIDATION_ERROR",
            format!("Repeated query parameter: {repeated}; use the comma-separated form instead"),
        )
        .with_details(serde_json::json!({ "field": repeated })));
    }

    let limit = pairs
        .get("limit")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(20);
    if limit < 1 || limit > 100 {
        return Err(
            ApiError::validation("limit must be between 1 and 100").with_details(
                serde_json::json!({ "field": "limit" }),
            ),
        );
    }

    let filters = jobs::parse_filters(
        pairs.get("status"),
        pairs.get("taskType"),
        pairs.get("algorithmId"),
        pairs.get("tag"),
        pairs.get("q"),
        pairs.get("createdFrom"),
        pairs.get("createdTo"),
        pairs.get("sort"),
    )
    .map_err(ApiError::from)?;

    let cursor = match pairs.get("cursor") {
        Some(value) => Some(jobs::decode_cursor(value, &filters).map_err(ApiError::from)?),
        None => None,
    };

    let result = jobs::list_jobs(state.db.pool(), &filters, cursor.as_ref(), limit + 1)
        .await
        .map_err(ApiError::from)?;
    let has_more = result.len() > limit as usize;
    let page = result.into_iter().take(limit as usize).collect::<Vec<_>>();
    let positions = jobs::queue_positions(
        state.db.pool(),
        &page.iter().map(|job| job.id).collect::<Vec<_>>(),
    )
    .await
    .map_err(ApiError::from)?;

    let next_cursor = if has_more {
        page.last().map(|last| {
            let value = jobs::sort_value(&filters, last);
            jobs::encode_cursor(&filters, &value, last.id)
        })
    } else {
        None
    };

    let accepted = page
        .iter()
        .map(|job| schemas::job_accepted_value(job, positions.get(&job.id).copied()))
        .collect::<Vec<_>>();

    Ok(Json(json!({
        "jobs": accepted,
        "nextCursor": next_cursor,
        "hasMore": has_more,
    })))
}

// ──────────────────────────────────────────────────────────────────────────────
// GET /api/v1/jobs/compare
// ─────────────────────────────────────────────────────────────────────────────────

pub async fn compare(
    state: State<AppState>,
    query: RawQuery,
) -> ApiResult<Json<Value>> {
    let ids = validation::parse_job_ids(
        query_value(query.0.as_deref(), "ids").as_deref(),
        2,
        5,
    )
    .map_err(|error| error)?;

    let jobs = jobs::jobs_by_ids(state.db.pool(), &ids)
        .await
        .map_err(ApiError::from)?;
    let job_map = jobs
        .iter()
        .map(|job| (job.id, job.clone()))
        .collect::<std::collections::HashMap<_, _>>();

    let mut missing = Vec::new();
    for id in &ids {
        if !job_map.contains_key(id) {
            missing.push(id.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(
            ApiError::new(404, "JOB_NOT_FOUND", "One or more jobs were not found")
                .with_details(json!({ "missingJobIds": missing })),
        );
    }

    let metrics = jobs::metrics_by_ids(state.db.pool(), &ids)
        .await
        .map_err(ApiError::from)?;
    let artefact_list = artefacts::list_artefacts_for_jobs(state.db.pool(), &ids)
        .await
        .map_err(ApiError::from)?;

    let mut artefacts: std::collections::HashMap<uuid::Uuid, Vec<mlrs_core::domain::Artefact>> =
        std::collections::HashMap::new();
    for artefact in artefact_list {
        artefacts
            .entry(artefact.job_id)
            .or_default()
            .push(artefact);
    }

    let mut entries: Vec<Value> = Vec::new();
    for id in &ids {
        let job = job_map.get(id).expect("job ids are checked above");
        let dataset = datasets::get_dataset(state.db.pool(), job.dataset_id)
            .await
            .map_err(ApiError::from)?
            .expect("job dataset is present");
        let metric = metrics.get(id);
        entries.push(json!({
            "jobId": job.id,
            "name": job.name,
            "tags": job.tags,
            "status": job.status,
            "taskType": job.task_type,
            "algorithmId": job.algorithm_id,
            "createdAt": schemas::timestamp(job.created_at),
            "dataset": {
                "datasetId": job.dataset_id,
                "filename": dataset.original_filename,
                "rowCount": dataset.row_count,
                "columnCount": dataset.columns.len(),
            },
            "featureColumns": job.feature_columns,
            "labelColumn": job.label_column,
            "splitRatio": job.split_ratio,
            "randomSeed": job.random_seed,
            "hyperparameters": job.hyperparameters,
            "metrics": metric.map(|metric| metric.metrics.clone()),
            "artefactFormats": artefacts
                .get(id)
                .map(|list| {
                    list.iter()
                        .map(|artefact| artefact.format.clone())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            "error": job.error_code.as_ref().map(|code| json!({
                "code": code,
                "message": job.error_message.clone().unwrap_or_default(),
            })),
        }));
    }

    Ok(Json(json!({
        "jobs": entries,
        "warnings": Vec::<Value>::new(),
    })))
}

// ──────────────────────────────────────────────────────────────────────────────
// GET /api/v1/jobs/artefacts.zip
// ─────────────────────────────────────────────────────────────────────────────────

pub async fn download_jobs_bundle(
    state: State<AppState>,
    query: RawQuery,
) -> ApiResult<Response> {
    let ids = validation::parse_job_ids(
        query_value(query.0.as_deref(), "ids").as_deref(),
        1,
        10,
    )
    .map_err(|error| error)?;

    let jobs = jobs::jobs_by_ids(state.db.pool(), &ids)
        .await
        .map_err(ApiError::from)?;
    let mut job_map = std::collections::HashMap::new();
    for job in jobs {
        job_map.insert(job.id, job);
    }
    let missing: Vec<String> = ids
        .iter()
        .filter(|id| !job_map.contains_key(*id))
        .map(|id| id.to_string())
        .collect();
    if !missing.is_empty() {
        return Err(
            ApiError::new(404, "JOB_NOT_FOUND", "One or more jobs were not found")
                .with_details(json!({ "missingJobIds": missing })),
        );
    }

    let mut bundle_jobs = Vec::new();
    let mut skipped = Vec::new();
    for id in &ids {
        let job = job_map.get(id).expect("job ids were checked above");
        let artefacts = artefacts::list_artefacts(state.db.pool(), *id)
            .await
            .map_err(ApiError::from)?;
        if artefacts.is_empty() {
            skipped.push(json!({
                "jobId": id.to_string(),
                "reason": if job.status == "completed" { "NO_ARTEFACTS" } else { "NOT_COMPLETED" },
            }));
            continue;
        }
        let metric = jobs::get_metric(state.db.pool(), *id)
            .await
            .map_err(ApiError::from)?;
        bundle_jobs.push(bundle::BundleJob {
            job: job.clone(),
            artefacts,
            metrics: metric.map(|metric| metric.metrics.clone()),
        });
    }

    if bundle_jobs.is_empty() {
        return Err(
            ApiError::new(404, "ARTEFACT_NOT_FOUND", "None of the selected jobs has artefacts")
                .with_details(json!({ "jobIds": ids.iter().map(|id| id.to_string()).collect::<Vec<_>>() })),
        );
    }

    let bundle = bundle::build_bundle(&bundle_jobs, skipped).map_err(ApiError::from)?;
    Ok(zip_response(bundle.bytes, "mlrs-jobs.zip"))
}

fn zip_response(bytes: Vec<u8>, filename: &str) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, HeaderValue::from_static("application/zip"))
        .header(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!("attachment; filename=\"{filename}\""))
                .expect("attachment disposition is ascii-safe"),
        )
        .body(Body::from(bytes))
        .expect("a built response cannot fail")
}
