//! Dataset and dataset-insight persistence.

use sqlx::SqlitePool;
use uuid::Uuid;

use mlrs_core::domain::{Dataset, DatasetInsight};
use mlrs_core::error::{MlrsError, MlrsResult};

use crate::convert::{database_error, timestamp, DatasetRow, InsightRow};

fn encode<T: serde::Serialize>(value: &T, field: &str) -> MlrsResult<String> {
    serde_json::to_string(value)
        .map_err(|err| MlrsError::internal(format!("failed to encode {field}: {err}")))
}

pub async fn insert_dataset(pool: &SqlitePool, dataset: &Dataset) -> MlrsResult<()> {
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

pub async fn get_dataset(pool: &SqlitePool, dataset_id: Uuid) -> MlrsResult<Option<Dataset>> {
    let row = sqlx::query_as::<_, DatasetRow>("SELECT * FROM datasets WHERE id = ?1")
        .bind(dataset_id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(|err| database_error("loading a dataset", err))?;
    row.map(DatasetRow::into_domain).transpose()
}

pub async fn insert_insight(pool: &SqlitePool, insight: &DatasetInsight) -> MlrsResult<()> {
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
) -> MlrsResult<Option<DatasetInsight>> {
    let row = sqlx::query_as::<_, InsightRow>(
        "SELECT * FROM dataset_insights WHERE dataset_id = ?1",
    )
    .bind(dataset_id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(|err| database_error("loading dataset insights", err))?;
    row.map(InsightRow::into_domain).transpose()
}
