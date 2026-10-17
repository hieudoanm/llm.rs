//! Job metrics persistence.

use std::collections::HashMap;

use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use uuid::Uuid;

use llm_core::domain::JobMetric;
use llm_core::error::LlmResult;

use crate::convert::{database_error, timestamp, MetricRow};
use super::helpers::encode;

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
