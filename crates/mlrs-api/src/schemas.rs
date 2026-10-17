//! Request bodies and response builders for the mlrs HTTP API.
//!
//! The Python service used Pydantic models. Here the request shapes are serde
//! structs and responses are built as JSON values, which keeps the wire format
//! (camelCase keys, enum strings, RFC3339 timestamps) identical without a
//! second layer of mapping types.

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use mlrs_core::domain::{Artefact, Dataset, Job, JobMetric, RLColumns, TaskType};

/// RFC3339 timestamp with the `+00:00` offset used by the Python service.
pub fn timestamp(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Micros, false)
}

fn empty_object() -> Value {
    json!({})
}

fn default_split_ratio() -> f64 {
    0.2
}

fn default_random_seed() -> i64 {
    42
}

/// Deserialize a field while remembering whether it was present at all.
fn present<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateJobRequest {
    pub dataset_id: Uuid,
    pub feature_columns: Vec<String>,
    pub task_type: TaskType,
    pub algorithm_id: String,
    #[serde(default)]
    pub label_column: Option<String>,
    #[serde(default)]
    pub rl_columns: Option<RLColumns>,
    #[serde(default = "empty_object")]
    pub hyperparameters: Value,
    #[serde(default = "default_split_ratio")]
    pub split_ratio: f64,
    #[serde(default = "default_random_seed")]
    pub random_seed: i64,
    #[serde(default)]
    pub export_pkl: bool,
    #[serde(default)]
    pub export_joblib: bool,
}

impl CreateJobRequest {
    /// Canonical body used for `Idempotency-Key` hashing.
    pub fn canonical(&self) -> Value {
        json!({
            "datasetId": self.dataset_id,
            "featureColumns": self.feature_columns,
            "taskType": self.task_type.as_str(),
            "algorithmId": self.algorithm_id,
            "labelColumn": self.label_column,
            "rlColumns": self.rl_columns,
            "hyperparameters": self.hyperparameters,
            "splitRatio": self.split_ratio,
            "randomSeed": self.random_seed,
            "exportPkl": self.export_pkl,
            "exportJoblib": self.export_joblib,
        })
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateJobMetadataRequest {
    /// `Some(Some(value))` when present, `Some(None)` for an explicit null.
    #[serde(default, deserialize_with = "present")]
    pub name: Option<Option<String>>,
    #[serde(default, deserialize_with = "present")]
    pub tags: Option<Option<Vec<String>>>,
}

pub fn library_versions() -> Value {
    json!({
        "engine": "mlrs-rust",
        "mlrs-core": env!("CARGO_PKG_VERSION"),
        "onnx-protobuf": "0.2.3",
        "rust": std::env::consts::ARCH,
    })
}

pub fn artefact_value(artefact: &Artefact) -> Value {
    json!({
        "artefactId": artefact.id,
        "format": artefact.format,
        "filename": artefact.filename,
        "byteSize": artefact.byte_size,
    })
}

pub fn dataset_value(dataset: &Dataset) -> Value {
    json!({
        "datasetId": dataset.id,
        "sha256": dataset.sha256,
        "filename": dataset.original_filename,
        "byteSize": dataset.byte_size,
        "columns": dataset.columns,
        "rowCount": dataset.row_count,
        "createdAt": timestamp(dataset.created_at),
    })
}

pub fn job_accepted_value(job: &Job, queue_position: Option<i64>) -> Value {
    json!({
        "jobId": job.id,
        "status": job.status,
        "datasetId": job.dataset_id,
        "taskType": job.task_type,
        "algorithmId": job.algorithm_id,
        "createdAt": timestamp(job.created_at),
        "name": job.name,
        "tags": job.tags,
        "attempts": job.attempts,
        "maxAttempts": job.max_attempts,
        "nextAttemptAt": job.next_attempt_at.map(timestamp),
        "queuePosition": queue_position,
    })
}

pub fn job_detail_value(
    job: &Job,
    artefacts: &[Artefact],
    metric: Option<&JobMetric>,
    queue_position: Option<i64>,
) -> Value {
    let error = match &job.error_code {
        Some(code) => json!({
            "code": code,
            "message": job.error_message.clone().unwrap_or_default(),
        }),
        None => Value::Null,
    };
    json!({
        "jobId": job.id,
        "status": job.status,
        "datasetId": job.dataset_id,
        "taskType": job.task_type,
        "algorithmId": job.algorithm_id,
        "createdAt": timestamp(job.created_at),
        "updatedAt": timestamp(job.updated_at),
        "featureColumns": job.feature_columns,
        "labelColumn": job.label_column,
        "rlColumns": job.rl_columns,
        "hyperparameters": job.hyperparameters,
        "metrics": metric.map(|metric| metric.metrics.clone()),
        "charts": metric.and_then(|metric| metric.charts.clone()),
        "artefacts": artefacts.iter().map(artefact_value).collect::<Vec<_>>(),
        "libraryVersions": job.library_versions,
        "error": error,
        "name": job.name,
        "tags": job.tags,
        "attempts": job.attempts,
        "maxAttempts": job.max_attempts,
        "nextAttemptAt": job.next_attempt_at.map(timestamp),
        "queuePosition": queue_position,
    })
}

/// Serialise a domain value that already uses camelCase serde attributes.
pub fn domain_value<T: Serialize>(value: &T) -> Value {
    serde_json::to_value(value).unwrap_or(Value::Null)
}

pub fn parse_uuid(raw: &str, field: &str) -> Result<Uuid, crate::error::ApiError> {
    Uuid::parse_str(raw.trim()).map_err(|_| {
        crate::error::ApiError::validation(format!("Invalid {field}")).with_details(json!({ "field": field }))
    })
}
