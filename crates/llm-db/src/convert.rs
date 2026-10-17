//! Row structs and conversions between SQLite and the domain types.

use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::Value;

use llm_core::domain::{
    Artefact, ColumnPreview, Dataset, DatasetInsight, Job, JobMetric, RLColumns,
};
use llm_core::error::{LlmError, LlmResult};

pub(crate) fn timestamp(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Nanos, true)
}

pub(crate) fn parse_timestamp(raw: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(raw)
        .map(|value| value.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use llm_core::domain::ColumnPreview;
    use uuid::Uuid;

    #[test]
    fn timestamp_roundtrip() {
        let dt = DateTime::parse_from_rfc3339("2025-06-15T12:30:45Z")
            .unwrap()
            .with_timezone(&Utc);
        let encoded = timestamp(dt);
        assert!(encoded.contains("2025-06-15T12:30:45"));
        let decoded = parse_timestamp(&encoded);
        assert_eq!(decoded, dt);
    }

    #[test]
    fn timestamp_invalid_falls_back_to_now() {
        let now = timestamp(Utc::now());
        assert!(now.contains('+') || now.ends_with('Z'));
    }

    #[test]
    fn optional_timestamp_none_returns_none() {
        assert!(optional_timestamp(None).is_none());
    }

    #[test]
    fn optional_timestamp_present_parses() {
        assert!(optional_timestamp(Some("2025-06-15T00:00:00Z")).is_some());
        assert!(optional_timestamp(Some("not-a-date")).is_some());
    }

    #[test]
    fn encode_decode_dataset_row() {
        let preview = ColumnPreview {
            name: "age".to_string(),
            inferred_dtype: "numeric".to_string(),
            null_count: 0,
            sample_values: vec![serde_json::json!(25)],
        };
        let ds = DatasetRow {
            id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            original_filename: "data.csv".to_string(),
            sha256: "abc".to_string(),
            byte_size: 100,
            storage_key: "key1".to_string(),
            columns: serde_json::to_string(&vec![preview]).unwrap(),
            row_count: 10,
            created_at: "2025-06-15T10:00:00Z".to_string(),
        };
        let domain: llm_core::domain::Dataset = ds.into_domain().unwrap();
        assert_eq!(domain.id, Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap());
        assert_eq!(domain.original_filename, "data.csv");
        assert_eq!(domain.byte_size, 100);
    }

    #[test]
    fn encode_decode_job_row_minimal() {
        let row = JobRow {
            id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            dataset_id: "123e4567-e89b-12d3-a456-426614174001".to_string(),
            task_type: "classification".to_string(),
            algorithm_id: "logistic".to_string(),
            feature_columns: serde_json::to_string(&vec!["x"]).unwrap(),
            label_column: None,
            rl_columns: None,
            hyperparameters: serde_json::to_string(&serde_json::json!({})).unwrap(),
            split_ratio: 0.3,
            random_seed: 42,
            export_pkl: 1,
            export_joblib: 0,
            status: "queued".to_string(),
            error_code: None,
            error_message: None,
            library_versions: None,
            name: Some("task1".to_string()),
            tags: serde_json::to_string(&serde_json::json!([])).unwrap(),
            attempts: 0,
            max_attempts: 2,
            next_attempt_at: None,
            created_at: "2025-06-15T08:00:00Z".to_string(),
            updated_at: "2025-06-15T08:00:01Z".to_string(),
        };
        let domain: llm_core::domain::Job = row.into_domain().unwrap();
        assert_eq!(domain.id, Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap());
        assert_eq!(domain.dataset_id, Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap());
        assert_eq!(domain.feature_columns, vec!["x".to_string()]);
        assert!(domain.export_pkl);
        assert!(!domain.export_joblib);
        assert_eq!(domain.max_attempts, 2);
    }

    #[test]
    fn encode_decode_metric_row() {
        let now = "2025-06-15T10:00:00Z".to_string();
        let row = MetricRow {
            job_id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            metrics: serde_json::to_string(&serde_json::json!({"loss": 0.5})).unwrap(),
            charts: None,
            created_at: now,
        };
        let domain: llm_core::domain::JobMetric = row.into_domain().unwrap();
        assert_eq!(domain.job_id, Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap());
        assert_eq!(domain.charts, None);
    }

    #[test]
    fn encode_decode_artefact_row() {
        let now = "2025-06-15T10:00:00Z".to_string();
        let row = ArtefactRow {
            id: "123e4567-e89b-12d3-a456-426614174002".to_string(),
            job_id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            format: "onnx".to_string(),
            filename: "model.onnx".to_string(),
            byte_size: 1024,
            storage_key: "key1".to_string(),
            created_at: now,
        };
        let domain: llm_core::domain::Artefact = row.into_domain().unwrap();
        assert_eq!(domain.format, "onnx");
        assert_eq!(domain.byte_size, 1024);
    }

    #[test]
    fn encode_decode_insight_row() {
        let now = "2025-06-15T10:00:00Z".to_string();
        let columns = serde_json::to_string(&serde_json::json!([{ "name": "a" }])).unwrap();
        let warnings = serde_json::to_string(&serde_json::json!([{ "type": "missing", "count": 3 }])).unwrap();
        let row = InsightRow {
            dataset_id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            row_count: 100,
            column_count: 5,
            total_missing: 10,
            columns,
            warnings,
            created_at: now,
        };
        let domain: llm_core::domain::DatasetInsight = row.into_domain().unwrap();
        assert_eq!(domain.row_count, 100);
        assert_eq!(domain.total_missing, 10);
    }
}

pub(crate) fn optional_timestamp(raw: Option<&str>) -> Option<DateTime<Utc>> {
    raw.map(parse_timestamp)
}

pub(crate) fn database_error(context: &str, err: sqlx::Error) -> LlmError {
    tracing::error!("database error while {context}: {err}");
    LlmError::database()
}

pub(crate) fn decode_json<T: serde::de::DeserializeOwned>(raw: &str, field: &str) -> LlmResult<T> {
    serde_json::from_str(raw).map_err(|err| {
        LlmError::internal(format!("failed to decode {field} from the database: {err}"))
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
    pub fn into_domain(self) -> LlmResult<Dataset> {
        Ok(Dataset {
            id: uuid::Uuid::parse_str(&self.id)
                .map_err(|err| LlmError::internal(format!("invalid dataset id: {err}")))?,
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

#[derive(Debug, Clone, sqlx::FromRow)]
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
    pub fn into_domain(self) -> LlmResult<Job> {
        Ok(Job {
            id: uuid::Uuid::parse_str(&self.id)
                .map_err(|err| LlmError::internal(format!("invalid job id: {err}")))?,
            dataset_id: uuid::Uuid::parse_str(&self.dataset_id).map_err(|err| {
                LlmError::internal(format!("invalid job dataset id: {err}"))
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
    pub fn into_domain(self) -> LlmResult<JobMetric> {
        Ok(JobMetric {
            job_id: uuid::Uuid::parse_str(&self.job_id)
                .map_err(|err| LlmError::internal(format!("invalid metric job id: {err}")))?,
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
    pub fn into_domain(self) -> LlmResult<Artefact> {
        Ok(Artefact {
            id: uuid::Uuid::parse_str(&self.id)
                .map_err(|err| LlmError::internal(format!("invalid artefact id: {err}")))?,
            job_id: uuid::Uuid::parse_str(&self.job_id).map_err(|err| {
                LlmError::internal(format!("invalid artefact job id: {err}"))
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
    pub fn into_domain(self) -> LlmResult<DatasetInsight> {
        Ok(DatasetInsight {
            dataset_id: uuid::Uuid::parse_str(&self.dataset_id)
                .map_err(|err| LlmError::internal(format!("invalid insight dataset id: {err}")))?,
            row_count: self.row_count,
            column_count: self.column_count,
            total_missing: self.total_missing,
            columns: decode_json(&self.columns, "insight columns")?,
            warnings: decode_json(&self.warnings, "insight warnings")?,
            created_at: parse_timestamp(&self.created_at),
        })
    }
}
