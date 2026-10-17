//! Dataset and dataset-insight persistence.

use sqlx::SqlitePool;
use uuid::Uuid;

use llm_core::domain::{Dataset, DatasetInsight};
use llm_core::error::{LlmError, LlmResult};

use crate::convert::{database_error, timestamp, DatasetRow, InsightRow};

fn encode<T: serde::Serialize>(value: &T, field: &str) -> LlmResult<String> {
    serde_json::to_string(value)
        .map_err(|err| LlmError::internal(format!("failed to encode {field}: {err}")))
}

pub async fn insert_dataset(pool: &SqlitePool, dataset: &Dataset) -> LlmResult<()> {
    sqlx::query(
        r#"
        INSERT INTO datasets (
            id, original_filename, sha256, byte_size, storage_key,
            columns, row_count, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
    )
    .bind(dataset.id.to_string())
    .bind(&dataset.original_filename)
    .bind(&dataset.sha256)
    .bind(dataset.byte_size)
    .bind(&dataset.storage_key)
    .bind(encode(&dataset.columns, "dataset columns")?)
    .bind(dataset.row_count)
    .bind(timestamp(dataset.created_at))
    .execute(pool)
    .await
    .map_err(|err| database_error("inserting a dataset", err))?;
    Ok(())
}

pub async fn get_dataset(pool: &SqlitePool, dataset_id: Uuid) -> LlmResult<Option<Dataset>> {
    let row = sqlx::query_as::<_, DatasetRow>("SELECT * FROM datasets WHERE id = ?1")
        .bind(dataset_id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(|err| database_error("loading a dataset", err))?;
    row.map(DatasetRow::into_domain).transpose()
}

pub async fn insert_insight(pool: &SqlitePool, insight: &DatasetInsight) -> LlmResult<()> {
    sqlx::query(
        r#"
        INSERT INTO dataset_insights (
            dataset_id, row_count, column_count, total_missing,
            columns, warnings, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
    )
    .bind(insight.dataset_id.to_string())
    .bind(insight.row_count)
    .bind(insight.column_count)
    .bind(insight.total_missing)
    .bind(encode(&insight.columns, "insight columns")?)
    .bind(encode(&insight.warnings, "insight warnings")?)
    .bind(timestamp(insight.created_at))
    .execute(pool)
    .await
    .map_err(|err| database_error("inserting dataset insights", err))?;
    Ok(())
}

pub async fn get_insight(
    pool: &SqlitePool,
    dataset_id: Uuid,
) -> LlmResult<Option<DatasetInsight>> {
    let row = sqlx::query_as::<_, InsightRow>(
        "SELECT * FROM dataset_insights WHERE dataset_id = ?1",
    )
    .bind(dataset_id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(|err| database_error("loading dataset insights", err))?;
    row.map(InsightRow::into_domain).transpose()
}

#[cfg(test)]
mod tests {
use super::*;
use llm_core::domain::ColumnPreview;
use chrono::Utc;
use serde_json::json;

fn create_test_dataset() -> Dataset {
    Dataset {
        id: Uuid::new_v4(),
        original_filename: "test.csv".to_string(),
        sha256: "abc123".to_string(),
        byte_size: 1000,
        storage_key: "test-key".to_string(),
        columns: vec![ColumnPreview {
            name: "col1".to_string(),
            inferred_dtype: "f64".to_string(),
            null_count: 0,
            sample_values: vec![],
        }],
        row_count: 10,
        created_at: Utc::now(),
    }
}

fn create_test_insight() -> DatasetInsight {
    DatasetInsight {
        dataset_id: Uuid::new_v4(),
        row_count: 100,
        column_count: 5,
        total_missing: 3,
        columns: json!([{
            "name": "col1"
        }]),
        warnings: json!([{
            "type": "missing",
            "count": 3
        }]),
        created_at: Utc::now(),
    }
}

    #[tokio::test]
    async fn test_insert_and_get_dataset() {
        let db = crate::Database::in_memory().await.unwrap();
        let dataset = create_test_dataset();
        insert_dataset(db.pool(), &dataset).await.unwrap();
        let retrieved = get_dataset(db.pool(), dataset.id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.original_filename, "test.csv");
        assert_eq!(retrieved.row_count, 10);
    }

    #[tokio::test]
    async fn test_get_dataset_not_found() {
        let db = crate::Database::in_memory().await.unwrap();
        let result = get_dataset(db.pool(), Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_insert_and_get_insight() {
        let db = crate::Database::in_memory().await.unwrap();
        let dataset = create_test_dataset();
        insert_dataset(db.pool(), &dataset).await.unwrap();
        let insight = DatasetInsight {
            dataset_id: dataset.id,
            ..create_test_insight()
        };
        insert_insight(db.pool(), &insight).await.unwrap();
        let retrieved = get_insight(db.pool(), dataset.id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.row_count, 100);
        assert_eq!(retrieved.column_count, 5);
    }

    #[tokio::test]
    async fn test_get_insight_not_found() {
        let db = crate::Database::in_memory().await.unwrap();
        let result = get_insight(db.pool(), Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_insert_dataset_duplicate() {
        let db = crate::Database::in_memory().await.unwrap();
        let dataset = create_test_dataset();
        insert_dataset(db.pool(), &dataset).await.unwrap();
        let result = insert_dataset(db.pool(), &dataset).await;
        assert!(result.is_err());
    }
}
