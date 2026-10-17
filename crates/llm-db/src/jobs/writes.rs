//! Job write/lifecycle operations.

use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use llm_core::domain::Job;
use llm_core::error::{LlmError, LlmResult};

use crate::convert::{database_error, timestamp, JobRow};

use super::helpers::encode;

pub async fn insert_job(pool: &SqlitePool, job: &Job) -> LlmResult<()> {
    sqlx::query(
        r#"
        INSERT INTO jobs (
            id, dataset_id, task_type, algorithm_id, feature_columns, label_column,
            rl_columns, hyperparameters, split_ratio, random_seed, export_pkl,
            export_joblib, status, error_code, error_message, library_versions,
            name, tags, attempts, max_attempts, next_attempt_at, created_at, updated_at
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23)
        "#,
    )
    .bind(job.id.to_string())
    .bind(job.dataset_id.to_string())
    .bind(job.task_type.clone())
    .bind(job.algorithm_id.clone())
    .bind(encode(&job.feature_columns, "feature_columns")?)
    .bind(job.label_column.clone())
    .bind(match &job.rl_columns {
        Some(v) => Some(encode(v, "rl_columns")?),
        None => None,
    })
    .bind(encode(&job.hyperparameters, "hyperparameters")?)
    .bind(job.split_ratio)
    .bind(job.random_seed)
    .bind(if job.export_pkl { 1i64 } else { 0i64 })
    .bind(if job.export_joblib { 1i64 } else { 0i64 })
    .bind(job.status.clone())
    .bind(job.error_code.clone())
    .bind(job.error_message.clone())
    .bind(match &job.library_versions {
        Some(v) => Some(encode(v, "library_versions")?),
        None => None,
    })
    .bind(job.name.clone())
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

pub async fn set_status(pool: &SqlitePool, job_id: Uuid, status: &str) -> LlmResult<()> {
    let now = timestamp(Utc::now());
    sqlx::query("UPDATE jobs SET status = ?1, updated_at = ?2 WHERE id = ?3")
        .bind(status)
        .bind(now)
        .bind(job_id.to_string())
        .execute(pool)
        .await
        .map_err(|err| database_error("updating job status", err))?;
    Ok(())
}

pub async fn record_failure(
    pool: &SqlitePool,
    job_id: Uuid,
    error_code: Option<String>,
    error_message: Option<String>,
) -> LlmResult<()> {
    let now = timestamp(Utc::now());
    sqlx::query("UPDATE jobs SET status = 'failed', error_code = ?1, error_message = ?2, updated_at = ?3 WHERE id = ?4")
        .bind(error_code)
        .bind(error_message)
        .bind(now)
        .bind(job_id.to_string())
        .execute(pool)
        .await
        .map_err(|err| database_error("recording job failure", err))?;
    Ok(())
}

pub async fn record_completion(
    pool: &SqlitePool,
    job_id: Uuid,
    library_versions: Option<serde_json::Value>,
) -> LlmResult<()> {
    let now = timestamp(Utc::now());
    let versions = match library_versions {
        Some(v) => Some(encode(&v, "library_versions")?),
        None => None,
    };
    sqlx::query("UPDATE jobs SET status = 'completed', library_versions = COALESCE(?1, library_versions), updated_at = ?2 WHERE id = ?3")
        .bind(versions)
        .bind(now)
        .bind(job_id.to_string())
        .execute(pool)
        .await
        .map_err(|err| database_error("recording job completion", err))?;
    Ok(())
}

pub async fn mark_attempt(pool: &SqlitePool, job_id: Uuid) -> LlmResult<()> {
    let now = timestamp(Utc::now());
    sqlx::query("UPDATE jobs SET attempts = attempts + 1, updated_at = ?1 WHERE id = ?2")
        .bind(now)
        .bind(job_id.to_string())
        .execute(pool)
        .await
        .map_err(|err| database_error("marking job attempt", err))?;
    Ok(())
}

pub async fn schedule_retry(
    pool: &SqlitePool,
    job_id: Uuid,
    delay_seconds: i64,
) -> LlmResult<()> {
    let now = Utc::now();
    let next_attempt = now + chrono::Duration::seconds(delay_seconds);
    let now_ts = timestamp(now);
    let next_ts = timestamp(next_attempt);
    sqlx::query("UPDATE jobs SET status = 'retrying', next_attempt_at = ?1, updated_at = ?2 WHERE id = ?3")
        .bind(next_ts)
        .bind(now_ts)
        .bind(job_id.to_string())
        .execute(pool)
        .await
        .map_err(|err| database_error("scheduling job retry", err))?;
    Ok(())
}

pub async fn requeue_retrying(pool: &SqlitePool) -> LlmResult<Vec<Uuid>> {
    let now = timestamp(Utc::now());
    let rows = sqlx::query_as::<_, JobRow>(
        "UPDATE jobs SET status = 'queued', next_attempt_at = NULL, updated_at = ?1 WHERE status = 'retrying' AND (next_attempt_at IS NULL OR next_attempt_at <= ?1) RETURNING *",
    )
    .bind(now.clone())
    .bind(now)
    .fetch_all(pool)
    .await
    .map_err(|err| database_error("requeueing retrying jobs", err))?;
    let mut ids = Vec::new();
    for row in rows {
        if let Ok(job) = row.into_domain() {
            ids.push(job.id);
        }
    }
    Ok(ids)
}

pub async fn fail_stale_active(pool: &SqlitePool) -> LlmResult<Vec<Uuid>> {
    let now = timestamp(Utc::now());
    let rows = sqlx::query_as::<_, JobRow>(
        "UPDATE jobs SET status = 'failed', error_code = COALESCE(error_code, 'STALE_JOB'), error_message = COALESCE(error_message, 'Job was interrupted'), updated_at = ?1 WHERE status IN ('validating','training','evaluating','exporting') RETURNING *",
    )
    .bind(now)
    .fetch_all(pool)
    .await
    .map_err(|err| database_error("failing stale jobs", err))?;
    let mut ids = Vec::new();
    for row in rows {
        if let Ok(job) = row.into_domain() {
            ids.push(job.id);
        }
    }
    Ok(ids)
}

pub async fn update_metadata(
    pool: &SqlitePool,
    job_id: Uuid,
    name: Option<String>,
    tags: Vec<String>,
) -> LlmResult<Job> {
    let now = timestamp(Utc::now());
    sqlx::query("UPDATE jobs SET name = ?1, tags = ?2, updated_at = ?3 WHERE id = ?4")
        .bind(name)
        .bind(encode(&tags, "tags")?)
        .bind(now)
        .bind(job_id.to_string())
        .execute(pool)
        .await
        .map_err(|err| database_error("updating job metadata", err))?;
    let job = super::queries::get_job(pool, job_id)
        .await?
        .ok_or_else(|| LlmError::new(404, "JOB_NOT_FOUND", "Job not found"))?;
    Ok(job)
}
