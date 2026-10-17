//! Domain types shared by the database, worker, API and training code.

use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

use crate::error::{LlmError, LlmResult};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
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
    type Err = LlmError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "classification" => Ok(TaskType::Classification),
            "regression" => Ok(TaskType::Regression),
            "semi_supervised" => Ok(TaskType::SemiSupervised),
            "clustering" => Ok(TaskType::Clustering),
            "anomaly_detection" => Ok(TaskType::AnomalyDetection),
            "reinforcement_learning" => Ok(TaskType::ReinforcementLearning),
            other => Err(LlmError::validation(format!(
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
    type Err = LlmError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        JobStatus::ALL
            .iter()
            .copied()
            .find(|status| status.as_str() == s)
            .ok_or_else(|| LlmError::validation(format!("Unknown job status: {s}")))
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
    type Err = LlmError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "onnx" => Ok(ArtefactFormat::Onnx),
            "pkl" => Ok(ArtefactFormat::Pkl),
            "joblib" => Ok(ArtefactFormat::Joblib),
            other => Err(LlmError::internal(format!(
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
    pub fn task(&self) -> LlmResult<TaskType> {
        self.task_type.parse()
    }

    pub fn status_enum(&self) -> LlmResult<JobStatus> {
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

pub fn utc_now() -> DateTime<Utc> {
    chrono::Utc::now()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip_job_status() {
        for s in [
            JobStatus::Queued,
            JobStatus::Validating,
            JobStatus::Training,
            JobStatus::Evaluating,
            JobStatus::Exporting,
            JobStatus::Retrying,
            JobStatus::Completed,
            JobStatus::Failed,
        ] {
            let j = serde_json::to_string(&s).unwrap();
            let s2: JobStatus = serde_json::from_str(&j).unwrap();
            assert_eq!(s2, s);
        }
    }

    #[test]
    fn serde_roundtrip_task_type() {
        for t in [
            TaskType::Classification,
            TaskType::Regression,
            TaskType::SemiSupervised,
            TaskType::Clustering,
            TaskType::AnomalyDetection,
            TaskType::ReinforcementLearning,
        ] {
            let j = serde_json::to_string(&t).unwrap();
            let t2: TaskType = serde_json::from_str(&j).unwrap();
            assert_eq!(t2, t);
        }
    }

    #[test]
    fn serde_roundtrip_artefact_format() {
        for f in [ArtefactFormat::Onnx, ArtefactFormat::Pkl, ArtefactFormat::Joblib] {
            let j = serde_json::to_string(&f).unwrap();
            let f2: ArtefactFormat = serde_json::from_str(&j).unwrap();
            assert_eq!(f2, f);
        }
    }

    #[test]
    fn serde_roundtrip_structs() {
        let ds = Dataset {
            id: Uuid::new_v4(),
            original_filename: "f.csv".to_string(),
            sha256: "h".to_string(),
            byte_size: 100,
            storage_key: "k".to_string(),
            columns: vec![],
            row_count: 10,
            created_at: chrono::Utc::now(),
        };
        let j = serde_json::to_string(&ds).unwrap();
        let _ds2: Dataset = serde_json::from_str(&j).unwrap();

        let job = Job {
            id: Uuid::new_v4(),
            dataset_id: Uuid::new_v4(),
            task_type: "classification".to_string(),
            algorithm_id: "logistic_regression".to_string(),
            feature_columns: vec!["a".to_string()],
            label_column: Some("b".to_string()),
            rl_columns: None,
            hyperparameters: serde_json::json!({}),
            split_ratio: 0.2,
            random_seed: 42,
            export_pkl: false,
            export_joblib: false,
            status: "queued".to_string(),
            error_code: None,
            error_message: None,
            library_versions: None,
            name: None,
            tags: vec![],
            attempts: 0,
            max_attempts: 3,
            next_attempt_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        assert!(job.task().is_ok());
        assert!(job.status_enum().is_ok());
        assert_eq!(job.hyperparameter::<i32>("x", 5), 5);
        assert_eq!(job.hyperparameter::<i32>("missing", 7), 7);
        let j2 = serde_json::to_string(&job).unwrap();
        let _job2: Job = serde_json::from_str(&j2).unwrap();

        let artefact = Artefact {
            id: Uuid::new_v4(),
            job_id: Uuid::new_v4(),
            format: "onnx".to_string(),
            filename: "model.onnx".to_string(),
            byte_size: 100,
            storage_key: "s".to_string(),
            created_at: chrono::Utc::now(),
        };
        let j3 = serde_json::to_string(&artefact).unwrap();
        let _a2: Artefact = serde_json::from_str(&j3).unwrap();
    }

    #[test]
    fn display_fromstr() {
        assert_eq!(JobStatus::Completed.as_str(), "completed");
        assert_eq!(format!("{}", JobStatus::Failed), "failed");
        assert_eq!("training".parse::<JobStatus>().unwrap(), JobStatus::Training);
        assert!("bad".parse::<JobStatus>().is_err());
        assert!(JobStatus::Completed.is_terminal());
        assert!(JobStatus::Training.is_active());
        assert!(!JobStatus::Queued.is_active());
        assert!(!JobStatus::Queued.is_terminal());

        assert_eq!(format!("{}", ArtefactFormat::Onnx), "onnx");
        assert_eq!("pkl".parse::<ArtefactFormat>().unwrap(), ArtefactFormat::Pkl);
        assert!("joblib".parse::<ArtefactFormat>().unwrap() == ArtefactFormat::Joblib);
        assert!("bad".parse::<ArtefactFormat>().is_err());
    }

    #[test]
    fn rl_columns_serde() {
        let r = RLColumns {
            action_column: "a".to_string(),
            reward_column: "r".to_string(),
            next_state_columns: vec!["n".to_string()],
            done_column: Some("d".to_string()),
            episode_column: Some("e".to_string()),
        };
        let j = serde_json::to_string(&r).unwrap();
        let _r2: RLColumns = serde_json::from_str(&j).unwrap();
        let r2 = RLColumns {
            action_column: "a".to_string(),
            reward_column: "r".to_string(),
            next_state_columns: vec![],
            done_column: None,
            episode_column: None,
        };
        let j2 = serde_json::to_string(&r2).unwrap();
        let _r3: RLColumns = serde_json::from_str(&j2).unwrap();
    }

    #[test]
    fn other_types() {
        let c = ColumnPreview {
            name: "a".to_string(),
            inferred_dtype: "numeric".to_string(),
            null_count: 0,
            sample_values: vec![serde_json::json!(1)],
        };
        let j = serde_json::to_string(&c).unwrap();
        let _c2: ColumnPreview = serde_json::from_str(&j).unwrap();

        let di = DatasetInsight {
            dataset_id: Uuid::new_v4(),
            row_count: 1,
            column_count: 1,
            total_missing: 0,
            columns: serde_json::json!([]),
            warnings: serde_json::json!([]),
            created_at: chrono::Utc::now(),
        };
        let j2 = serde_json::to_string(&di).unwrap();
        let _di2: DatasetInsight = serde_json::from_str(&j2).unwrap();

        let jm = JobMetric {
            job_id: Uuid::new_v4(),
            metrics: serde_json::json!({"acc": 0.9}),
            charts: Some(serde_json::json!({})),
            created_at: chrono::Utc::now(),
        };
        let j3 = serde_json::to_string(&jm).unwrap();
        let _jm2: JobMetric = serde_json::from_str(&j3).unwrap();
    }

    #[test]
    fn fromstr_task_type() {
        assert_eq!("classification".parse::<TaskType>().unwrap(), TaskType::Classification);
        assert_eq!("regression".parse::<TaskType>().unwrap(), TaskType::Regression);
        assert_eq!("semi_supervised".parse::<TaskType>().unwrap(), TaskType::SemiSupervised);
        assert_eq!("clustering".parse::<TaskType>().unwrap(), TaskType::Clustering);
        assert_eq!("anomaly_detection".parse::<TaskType>().unwrap(), TaskType::AnomalyDetection);
        assert_eq!("reinforcement_learning".parse::<TaskType>().unwrap(), TaskType::ReinforcementLearning);
        assert!("bad".parse::<TaskType>().is_err());
    }

    #[test]
    fn utc_now_works() {
        let _ = chrono::Utc::now();
    }
}
