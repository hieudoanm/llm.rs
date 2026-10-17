//! Row structs and conversions between SQLite and the domain types.

use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::Value;

use mlrs_core::domain::{
    Artefact, ColumnPreview, Dataset, DatasetInsight, Job, JobMetric, RLColumns,
};
use mlrs_core::error::{MlrsError, MlrsResult};

pub(crate) fn timestamp(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Nanos, true)
}

pub(crate) fn parse_timestamp(raw: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(raw)
        .map(|value| value.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

pub(crate) fn optional_timestamp(raw: Option<&str>) -> Option<DateTime<Utc>> {
    raw.map(parse_timestamp)
}

pub(crate) fn database_error(context: &str, err: sqlx::Error) -> MlrsError {
    tracing::error!("database error while {context}: {err}");
    MlrsError::database()
}

pub(crate) fn decode_json<T: serde::de::DeserializeOwned>(raw: &str, field: &str) -> MlrsResult<T> {
    serde_json::from_str(raw).map_err(|err| {
        MlrsError::internal(format!("failed to decode {field} from the database: {err}"))
    })
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct DatasetRow {
    pub id: String,
    pub original_filename: String,
    pub sha256: String,
    pub byte_size: i64,
    pub storage_key: String,
    pub columns: String,
    pub row_count: i64,
    pub created_at: String,
}

impl DatasetRow {
    pub fn into_domain(self) -> MlrsResult<Dataset> {
        Ok(Dataset {
            id: uuid::Uuid::parse_str(&self.id)
                .map_err(|err| MlrsError::internal(format!("invalid dataset id: {err}")))?,
            original_filename: self.original_filename,
            sha256: self.sha256,
            byte_size: self.byte_size,
            storage_key: self.storage_key,
            columns: decode_json::<Vec<ColumnPreview>>(&self.columns, "dataset columns")?,
            row_count: self.row_count,
            created_at: parse_timestamp(&self.created_at),
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct JobRow {
    pub id: String,
    pub dataset_id: String,
    pub task_type: String,
    pub algorithm_id: String,
    pub feature_columns: String,
    pub label_column: Option<String>,
    pub rl_columns: Option<String>,
    pub hyperparameters: String,
    pub split_ratio: f64,
    pub random_seed: i64,
    pub export_pkl: i64,
    pub export_joblib: i64,
    pub status: String,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub library_versions: Option<String>,
    pub name: Option<String>,
    pub tags: String,
    pub attempts: i64,
    pub max_attempts: i64,
    pub next_attempt_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl JobRow {
    pub fn into_domain(self) -> MlrsResult<Job> {
        Ok(Job {
            id: uuid::Uuid::parse_str(&self.id)
                .map_err(|err| MlrsError::internal(format!("invalid job id: {err}")))?,
            dataset_id: uuid::Uuid::parse_str(&self.dataset_id).map_err(|err| {
                MlrsError::internal(format!("invalid job dataset id: {err}"))
            })?,
            task_type: self.task_type,
            algorithm_id: self.algorithm_id,
            feature_columns: decode_json(&self.feature_columns, "feature columns")?,
            label_column: self.label_column,
            rl_columns: match self.rl_columns.as_deref() {
                Some(raw) => Some(decode_json::<RLColumns>(raw, "rl columns")?),
                None => None,
            },
            hyperparameters: decode_json::<Value>(&self.hyperparameters, "hyperparameters")?,
            split_ratio: self.split_ratio,
            random_seed: self.random_seed,
            export_pkl: self.export_pkl != 0,
            export_joblib: self.export_joblib != 0,
            status: self.status,
            error_code: self.error_code,
            error_message: self.error_message,
            library_versions: match self.library_versions.as_deref() {
                Some(raw) => Some(decode_json::<Value>(raw, "library versions")?),
                None => None,
            },
            name: self.name,
            tags: decode_json(&self.tags, "tags")?,
            attempts: self.attempts,
            max_attempts: self.max_attempts,
            next_attempt_at: optional_timestamp(self.next_attempt_at.as_deref()),
            created_at: parse_timestamp(&self.created_at),
            updated_at: parse_timestamp(&self.updated_at),
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct MetricRow {
    pub job_id: String,
    pub metrics: String,
    pub charts: Option<String>,
    pub created_at: String,
}

impl MetricRow {
    pub fn into_domain(self) -> MlrsResult<JobMetric> {
        Ok(JobMetric {
            job_id: uuid::Uuid::parse_str(&self.job_id)
                .map_err(|err| MlrsError::internal(format!("invalid metric job id: {err}")))?,
            metrics: decode_json(&self.metrics, "metrics")?,
            charts: match self.charts.as_deref() {
                Some(raw) => Some(decode_json::<Value>(raw, "charts")?),
                None => None,
            },
            created_at: parse_timestamp(&self.created_at),
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct ArtefactRow {
    pub id: String,
    pub job_id: String,
    pub format: String,
    pub filename: String,
    pub byte_size: i64,
    pub storage_key: String,
    pub created_at: String,
}

impl ArtefactRow {
    pub fn into_domain(self) -> MlrsResult<Artefact> {
        Ok(Artefact {
            id: uuid::Uuid::parse_str(&self.id)
                .map_err(|err| MlrsError::internal(format!("invalid artefact id: {err}")))?,
            job_id: uuid::Uuid::parse_str(&self.job_id).map_err(|err| {
                MlrsError::internal(format!("invalid artefact job id: {err}"))
            })?,
            format: self.format,
            filename: self.filename,
            byte_size: self.byte_size,
            storage_key: self.storage_key,
            created_at: parse_timestamp(&self.created_at),
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct InsightRow {
    pub dataset_id: String,
    pub row_count: i64,
    pub column_count: i64,
    pub total_missing: i64,
    pub columns: String,
    pub warnings: String,
    pub created_at: String,
}

impl InsightRow {
    pub fn into_domain(self) -> MlrsResult<DatasetInsight> {
        Ok(DatasetInsight {
            dataset_id: uuid::Uuid::parse_str(&self.dataset_id)
                .map_err(|err| MlrsError::internal(format!("invalid insight dataset id: {err}")))?,
            row_count: self.row_count,
            column_count: self.column_count,
            total_missing: self.total_missing,
            columns: decode_json(&self.columns, "insight columns")?,
            warnings: decode_json(&self.warnings, "insight warnings")?,
            created_at: parse_timestamp(&self.created_at),
        })
    }
}
