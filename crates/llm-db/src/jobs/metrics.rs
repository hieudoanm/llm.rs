//! Job metrics persistence.

use std::collections::HashMap;

use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use uuid::Uuid;

use llm_core::domain::JobMetric;
use llm_core::error::{LlmError, LlmResult};

use crate::convert::{database_error, timestamp, MetricRow};

fn json_encode<T: serde::Serialize>(value: &T, field: &str) -> LlmResult<String> {
    serde_json::to_string(value)
        .map_err(|err| LlmError::internal(format!("failed to encode {field}: {err}")))
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
    .bind(json_encode(&metric.metrics, "metrics")?)
    .bind(match &metric.charts {
        Some(charts) => Some(json_encode(charts, "charts")?),
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

#[cfg(test)]
mod tests {
    use super::*;
    use llm_core::domain::{Dataset, Job, JobMetric};
    use chrono::Utc;
    use serde_json::Value;
    use uuid::Uuid;
    use crate::datasets;
    use crate::jobs::writes::insert_job;

    fn create_test_dataset() -> Dataset {
        Dataset {
            id: Uuid::new_v4(),
            original_filename: "test.csv".to_string(),
            sha256: "abc123".to_string(),
            byte_size: 1000,
            storage_key: "test-key".to_string(),
            columns: vec![],
            row_count: 10,
            created_at: Utc::now(),
        }
    }

    async fn create_test_job(db: &crate::Database, dataset_id: Uuid) -> Uuid {
        let job_id = Uuid::new_v4();
        let job = Job {
            id: job_id,
            dataset_id,
            task_type: "classification".to_string(),
            algorithm_id: "algo-1".to_string(),
            feature_columns: vec![],
            label_column: Some("label".to_string()),
            rl_columns: None,
            hyperparameters: Value::Object(Default::default()),
            split_ratio: 0.8,
            random_seed: 42,
            export_pkl: false,
            export_joblib: true,
            status: "queued".to_string(),
            error_code: None,
            error_message: None,
            library_versions: None,
            name: None,
            tags: vec![],
            attempts: 0,
            max_attempts: 2,
            next_attempt_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        insert_job(db.pool(), &job).await.unwrap();
        job_id
    }

    fn create_test_metric(job_id: Uuid) -> JobMetric {
        JobMetric {
            job_id,
            metrics: serde_json::json!({"accuracy": 0.95, "loss": 0.05}),
            charts: Some(serde_json::json!({"type": "line"})),
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_record_and_get_metric() {
        let db = crate::Database::in_memory().await.unwrap();
        let dataset = create_test_dataset();
        datasets::insert_dataset(db.pool(), &dataset).await.unwrap();
        let job_id = create_test_job(&db, dataset.id).await;
        let metric = create_test_metric(job_id);
        record_metrics(db.pool(), &metric).await.unwrap();
        let retrieved = get_metric(db.pool(), job_id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.job_id, job_id);
        assert_eq!(retrieved.metrics["accuracy"].as_f64(), Some(0.95));
    }

    #[tokio::test]
    async fn test_get_metric_not_found() {
        let db = crate::Database::in_memory().await.unwrap();
        let result = get_metric(db.pool(), Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_record_metrics_update() {
        let db = crate::Database::in_memory().await.unwrap();
        let dataset = create_test_dataset();
        datasets::insert_dataset(db.pool(), &dataset).await.unwrap();
        let job_id = create_test_job(&db, dataset.id).await;
        let metric1 = create_test_metric(job_id);
        record_metrics(db.pool(), &metric1).await.unwrap();

        let mut metric2 = create_test_metric(job_id);
        metric2.metrics = serde_json::json!({"accuracy": 0.98, "loss": 0.02});
        record_metrics(db.pool(), &metric2).await.unwrap();

        let retrieved = get_metric(db.pool(), job_id).await.unwrap().unwrap();
        assert_eq!(retrieved.metrics["accuracy"].as_f64(), Some(0.98));
    }

    #[tokio::test]
    async fn test_metrics_by_ids_empty() {
        let db = crate::Database::in_memory().await.unwrap();
        let result = metrics_by_ids(db.pool(), &[]).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_metrics_by_ids() {
        let db = crate::Database::in_memory().await.unwrap();
        let dataset = create_test_dataset();
        datasets::insert_dataset(db.pool(), &dataset).await.unwrap();
        let job_id = create_test_job(&db, dataset.id).await;
        let metric = create_test_metric(job_id);
        record_metrics(db.pool(), &metric).await.unwrap();
        let result = metrics_by_ids(db.pool(), &[job_id]).await.unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains_key(&job_id));
    }

    #[tokio::test]
    async fn test_delete_metrics() {
        let db = crate::Database::in_memory().await.unwrap();
        let dataset = create_test_dataset();
        datasets::insert_dataset(db.pool(), &dataset).await.unwrap();
        let job_id = create_test_job(&db, dataset.id).await;
        let metric = create_test_metric(job_id);
        record_metrics(db.pool(), &metric).await.unwrap();
        delete_metrics(db.pool(), job_id).await.unwrap();
        let result = get_metric(db.pool(), job_id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_metrics_without_charts() {
        let db = crate::Database::in_memory().await.unwrap();
        let dataset = create_test_dataset();
        datasets::insert_dataset(db.pool(), &dataset).await.unwrap();
        let job_id = create_test_job(&db, dataset.id).await;
        let metric = JobMetric {
            job_id,
            metrics: serde_json::json!({"accuracy": 0.9}),
            charts: None,
            created_at: Utc::now(),
        };
        record_metrics(db.pool(), &metric).await.unwrap();
        let retrieved = get_metric(db.pool(), job_id).await.unwrap().unwrap();
        assert!(retrieved.charts.is_none());
    }
}
