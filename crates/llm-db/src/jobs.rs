//! Job persistence: discovery filters, keyset pagination, the worker lifecycle
//! writes, and metrics.

use std::collections::HashMap;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use uuid::Uuid;

use llm_core::domain::{utc_now, Job, JobMetric, JobStatus, TaskType};
use llm_core::error::{LlmError, LlmResult};

use crate::convert::{database_error, timestamp, JobRow, MetricRow};

pub const MAX_QUERY_LENGTH: usize = 200;
pub const DEFAULT_MAX_ATTEMPTS: i64 = 2;

pub const ACTIVE_STATUSES: [&str; 4] =
    ["validating", "training", "evaluating", "exporting"];

/// Sortable fields and the direction used when the caller omits `:asc`/`:desc`.
pub const SORT_FIELDS: [(&str, &str); 4] = [
    ("createdAt", "desc"),
    ("updatedAt", "desc"),
    ("status", "asc"),
    ("algorithmId", "asc"),
];

#[derive(Debug, Clone)]
pub struct JobFilters {
    pub statuses: Vec<String>,
    pub task_types: Vec<String>,
    pub algorithm_ids: Vec<String>,
    pub tags: Vec<String>,
    pub q: Option<String>,
    pub created_from: Option<DateTime<Utc>>,
    pub created_to: Option<DateTime<Utc>>,
    pub sort_field: String,
    pub sort_direction: String,
}

impl Default for JobFilters {
    fn default() -> Self {
        Self {
            statuses: Vec::new(),
            task_types: Vec::new(),
            algorithm_ids: Vec::new(),
            tags: Vec::new(),
            q: None,
            created_from: None,
            created_to: None,
            sort_field: "createdAt".to_string(),
            sort_direction: "desc".to_string(),
        }
    }
}

impl JobFilters {
    pub fn ascending(&self) -> bool {
        self.sort_direction == "asc"
    }

    pub fn sort_key(&self) -> String {
        format!("{}:{}", self.sort_field, self.sort_direction)
    }
}

/// An opaque pagination position: the sort value of the last row plus its id.
#[derive(Debug, Clone)]
pub struct JobCursor {
    pub value: String,
    pub id: Uuid,
}

#[derive(Debug, Clone)]
pub struct WorkerCounts {
    pub running: i64,
    pub queued: i64,
    pub retrying: i64,
}

fn invalid(field: &str, message: impl Into<String>) -> LlmError {
    LlmError::new(400, "VALIDATION_ERROR", message).with_details(json!({ "field": field }))
}

fn split(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}

fn parse_query_timestamp(field: &str, value: Option<&str>) -> LlmResult<Option<DateTime<Utc>>> {
    let Some(raw) = value.map(str::trim).filter(|raw| !raw.is_empty()) else {
        return Ok(None);
    };
    if let Ok(parsed) = DateTime::parse_from_rfc3339(&raw.replace('Z', "+00:00")) {
        return Ok(Some(parsed.with_timezone(&Utc)));
    }
    for format in ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%d %H:%M:%S%.f"] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(raw, format) {
            return Ok(Some(DateTime::from_naive_utc_and_offset(naive, Utc)));
        }
    }
    Err(invalid(
        field,
        format!("{field} must be an ISO-8601 timestamp"),
    ))
}

fn parse_sort(value: Option<&str>) -> LlmResult<(String, String)> {
    let Some(raw) = value.map(str::trim).filter(|raw| !raw.is_empty()) else {
        return Ok(("createdAt".to_string(), "desc".to_string()));
    };
    let (field, direction) = match raw.split_once(':') {
        Some((field, direction)) => (field.trim(), Some(direction.trim().to_ascii_lowercase())),
        None => (raw, None),
    };
    let Some((_, default_direction)) = SORT_FIELDS.iter().find(|(name, _)| *name == field) else {
        return Err(invalid(
            "sort",
            format!("Unknown sort field: {field}"),
        )
        .with_details(json!({
            "field": "sort",
            "allowed": SORT_FIELDS.iter().map(|(name, _)| *name).collect::<Vec<_>>(),
        })));
    };
    let direction = match direction {
        None => default_direction.to_string(),
        Some(direction) if direction == "asc" || direction == "desc" => direction,
        Some(_) => {
            return Err(invalid(
                "sort",
                "sort direction must be \"asc\" or \"desc\"",
            ))
        }
    };
    Ok((field.to_string(), direction))
}

/// Validate the query parameters of `GET /api/v1/jobs`.
#[allow(clippy::too_many_arguments)]
pub fn parse_filters(
    statuses: Option<&str>,
    task_types: Option<&str>,
    algorithm_ids: Option<&str>,
    tags: Option<&str>,
    q: Option<&str>,
    created_from: Option<&str>,
    created_to: Option<&str>,
    sort: Option<&str>,
) -> LlmResult<JobFilters> {
    let status_values = split(statuses);
    for status in &status_values {
        if !JobStatus::ALL.iter().any(|known| known.as_str() == status) {
            return Err(invalid("status", format!("Unknown job status: {status}")));
        }
    }

    let task_values = split(task_types);
    for task_type in &task_values {
        if !TaskType::ALL.iter().any(|known| known.as_str() == task_type) {
            return Err(invalid("taskType", format!("Unknown task type: {task_type}")));
        }
    }

    let algorithm_values = split(algorithm_ids);
    let tag_values: Vec<String> = split(tags)
        .into_iter()
        .map(|tag| tag.to_lowercase())
        .collect();

    let search = q.map(str::trim).filter(|value| !value.is_empty());
    if let Some(search) = search {
        if search.chars().count() > MAX_QUERY_LENGTH {
            return Err(invalid(
                "q",
                format!("q must be at most {MAX_QUERY_LENGTH} characters"),
            ));
        }
    }

    let from_dt = parse_query_timestamp("createdFrom", created_from)?;
    let to_dt = parse_query_timestamp("createdTo", created_to)?;
    if let (Some(from), Some(to)) = (from_dt, to_dt) {
        if to < from {
            return Err(invalid(
                "createdTo",
                "createdTo must not be earlier than createdFrom",
            ));
        }
    }

    let (sort_field, sort_direction) = parse_sort(sort)?;
    Ok(JobFilters {
        statuses: status_values,
        task_types: task_values,
        algorithm_ids: algorithm_values,
        tags: tag_values,
        q: search.map(str::to_string),
        created_from: from_dt,
        created_to: to_dt,
        sort_field,
        sort_direction,
    })
}

/// A stable fingerprint of the active filter and sort set.
pub fn fingerprint(filters: &JobFilters) -> String {
    let payload = json!({
        "algorithmId": filters.algorithm_ids,
        "createdFrom": filters.created_from.map(timestamp).unwrap_or_default(),
        "createdTo": filters.created_to.map(timestamp).unwrap_or_default(),
        "q": filters.q.clone().unwrap_or_default(),
        "sort": filters.sort_key(),
        "status": filters.statuses,
        "tag": filters.tags,
        "taskType": filters.task_types,
    });
    let serialized = serde_json::to_string(&payload).unwrap_or_default();
    let digest = Sha256::digest(serialized.as_bytes());
    digest.iter().take(8).map(|byte| format!("{byte:02x}")).collect()
}

/// The JSON-serialisable sort key of `job`.
pub fn sort_value(filters: &JobFilters, job: &Job) -> String {
    match filters.sort_field.as_str() {
        "createdAt" => timestamp(job.created_at),
        "updatedAt" => timestamp(job.updated_at),
        "status" => job.status.clone(),
        _ => job.algorithm_id.clone(),
    }
}

pub fn encode_cursor(filters: &JobFilters, sort_value: &str, job_id: Uuid) -> String {
    let payload = json!({
        "v": 1,
        "fp": fingerprint(filters),
        "k": sort_value,
        "id": job_id.to_string(),
    });
    let raw = serde_json::to_vec(&payload).unwrap_or_default();
    URL_SAFE_NO_PAD.encode(raw)
}

/// Decode a cursor, rejecting one issued for a different filter/sort set.
pub fn decode_cursor(cursor: &str, filters: &JobFilters) -> LlmResult<JobCursor> {
    let trimmed = cursor.trim_end_matches('=');
    let raw = URL_SAFE_NO_PAD
        .decode(trimmed)
        .map_err(|_| invalid("cursor", "Invalid cursor"))?;
    let payload: Value =
        serde_json::from_slice(&raw).map_err(|_| invalid("cursor", "Invalid cursor"))?;
    if payload.get("v").and_then(Value::as_i64) != Some(1) {
        return Err(invalid("cursor", "Invalid cursor"));
    }
    let matches = payload.get("fp").and_then(Value::as_str) == Some(&fingerprint(filters));
    if !matches {
        return Err(invalid(
            "cursor",
            "Cursor does not match the active filters; restart pagination",
        ));
    }
    let id = payload
        .get("id")
        .and_then(Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or_else(|| invalid("cursor", "Invalid cursor"))?;
    let value = payload
        .get("k")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid("cursor", "Invalid cursor"))?;

    // Validate the value against the sort field so a malformed cursor fails
    // here rather than during the SQL bind.
    comparable_value(filters, value)?;
    Ok(JobCursor {
        value: value.to_string(),
        id,
    })
}

enum Comparable {
    Text(String),
    Ordinal(i64),
}

fn parse_cursor_timestamp(value: &str) -> LlmResult<DateTime<Utc>> {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(&value.replace('Z', "+00:00")) {
        return Ok(parsed.with_timezone(&Utc));
    }
    for format in ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S%.f%:z"] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(value, format) {
            return Ok(DateTime::from_naive_utc_and_offset(naive, Utc));
        }
    }
    Err(invalid("cursor", "Invalid cursor"))
}

fn comparable_value(filters: &JobFilters, value: &str) -> LlmResult<Comparable> {
    match filters.sort_field.as_str() {
        "createdAt" | "updatedAt" => {
            parse_cursor_timestamp(value).map(|parsed| Comparable::Text(timestamp(parsed)))
        }
        "status" => {
            let status: JobStatus = value
                .parse()
                .map_err(|_| invalid("cursor", "Invalid cursor"))?;
            Ok(Comparable::Ordinal(status.ordinal()))
        }
        _ => Ok(Comparable::Text(value.to_string())),
    }
}

fn sort_expression(filters: &JobFilters) -> String {
    match filters.sort_field.as_str() {
        "createdAt" => "created_at".to_string(),
        "updatedAt" => "updated_at".to_string(),
        "status" => "(CASE status \
            WHEN 'queued' THEN 0 \
            WHEN 'validating' THEN 1 \
            WHEN 'training' THEN 2 \
            WHEN 'evaluating' THEN 3 \
            WHEN 'exporting' THEN 4 \
            WHEN 'retrying' THEN 5 \
            WHEN 'completed' THEN 6 \
            WHEN 'failed' THEN 7 END)"
            .to_string(),
        _ => "algorithm_id".to_string(),
    }
}

fn push_bound(builder: &mut QueryBuilder<'_, Sqlite>, value: &Comparable) {
    match value {
        Comparable::Text(text) => {
            builder.push_bind(text.clone());
        }
        Comparable::Ordinal(ordinal) => {
            builder.push_bind(*ordinal);
        }
    }
}

fn push_in(builder: &mut QueryBuilder<'_, Sqlite>, column: &str, values: &[String]) {
    builder.push(" AND ").push(column).push(" IN (");
    let mut separated = builder.separated(", ");
    for value in values {
        separated.push_bind(value.clone());
    }
    separated.push_unseparated(")");
}

/// Build and execute the jobs-list query.
pub async fn list_jobs(
    pool: &SqlitePool,
    filters: &JobFilters,
    cursor: Option<&JobCursor>,
    limit: i64,
) -> LlmResult<Vec<Job>> {
    let mut builder = QueryBuilder::<Sqlite>::new("SELECT * FROM jobs WHERE 1 = 1");

    if !filters.statuses.is_empty() {
        push_in(&mut builder, "status", &filters.statuses);
    }
    if !filters.task_types.is_empty() {
        push_in(&mut builder, "task_type", &filters.task_types);
    }
    if !filters.algorithm_ids.is_empty() {
        push_in(&mut builder, "algorithm_id", &filters.algorithm_ids);
    }
    for tag in &filters.tags {
        builder.push(
            " AND EXISTS (SELECT 1 FROM json_each(jobs.tags) WHERE json_each.value = ",
        );
        builder.push_bind(tag.clone());
        builder.push(")");
    }
    if let Some(search) = &filters.q {
        let pattern = format!("%{}%", search.to_lowercase());
        builder.push(" AND (lower(COALESCE(name, '')) LIKE ");
        builder.push_bind(pattern.clone());
        builder.push(" OR lower(tags) LIKE ");
        builder.push_bind(pattern.clone());
        builder.push(" OR lower(algorithm_id) LIKE ");
        builder.push_bind(pattern);
        builder.push(")");
    }
    if let Some(from) = filters.created_from {
        builder.push(" AND created_at >= ");
        builder.push_bind(timestamp(from));
    }
    if let Some(to) = filters.created_to {
        builder.push(" AND created_at < ");
        builder.push_bind(timestamp(to));
    }

    if let Some(cursor) = cursor {
        let comparable = comparable_value(filters, &cursor.value)?;
        let key = sort_expression(filters);
        let operator = if filters.ascending() { " > " } else { " < " };
        builder.push(" AND ((").push(&key).push(")").push(operator);
        push_bound(&mut builder, &comparable);
        builder
            .push(" OR ((")
            .push(&key)
            .push(") = ");
        push_bound(&mut builder, &comparable);
        builder.push(" AND id < ");
        builder.push_bind(cursor.id.to_string());
        builder.push("))");
    }

    let key = sort_expression(filters);
    builder
        .push(" ORDER BY (")
        .push(&key)
        .push(") ")
        .push(if filters.ascending() { "ASC" } else { "DESC" })
        .push(", id DESC LIMIT ");
    builder.push_bind(limit);

    let rows = builder
        .build_query_as::<JobRow>()
        .fetch_all(pool)
        .await
        .map_err(|err| database_error("listing jobs", err))?;
    rows.into_iter().map(JobRow::into_domain).collect()
}

/// The number of queued jobs ahead of each queued job in `job_ids`.
pub async fn queue_positions(
    pool: &SqlitePool,
    job_ids: &[Uuid],
) -> LlmResult<HashMap<Uuid, i64>> {
    if job_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let mut builder = QueryBuilder::<Sqlite>::new(
        "SELECT id, ahead FROM (\
            SELECT id, ROW_NUMBER() OVER (ORDER BY created_at ASC, id ASC) - 1 AS ahead \
            FROM jobs WHERE status = 'queued'\
        ) WHERE id IN (",
    );
    let mut separated = builder.separated(", ");
    for job_id in job_ids {
        separated.push_bind(job_id.to_string());
    }
    separated.push_unseparated(")");

    let rows: Vec<(String, i64)> = builder
        .build_query_as::<(String, i64)>()
        .fetch_all(pool)
        .await
        .map_err(|err| database_error("ranking the job queue", err))?;

    let mut positions = HashMap::new();
    for (id, ahead) in rows {
        if let Ok(id) = Uuid::parse_str(&id) {
            positions.insert(id, ahead);
        }
    }
    Ok(positions)
}

/// Ids of the jobs still waiting in the queue, oldest first.
pub async fn queued_ids(pool: &SqlitePool) -> LlmResult<Vec<Uuid>> {
    let ids = sqlx::query_scalar::<_, String>(
        "SELECT id FROM jobs WHERE status = 'queued' ORDER BY created_at ASC, id ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|err| database_error("listing queued jobs", err))?;
    Ok(ids
        .into_iter()
        .filter_map(|id| Uuid::parse_str(&id).ok())
        .collect())
}

pub async fn worker_counts(pool: &SqlitePool) -> LlmResult<WorkerCounts> {
    let running = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM jobs WHERE status IN ('validating', 'training', 'evaluating', 'exporting')",
    )
    .fetch_one(pool)
    .await
    .map_err(|err| database_error("counting running jobs", err))?;
    let queued = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM jobs WHERE status = 'queued'")
        .fetch_one(pool)
        .await
        .map_err(|err| database_error("counting queued jobs", err))?;
    let retrying =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM jobs WHERE status = 'retrying'")
            .fetch_one(pool)
            .await
            .map_err(|err| database_error("counting retrying jobs", err))?;
    Ok(WorkerCounts {
        running,
        queued,
        retrying,
    })
}

fn encode<T: serde::Serialize>(value: &T, field: &str) -> LlmResult<String> {
    serde_json::to_string(value)
        .map_err(|err| LlmError::internal(format!("failed to encode {field}: {err}")))
}

pub async fn insert_job(pool: &SqlitePool, job: &Job) -> LlmResult<()> {
    sqlx::query(
        r#"
        INSERT INTO jobs (
            id, dataset_id, task_type, algorithm_id, feature_columns,
            label_column, rl_columns, hyperparameters, split_ratio, random_seed,
            export_pkl, export_joblib, status, error_code, error_message,
            library_versions, name, tags, attempts, max_attempts,
            next_attempt_at, created_at, updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
            ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20,
            ?21, ?22, ?23
        )
        "#,
    )
    .bind(job.id.to_string())
    .bind(job.dataset_id.to_string())
    .bind(&job.task_type)
    .bind(&job.algorithm_id)
    .bind(encode(&job.feature_columns, "feature columns")?)
    .bind(&job.label_column)
    .bind(match &job.rl_columns {
        Some(columns) => Some(encode(columns, "rl columns")?),
        None => None,
    })
    .bind(encode(&job.hyperparameters, "hyperparameters")?)
    .bind(job.split_ratio)
    .bind(job.random_seed)
    .bind(job.export_pkl as i64)
    .bind(job.export_joblib as i64)
    .bind(&job.status)
    .bind(&job.error_code)
    .bind(&job.error_message)
    .bind(match &job.library_versions {
        Some(versions) => Some(encode(versions, "library versions")?),
        None => None,
    })
    .bind(&job.name)
    .bind(encode(&job.tags, "tags")?)
    .bind(job.attempts)
    .bind(job.max_attempts)
    .bind(job.next_attempt_at.map(timestamp))
    .bind(timestamp(job.created_at))
    .bind(timestamp(job.updated_at))
    .execute(pool)
    .await
    .map_err(|err| database_error("inserting a job", err))?;
    Ok(())
}

pub async fn get_job(pool: &SqlitePool, job_id: Uuid) -> LlmResult<Option<Job>> {
    let row = sqlx::query_as::<_, JobRow>("SELECT * FROM jobs WHERE id = ?1")
        .bind(job_id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(|err| database_error("loading a job", err))?;
    row.map(JobRow::into_domain).transpose()
}

pub async fn jobs_by_ids(pool: &SqlitePool, job_ids: &[Uuid]) -> LlmResult<Vec<Job>> {
    if job_ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut builder = QueryBuilder::<Sqlite>::new("SELECT * FROM jobs WHERE id IN (");
    let mut separated = builder.separated(", ");
    for job_id in job_ids {
        separated.push_bind(job_id.to_string());
    }
    separated.push_unseparated(")");

    let rows = builder
        .build_query_as::<JobRow>()
        .fetch_all(pool)
        .await
        .map_err(|err| database_error("loading jobs", err))?;
    rows.into_iter().map(JobRow::into_domain).collect()
}

pub async fn job_exists(pool: &SqlitePool, job_id: Uuid) -> LlmResult<bool> {
    let found = sqlx::query_scalar::<_, i64>("SELECT 1 FROM jobs WHERE id = ?1")
        .bind(job_id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(|err| database_error("checking a job", err))?;
    Ok(found.is_some())
}

pub async fn set_status(pool: &SqlitePool, job_id: Uuid, status: &str) -> LlmResult<()> {
    sqlx::query("UPDATE jobs SET status = ?2, updated_at = ?3 WHERE id = ?1")
        .bind(job_id.to_string())
        .bind(status)
        .bind(timestamp(utc_now()))
        .execute(pool)
        .await
        .map_err(|err| database_error("updating a job status", err))?;
    Ok(())
}

pub async fn record_failure(
    pool: &SqlitePool,
    job_id: Uuid,
    code: &str,
    message: &str,
) -> LlmResult<()> {
    sqlx::query(
        "UPDATE jobs SET status = 'failed', error_code = ?2, error_message = ?3, \
         updated_at = ?4 WHERE id = ?1",
    )
    .bind(job_id.to_string())
    .bind(code)
    .bind(message)
    .bind(timestamp(utc_now()))
    .execute(pool)
    .await
    .map_err(|err| database_error("recording a job failure", err))?;
    Ok(())
}

pub async fn record_completion(
    pool: &SqlitePool,
    job_id: Uuid,
    library_versions: &Value,
) -> LlmResult<()> {
    sqlx::query(
        "UPDATE jobs SET status = 'completed', library_versions = ?2, \
         error_code = NULL, error_message = NULL, updated_at = ?3 WHERE id = ?1",
    )
    .bind(job_id.to_string())
    .bind(encode(library_versions, "library versions")?)
    .bind(timestamp(utc_now()))
    .execute(pool)
    .await
    .map_err(|err| database_error("completing a job", err))?;
    Ok(())
}

pub async fn mark_attempt(pool: &SqlitePool, job_id: Uuid) -> LlmResult<()> {
    sqlx::query(
        "UPDATE jobs SET attempts = attempts + 1, next_attempt_at = NULL, \
         updated_at = ?2 WHERE id = ?1",
    )
    .bind(job_id.to_string())
    .bind(timestamp(utc_now()))
    .execute(pool)
    .await
    .map_err(|err| database_error("marking a job attempt", err))?;
    Ok(())
}

pub async fn schedule_retry(
    pool: &SqlitePool,
    job_id: Uuid,
    next_attempt_at: DateTime<Utc>,
) -> LlmResult<()> {
    sqlx::query(
        "UPDATE jobs SET status = 'retrying', next_attempt_at = ?2, \
         updated_at = ?3 WHERE id = ?1",
    )
    .bind(job_id.to_string())
    .bind(timestamp(next_attempt_at))
    .bind(timestamp(utc_now()))
    .execute(pool)
    .await
    .map_err(|err| database_error("scheduling a job retry", err))?;
    Ok(())
}

/// Reset the jobs waiting for a retry and return their ids so the caller can
/// re-enqueue them.
pub async fn requeue_retrying(pool: &SqlitePool) -> LlmResult<Vec<Uuid>> {
    let ids = sqlx::query_scalar::<_, String>("SELECT id FROM jobs WHERE status = 'retrying'")
        .fetch_all(pool)
        .await
        .map_err(|err| database_error("listing retrying jobs", err))?;

    sqlx::query(
        "UPDATE jobs SET status = 'queued', next_attempt_at = NULL, updated_at = ?1 \
         WHERE status = 'retrying'",
    )
    .bind(timestamp(utc_now()))
    .execute(pool)
    .await
    .map_err(|err| database_error("requeuing retrying jobs", err))?;

    Ok(ids
        .into_iter()
        .filter_map(|id| Uuid::parse_str(&id).ok())
        .collect())
}

/// Fail jobs abandoned in an active status for longer than `cutoff`.
pub async fn fail_stale_active(
    pool: &SqlitePool,
    cutoff: DateTime<Utc>,
    message: &str,
) -> LlmResult<u64> {
    let result = sqlx::query(
        "UPDATE jobs SET status = 'failed', error_code = 'WORKER_INTERRUPTED', \
         error_message = ?1, updated_at = ?2 \
         WHERE status IN ('validating', 'training', 'evaluating', 'exporting') \
         AND updated_at < ?3",
    )
    .bind(message)
    .bind(timestamp(utc_now()))
    .bind(timestamp(cutoff))
    .execute(pool)
    .await
    .map_err(|err| database_error("failing interrupted jobs", err))?;
    Ok(result.rows_affected())
}

pub async fn update_metadata(
    pool: &SqlitePool,
    job_id: Uuid,
    name: Option<&str>,
    tags: &[String],
) -> LlmResult<()> {
    sqlx::query("UPDATE jobs SET name = ?2, tags = ?3, updated_at = ?4 WHERE id = ?1")
        .bind(job_id.to_string())
        .bind(name)
        .bind(encode(&tags, "tags")?)
        .bind(timestamp(utc_now()))
        .execute(pool)
        .await
        .map_err(|err| database_error("updating job metadata", err))?;
    Ok(())
}

/// Delete a job (metrics, artefacts and idempotency rows cascade). Returns
/// whether a row was removed.
pub async fn delete_job(pool: &SqlitePool, job_id: Uuid) -> LlmResult<bool> {
    let result = sqlx::query("DELETE FROM jobs WHERE id = ?1")
        .bind(job_id.to_string())
        .execute(pool)
        .await
        .map_err(|err| database_error("deleting a job", err))?;
    Ok(result.rows_affected() > 0)
}

pub async fn record_metrics(pool: &SqlitePool, metric: &JobMetric) -> LlmResult<()> {
    sqlx::query(
        r#"
        INSERT INTO job_metrics (job_id, metrics, charts, created_at)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(job_id) DO UPDATE SET
            metrics = excluded.metrics,
            charts = excluded.charts,
            created_at = excluded.created_at
        "#,
    )
    .bind(metric.job_id.to_string())
    .bind(encode(&metric.metrics, "metrics")?)
    .bind(match &metric.charts {
        Some(charts) => Some(encode(charts, "charts")?),
        None => None,
    })
    .bind(timestamp(metric.created_at))
    .execute(pool)
    .await
    .map_err(|err| database_error("recording job metrics", err))?;
    Ok(())
}

pub async fn get_metric(pool: &SqlitePool, job_id: Uuid) -> LlmResult<Option<JobMetric>> {
    let row = sqlx::query_as::<_, MetricRow>(
        "SELECT * FROM job_metrics WHERE job_id = ?1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(job_id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(|err| database_error("loading job metrics", err))?;
    row.map(MetricRow::into_domain).transpose()
}

pub async fn metrics_by_ids(
    pool: &SqlitePool,
    job_ids: &[Uuid],
) -> LlmResult<HashMap<Uuid, JobMetric>> {
    if job_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let mut builder =
        QueryBuilder::<Sqlite>::new("SELECT * FROM job_metrics WHERE job_id IN (");
    let mut separated = builder.separated(", ");
    for job_id in job_ids {
        separated.push_bind(job_id.to_string());
    }
    separated.push_unseparated(")");

    let rows = builder
        .build_query_as::<MetricRow>()
        .fetch_all(pool)
        .await
        .map_err(|err| database_error("loading job metrics", err))?;
    let mut metrics = HashMap::new();
    for row in rows {
        let metric = row.into_domain()?;
        metrics.insert(metric.job_id, metric);
    }
    Ok(metrics)
}

pub async fn delete_metrics(pool: &SqlitePool, job_id: Uuid) -> LlmResult<()> {
    sqlx::query("DELETE FROM job_metrics WHERE job_id = ?1")
        .bind(job_id.to_string())
        .execute(pool)
        .await
        .map_err(|err| database_error("deleting job metrics", err))?;
    Ok(())
}
