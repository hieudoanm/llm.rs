//! Domain types shared by the database, worker, API and training code.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

use crate::error::{MlrsError, MlrsResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    Classification,
    Regression,
    SemiSupervised,
    Clustering,
    AnomalyDetection,
    ReinforcementLearning,
}

impl TaskType {
    pub const ALL: [TaskType; 6] = [
        TaskType::Classification,
        TaskType::Regression,
        TaskType::SemiSupervised,
        TaskType::Clustering,
        TaskType::AnomalyDetection,
        TaskType::ReinforcementLearning,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            TaskType::Classification => "classification",
            TaskType::Regression => "regression",
            TaskType::SemiSupervised => "semi_supervised",
            TaskType::Clustering => "clustering",
            TaskType::AnomalyDetection => "anomaly_detection",
            TaskType::ReinforcementLearning => "reinforcement_learning",
        }
    }

    pub fn label_required(self) -> bool {
        matches!(
            self,
            TaskType::Classification | TaskType::Regression | TaskType::SemiSupervised
        )
    }

    pub fn label_forbidden(self) -> bool {
        !self.label_required()
    }
}

impl fmt::Display for TaskType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TaskType {
    type Err = MlrsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "classification" => Ok(TaskType::Classification),
            "regression" => Ok(TaskType::Regression),
            "semi_supervised" => Ok(TaskType::SemiSupervised),
            "clustering" => Ok(TaskType::Clustering),
            "anomaly_detection" => Ok(TaskType::AnomalyDetection),
            "reinforcement_learning" => Ok(TaskType::ReinforcementLearning),
            other => Err(MlrsError::validation(format!(
                "Invalid taskType: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Validating,
    Training,
    Evaluating,
    Exporting,
    Retrying,
    Completed,
    Failed,
}

impl JobStatus {
    pub const ALL: [JobStatus; 8] = [
        JobStatus::Queued,
        JobStatus::Validating,
        JobStatus::Training,
        JobStatus::Evaluating,
        JobStatus::Exporting,
        JobStatus::Retrying,
        JobStatus::Completed,
        JobStatus::Failed,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            JobStatus::Queued => "queued",
            JobStatus::Validating => "validating",
            JobStatus::Training => "training",
            JobStatus::Evaluating => "evaluating",
            JobStatus::Exporting => "exporting",
            JobStatus::Retrying => "retrying",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
        }
    }

    pub fn ordinal(self) -> i64 {
        JobStatus::ALL
            .iter()
            .position(|s| *s == self)
            .unwrap_or(0) as i64
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, JobStatus::Completed | JobStatus::Failed)
    }

    pub fn is_active(self) -> bool {
        matches!(
            self,
            JobStatus::Validating
                | JobStatus::Training
                | JobStatus::Evaluating
                | JobStatus::Exporting
        )
    }
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for JobStatus {
    type Err = MlrsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        JobStatus::ALL
            .iter()
            .copied()
            .find(|status| status.as_str() == s)
            .ok_or_else(|| MlrsError::validation(format!("Unknown job status: {s}")))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtefactFormat {
    Onnx,
    Pkl,
    Joblib,
}

impl ArtefactFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            ArtefactFormat::Onnx => "onnx",
            ArtefactFormat::Pkl => "pkl",
            ArtefactFormat::Joblib => "joblib",
        }
    }
}

impl fmt::Display for ArtefactFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ArtefactFormat {
    type Err = MlrsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "onnx" => Ok(ArtefactFormat::Onnx),
            "pkl" => Ok(ArtefactFormat::Pkl),
            "joblib" => Ok(ArtefactFormat::Joblib),
            other => Err(MlrsError::internal(format!(
                "Unknown artefact format: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ColumnPreview {
    pub name: String,
    pub inferred_dtype: String,
    pub null_count: i64,
    pub sample_values: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dataset {
    pub id: Uuid,
    pub original_filename: String,
    pub sha256: String,
    pub byte_size: i64,
    pub storage_key: String,
    pub columns: Vec<ColumnPreview>,
    pub row_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RLColumns {
    pub action_column: String,
    pub reward_column: String,
    #[serde(default)]
    pub next_state_columns: Vec<String>,
    #[serde(default)]
    pub done_column: Option<String>,
    #[serde(default)]
    pub episode_column: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub task_type: String,
    pub algorithm_id: String,
    pub feature_columns: Vec<String>,
    pub label_column: Option<String>,
    pub rl_columns: Option<RLColumns>,
    pub hyperparameters: Value,
    pub split_ratio: f64,
    pub random_seed: i64,
    pub export_pkl: bool,
    pub export_joblib: bool,
    pub status: String,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub library_versions: Option<Value>,
    pub name: Option<String>,
    pub tags: Vec<String>,
    pub attempts: i64,
    pub max_attempts: i64,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Job {
    pub fn task(&self) -> MlrsResult<TaskType> {
        self.task_type.parse()
    }

    pub fn status_enum(&self) -> MlrsResult<JobStatus> {
        self.status.parse()
    }

    pub fn hyperparameter<T: serde::de::DeserializeOwned>(&self, key: &str, default: T) -> T {
        self.hyperparameters
            .get(key)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or(default)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobMetric {
    pub job_id: Uuid,
    pub metrics: Value,
    pub charts: Option<Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artefact {
    pub id: Uuid,
    pub job_id: Uuid,
    pub format: String,
    pub filename: String,
    pub byte_size: i64,
    pub storage_key: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetInsight {
    pub dataset_id: Uuid,
    pub row_count: i64,
    pub column_count: i64,
    pub total_missing: i64,
    pub columns: Value,
    pub warnings: Value,
    pub created_at: DateTime<Utc>,
}

/// Canonical UTC now, matching the Python backend's `utc_now()`.
pub fn utc_now() -> DateTime<Utc> {
    Utc::now()
}
