//! Job read/query operations.

use std::collections::HashMap;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use sqlx::{QueryBuilder, Row, Sqlite, SqlitePool};
use uuid::Uuid;

use llm_core::domain::Job;
use llm_core::error::{LlmError, LlmResult};

use crate::convert::{database_error, timestamp, JobRow};

use super::filters::{JobCursor, JobFilters, WorkerCounts};
use super::helpers::{comparable_value, push_bound, push_in, sort_expression};

fn encode_cursor(value: &str, id: &Uuid) -> String {
    let payload = format!("{}|{}", value, id);
    URL_SAFE_NO_PAD.encode(payload.as_bytes())
}

pub fn decode_cursor(cursor: &str) -> LlmResult<(String, Uuid)> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor.as_bytes())
        .map_err(|_| LlmError::validation("Invalid cursor"))?;
    let text = String::from_utf8(bytes).map_err(|_| LlmError::validation("Invalid cursor"))?;
    let mut parts = text.splitn(2, '|');
    let value = parts.next().ok_or_else(|| LlmError::validation("Invalid cursor"))?;
    let id = parts
        .next()
        .and_then(|raw| Uuid::parse_str(raw).ok())
        .ok_or_else(|| LlmError::validation("Invalid cursor"))?;
    Ok((value.to_string(), id))
}

#[derive(Debug)]
pub struct JobPage {
    pub items: Vec<Job>,
    pub next_cursor: Option<String>,
}

pub async fn list_jobs(
    pool: &SqlitePool,
    filters: &JobFilters,
    limit: usize,
    cursor: Option<&JobCursor>,
) -> LlmResult<JobPage> {
    let limit = if limit == 0 { 50 } else { limit.min(200) };
    let mut builder = QueryBuilder::<Sqlite>::new(
        "SELECT j.* FROM jobs j",
    );
    if !filters.tags.is_empty() {
        builder.push(" LEFT JOIN job_tags t ON t.job_id = j.id");
    }
    builder.push(" WHERE 1=1");
    push_in(&mut builder, "j.status", &filters.statuses);
    push_in(&mut builder, "j.task_type", &filters.task_types);
    push_in(&mut builder, "j.algorithm_id", &filters.algorithm_ids);
    if !filters.tags.is_empty() {
        builder.push(" AND t.tag IN (");
        let mut separated = builder.separated(", ");
        for tag in &filters.tags {
            separated.push_bind(tag.clone());
        }
        builder.push(")");
    }
    if let Some(q) = &filters.q {
        builder.push(" AND (instr(lower(j.algorithm_id), lower(?)) > 0 OR instr(lower(j.task_type), lower(?)) > 0)");
        builder.push_bind(q.clone());
        builder.push_bind(q.clone());
    }
    if let Some(from) = filters.created_from {
        builder.push(" AND j.created_at >= ?");
        builder.push_bind(timestamp(from));
    }
    if let Some(to) = filters.created_to {
        builder.push(" AND j.created_at <= ?");
        builder.push_bind(timestamp(to));
    }
    if let Some(cursor) = cursor {
        let sort_expr = sort_expression(filters);
        let comparator = if filters.ascending() { ">" } else { "<" };
        let tie_comparator = if filters.ascending() { ">" } else { "<" };
        builder.push(" AND (");
        builder.push(&sort_expr);
        builder.push(format!(" {} ", comparator));
        let value = comparable_value(filters, &cursor.value)?;
        push_bound(&mut builder, &value);
        builder.push(" OR (");
        builder.push(&sort_expr);
        builder.push(" = ");
        push_bound(&mut builder, &value);
        builder.push(format!(" AND j.id {} ?))", tie_comparator));
        builder.push_bind(cursor.id.to_string());
    }

    if !filters.tags.is_empty() {
        builder.push(" GROUP BY j.id");
    }
    builder.push(" ORDER BY ");
    builder.push(sort_expression(filters));
    builder.push(if filters.ascending() { " ASC" } else { " DESC" });
    builder.push(", j.id ");
    builder.push(if filters.ascending() { " ASC" } else { " DESC" });
    builder.push(" LIMIT ?");
    builder.push_bind((limit as i64) + 1);

    let query = builder.build_query_as::<JobRow>();
    let rows = query.fetch_all(pool).await.map_err(|err| database_error("listing jobs", err))?;
    let mut items: Vec<Job> = Vec::with_capacity(rows.len());
    let has_more = rows.len() > limit;
    let mut rows_iter = rows.into_iter();
    let mut count = 0;
    let mut last_row: Option<JobRow> = None;
    while let Some(row) = rows_iter.next() {
        count += 1;
        if count > limit {
            break;
        }
        last_row = Some(row.clone());
        items.push(row.into_domain()?);
    }
    let next_cursor = if has_more {
        if let Some(row) = last_row {
            let job = row.into_domain()?;
            let cursor_value = match filters.sort_field.as_str() {
                "createdAt" => timestamp(job.created_at),
                "updatedAt" => timestamp(job.updated_at),
                "status" => job.status.clone(),
                "algorithmId" => job.algorithm_id.clone(),
                _ => timestamp(job.created_at),
            };
            Some(encode_cursor(&cursor_value, &job.id))
        } else {
            None
        }
    } else {
        None
    };
    Ok(JobPage { items, next_cursor })
}

pub async fn queue_positions(pool: &SqlitePool, job_ids: &[Uuid]) -> LlmResult<HashMap<Uuid, i64>> {
    if job_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let mut positions = HashMap::new();
    for job_id in job_ids {
        let row = sqlx::query(
            "SELECT rank FROM (SELECT id, ROW_NUMBER() OVER (ORDER BY created_at ASC, id ASC) AS rank FROM jobs WHERE status = 'queued') WHERE id = ?1",
        )
        .bind(job_id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(|err| database_error("loading queue position", err))?;
        if let Some(row) = row {
            let rank: i64 = row.try_get("rank").map_err(|err| database_error("loading queue position", err))?;
            positions.insert(*job_id, rank);
        }
    }
    Ok(positions)
}

pub async fn queued_ids(pool: &SqlitePool) -> LlmResult<Vec<Uuid>> {
    let rows = sqlx::query("SELECT id FROM jobs WHERE status = 'queued' ORDER BY created_at ASC, id ASC")
        .fetch_all(pool)
        .await
        .map_err(|err| database_error("listing queued jobs", err))?;
    let mut ids = Vec::new();
    for row in rows {
        let id: String = row.try_get("id").map_err(|err| database_error("listing queued jobs", err))?;
        if let Ok(uuid) = Uuid::parse_str(&id) {
            ids.push(uuid);
        }
    }
    Ok(ids)
}

pub async fn worker_counts(pool: &SqlitePool) -> LlmResult<WorkerCounts> {
    let mut validating = 0i64;
    let mut training = 0i64;
    let mut evaluating = 0i64;
    let mut exporting = 0i64;
    let rows = sqlx::query("SELECT status, COUNT(*) as count FROM jobs WHERE status IN ('validating','training','evaluating','exporting') GROUP BY status")
        .fetch_all(pool)
        .await
        .map_err(|err| database_error("counting worker jobs", err))?;
    for row in rows {
        let status: String = row.try_get("status").map_err(|err| database_error("counting worker jobs", err))?;
        let count: i64 = row.try_get("count").map_err(|err| database_error("counting worker jobs", err))?;
        match status.as_str() {
            "validating" => validating = count,
            "training" => training = count,
            "evaluating" => evaluating = count,
            "exporting" => exporting = count,
            _ => {}
        }
    }
    Ok(WorkerCounts { validating, training, evaluating, exporting })
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
    let mut jobs = Vec::new();
    for row in rows {
        jobs.push(row.into_domain()?);
    }
    Ok(jobs)
}

pub async fn job_exists(pool: &SqlitePool, job_id: Uuid) -> LlmResult<bool> {
    let row = sqlx::query("SELECT 1 FROM jobs WHERE id = ?1")
        .bind(job_id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(|err| database_error("checking job existence", err))?;
    Ok(row.is_some())
}

pub async fn delete_job(pool: &SqlitePool, job_id: Uuid) -> LlmResult<bool> {
    let result = sqlx::query("DELETE FROM jobs WHERE id = ?1")
        .bind(job_id.to_string())
        .execute(pool)
        .await
        .map_err(|err| database_error("deleting a job", err))?;
    Ok(result.rows_affected() > 0)
}
