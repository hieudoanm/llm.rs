//! Background training worker.
//!
//! Mirrors the Python worker's lifecycle and resilience properties:
//!
//! * bounded concurrency (`SKL_MAX_CONCURRENT_JOBS`);
//! * a per-attempt timeout (`SKL_JOB_TIMEOUT_SECONDS`);
//! * full-jitter retry backoff capped at 30 seconds;
//! * deterministic failures are never retried.
//!
//! Each attempt advances a job through `validating -> training -> evaluating ->
//! exporting -> completed`, records metrics, and writes the ONNX artefact plus
//! the optional `.pkl`/`.joblib` snapshots.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{Duration as ChronoDuration, Utc};
use rand::Rng;
use serde_json::Value;
use tokio::sync::{mpsc, Semaphore};
use uuid::Uuid;

use mlrs_core::config::Settings;
use mlrs_core::dataframe::DataFrame;
use mlrs_core::domain::{utc_now, Artefact, Job, JobMetric};
use mlrs_core::export::onnx::export_onnx;
use mlrs_core::export::export_snapshot;
use mlrs_core::storage::{read_dataset, remove_job_files, write_artefact};
use mlrs_core::training::train;
use mlrs_db::jobs;
use mlrs_db::Database;

use crate::schemas::library_versions;

const MAX_BACKOFF_SECONDS: f64 = 30.0;
const INTERRUPTED_MESSAGE: &str = "The worker was interrupted before the job finished.";

#[derive(Clone)]
pub struct WorkerHandle {
    enqueue: mpsc::UnboundedSender<Uuid>,
    cancelled: Arc<Mutex<HashSet<Uuid>>>,
}

impl WorkerHandle {
    /// A handle with no worker attached (tests and embedded reads).
    pub fn noop() -> Self {
        let (enqueue, _receiver) = mpsc::unbounded_channel();
        Self {
            enqueue,
            cancelled: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn enqueue(&self, job_id: Uuid) {
        let _ = self.enqueue.send(job_id);
    }

    pub fn cancel(&self, job_id: Uuid) {
        if let Ok(mut cancelled) = self.cancelled.lock() {
            cancelled.insert(job_id);
        }
    }
}

fn is_cancelled(cancelled: &Arc<Mutex<HashSet<Uuid>>>, job_id: Uuid) -> bool {
    cancelled
        .lock()
        .map(|set| set.contains(&job_id))
        .unwrap_or(false)
}

#[derive(Debug)]
struct Failure {
    code: &'static str,
    message: String,
    transient: bool,
}

impl Failure {
    fn deterministic(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            transient: false,
        }
    }

    fn transient(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            transient: true,
        }
    }
}

impl From<mlrs_core::error::MlrsError> for Failure {
    fn from(error: mlrs_core::error::MlrsError) -> Self {
        let message = if error.message.is_empty() {
            format!("Job failed (code {})", error.code)
        } else {
            error.message
        };
        let transient = matches!(
            error.code.as_str(),
            "DATABASE_ERROR" | "SERVICE_UNAVAILABLE" | "INTERNAL_ERROR"
        );
        if transient {
            Failure::transient("INTERNAL_ERROR", message)
        } else {
            Failure::deterministic("TRAINING_FAILED", message)
        }
    }
}

/// Full-jitter backoff, capped at 30 seconds.
pub fn backoff_delay(attempt: i64, base_seconds: i64) -> f64 {
    let exponent = (attempt - 1).max(0) as u32;
    let ceiling = MAX_BACKOFF_SECONDS.min(base_seconds.max(0) as f64 * 2_f64.powi(exponent as i32));
    if ceiling <= 0.0 {
        return 0.0;
    }
    rand::thread_rng().gen::<f64>() * ceiling
}

/// Apply restart recovery and start the worker loop.
pub async fn start(db: Database, settings: Settings) -> WorkerHandle {
    let cutoff = Utc::now() - ChronoDuration::seconds(settings.worker_stale_seconds);
    if let Ok(count) = jobs::fail_stale_active(db.pool(), cutoff, INTERRUPTED_MESSAGE).await {
        if count > 0 {
            tracing::warn!("Failed {count} job(s) interrupted by a previous worker");
        }
    }

    let mut pending = Vec::new();
    match jobs::requeue_retrying(db.pool()).await {
        Ok(ids) => {
            if !ids.is_empty() {
                tracing::info!("Requeued {} retrying job(s)", ids.len());
            }
            pending.extend(ids);
        }
        Err(error) => tracing::error!("Failed to requeue retrying jobs: {error}"),
    }
    match jobs::queued_ids(db.pool()).await {
        Ok(ids) => pending.extend(ids),
        Err(error) => tracing::error!("Failed to load queued jobs: {error}"),
    }

    let (enqueue, mut receiver) = mpsc::unbounded_channel::<Uuid>();
    let cancelled = Arc::new(Mutex::new(HashSet::new()));
    let handle = WorkerHandle {
        enqueue: enqueue.clone(),
        cancelled: cancelled.clone(),
    };

    for job_id in pending {
        let _ = enqueue.send(job_id);
    }

    let semaphore = Arc::new(Semaphore::new(settings.max_concurrent_jobs.max(1) as usize));
    tokio::spawn(async move {
        while let Some(job_id) = receiver.recv().await {
            if is_cancelled(&cancelled, job_id) {
                continue;
            }
            let db = db.clone();
            let settings = settings.clone();
            let semaphore = semaphore.clone();
            let cancelled = cancelled.clone();
            tokio::spawn(async move {
                process_job(db, settings, semaphore, cancelled, job_id).await;
            });
        }
    });

    handle
}

async fn process_job(
    db: Database,
    settings: Settings,
    semaphore: Arc<Semaphore>,
    cancelled: Arc<Mutex<HashSet<Uuid>>>,
    job_id: Uuid,
) {
    let max_attempts = settings.job_max_attempts.max(1);
    for attempt in 1..=max_attempts {
        if is_cancelled(&cancelled, job_id) {
            return;
        }
        let Ok(permit) = semaphore.clone().acquire_owned().await else {
            return;
        };
        let timeout = Duration::from_secs(settings.job_timeout_seconds.max(1) as u64);
        let outcome = tokio::time::timeout(timeout, run_job(&db, job_id, &cancelled)).await;
        drop(permit);

        match outcome {
            Err(_) => {
                let seconds = settings.job_timeout_seconds;
                let _ = remove_artefacts(&db, job_id).await;
                let _ = jobs::record_failure(
                    db.pool(),
                    job_id,
                    "JOB_TIMEOUT",
                    &format!("Training exceeded the {seconds} s time limit and was stopped."),
                )
                .await;
                return;
            }
            Ok(Ok(())) => return,
            Ok(Err(failure)) => {
                if is_cancelled(&cancelled, job_id) {
                    return;
                }
                if !failure.transient {
                    tracing::error!(
                        "Job {job_id} failed ({}): {}",
                        failure.code,
                        failure.message
                    );
                    let _ = jobs::record_failure(
                        db.pool(),
                        job_id,
                        failure.code,
                        &failure.message,
                    )
                    .await;
                    return;
                }
                if attempt >= max_attempts {
                    tracing::error!(
                        "Job {job_id} exhausted its retry budget after {attempt} attempt(s): {}",
                        failure.message
                    );
                    let _ = jobs::record_failure(
                        db.pool(),
                        job_id,
                        "JOB_RETRY_EXHAUSTED",
                        &failure.message,
                    )
                    .await;
                    return;
                }
                let delay = backoff_delay(attempt, settings.job_retry_base_seconds);
                tracing::warn!(
                    "Job {job_id} attempt {attempt}/{max_attempts} failed transiently ({}); \
                     retrying in {delay:.2}s",
                    failure.code
                );
                let next = Utc::now() + ChronoDuration::milliseconds((delay * 1000.0) as i64);
                let _ = jobs::schedule_retry(db.pool(), job_id, next).await;
                tokio::time::sleep(Duration::from_secs_f64(delay)).await;
                if is_cancelled(&cancelled, job_id) {
                    return;
                }
                let _ = jobs::set_status(db.pool(), job_id, "queued").await;
            }
        }
    }
}

async fn remove_artefacts(db: &Database, job_id: Uuid) -> Result<(), mlrs_core::error::MlrsError> {
    mlrs_db::artefacts::delete_artefacts(db.pool(), job_id).await?;
    remove_job_files(job_id);
    Ok(())
}

fn ensure_not_deleted(cancelled: &Arc<Mutex<HashSet<Uuid>>>, job_id: Uuid) -> bool {
    !is_cancelled(cancelled, job_id)
}

async fn run_job(
    db: &Database,
    job_id: Uuid,
    cancelled: &Arc<Mutex<HashSet<Uuid>>>,
) -> Result<(), Failure> {
    let Some(job) = jobs::get_job(db.pool(), job_id)
        .await
        .map_err(Failure::from)?
    else {
        tracing::error!("Job {job_id} not found");
        return Ok(());
    };

    if is_cancelled(cancelled, job_id) {
        return Ok(());
    }

    tracing::info!(
        "Starting job {job_id} (task={}, algo={})",
        job.task_type,
        job.algorithm_id
    );

    // --- validating ---
    jobs::set_status(db.pool(), job_id, "validating")
        .await
        .map_err(Failure::from)?;

    let dataset = mlrs_db::datasets::get_dataset(db.pool(), job.dataset_id)
        .await
        .map_err(Failure::from)?
        .ok_or_else(|| Failure::deterministic("TRAINING_FAILED", "Dataset not found"))?;
    let bytes = read_dataset(&dataset.storage_key)
        .map_err(|error| Failure::deterministic("TRAINING_FAILED", error.message))?;
    let text = String::from_utf8(bytes).map_err(|error| {
        Failure::deterministic("TRAINING_FAILED", format!("Dataset is not UTF-8: {error}"))
    })?;
    let frame = DataFrame::from_csv_str(&text)
        .map_err(|error| Failure::deterministic("TRAINING_FAILED", error.message))?;

    validate_mapping(&job, &frame)?;

    // --- training ---
    jobs::set_status(db.pool(), job_id, "training")
        .await
        .map_err(Failure::from)?;
    let output = train(&job, &frame).map_err(Failure::from)?;

    if !ensure_not_deleted(cancelled, job_id) {
        return Ok(());
    }
    if !jobs::job_exists(db.pool(), job_id)
        .await
        .map_err(Failure::from)?
    {
        tracing::info!("Job {job_id} was deleted during training; stopping");
        return Ok(());
    }

    let metric = JobMetric {
        job_id,
        metrics: output.metrics.clone(),
        charts: Some(output.charts.clone()),
        created_at: utc_now(),
    };
    jobs::record_metrics(db.pool(), &metric)
        .await
        .map_err(Failure::from)?;

    // --- evaluating ---
    jobs::set_status(db.pool(), job_id, "evaluating")
        .await
        .map_err(Failure::from)?;

    // --- exporting ---
    jobs::set_status(db.pool(), job_id, "exporting")
        .await
        .map_err(Failure::from)?;

    if !jobs::job_exists(db.pool(), job_id)
        .await
        .map_err(Failure::from)?
    {
        tracing::info!("Job {job_id} was deleted before export; stopping");
        return Ok(());
    }

    let onnx = export_onnx(&output.model)
        .map_err(|error| Failure::deterministic("EXPORT_FAILED", error.message))?;
    record_artefact(db, job_id, "onnx", &format!("{job_id}.onnx"), &onnx).await?;

    if job.export_pkl {
        let snapshot = export_snapshot(&output.model)
            .map_err(|error| Failure::deterministic("EXPORT_FAILED", error.message))?;
        record_artefact(db, job_id, "pkl", &format!("{job_id}.pkl"), &snapshot).await?;
    }
    if job.export_joblib {
        let snapshot = export_snapshot(&output.model)
            .map_err(|error| Failure::deterministic("EXPORT_FAILED", error.message))?;
        record_artefact(
            db,
            job_id,
            "joblib",
            &format!("{job_id}.joblib"),
            &snapshot,
        )
        .await?;
    }

    jobs::record_completion(db.pool(), job_id, &library_versions())
        .await
        .map_err(Failure::from)?;
    tracing::info!("Job {job_id} completed");
    Ok(())
}

async fn record_artefact(
    db: &Database,
    job_id: Uuid,
    format: &str,
    filename: &str,
    bytes: &[u8],
) -> Result<(), Failure> {
    let (storage_key, _path, byte_size) =
        write_artefact(job_id, bytes, filename).map_err(|error| {
            Failure::deterministic("EXPORT_FAILED", error.message)
        })?;
    let artefact = Artefact {
        id: Uuid::new_v4(),
        job_id,
        format: format.to_string(),
        filename: filename.to_string(),
        byte_size,
        storage_key,
        created_at: utc_now(),
    };
    mlrs_db::artefacts::insert_artefact(db.pool(), &artefact)
        .await
        .map_err(Failure::from)?;
    Ok(())
}

fn validate_mapping(job: &Job, frame: &DataFrame) -> Result<(), Failure> {
    let missing: Vec<&String> = job
        .feature_columns
        .iter()
        .filter(|column| !frame.has_column(column))
        .collect();
    if !missing.is_empty() {
        return Err(Failure::deterministic(
            "TRAINING_FAILED",
            format!("INVALID_COLUMN_MAP: feature columns not in dataset: {missing:?}"),
        ));
    }

    if job.task_type == "reinforcement_learning" {
        if job.label_column.is_some() {
            return Err(Failure::deterministic(
                "TRAINING_FAILED",
                "INVALID_COLUMN_MAP: reinforcement_learning jobs must not set a label column",
            ));
        }
    } else if job.task_type == "classification"
        || job.task_type == "regression"
        || job.task_type == "semi_supervised"
    {
        let Some(label) = job.label_column.as_deref() else {
            return Err(Failure::deterministic(
                "TRAINING_FAILED",
                format!(
                    "INVALID_COLUMN_MAP: a label column is required for {}",
                    job.task_type
                ),
            ));
        };
        if !frame.has_column(label) {
            return Err(Failure::deterministic(
                "TRAINING_FAILED",
                format!("INVALID_COLUMN_MAP: label column not in dataset: {label}"),
            ));
        }
        if job.feature_columns.iter().any(|column| column == label) {
            return Err(Failure::deterministic(
                "TRAINING_FAILED",
                format!(
                    "INVALID_COLUMN_MAP: label column '{label}' is also a feature column"
                ),
            ));
        }
    }

    Ok(())
}

/// Convert a metric payload to JSON, accepting non-finite numbers as null.
pub fn finite(value: f64) -> Value {
    if value.is_finite() {
        serde_json::json!(value)
    } else {
        Value::Null
    }
}
