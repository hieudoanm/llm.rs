//! Training command.

use anyhow::Result;
use llm_core::catalogue::find;
use llm_core::config::get_settings;
use llm_core::dataframe::DataFrame;
use llm_core::domain::{ArtefactFormat, Job, JobStatus, TaskType, utc_now};
use llm_core::export;
use llm_core::storage::{dataset_path, ensure_dirs, write_artefact};
use llm_core::training::{train, TrainingOutput};
use llm_db::{artefacts, datasets, jobs, Database};
use serde_json::{json, Value};
use uuid::Uuid;

pub async fn cmd_train(
    dataset_id_str: &str,
    task: &TaskType,
    algorithm_id: &str,
    features_str: &str,
    label: Option<&str>,
    hyperparameters_str: &str,
    split_ratio: f64,
    random_seed: i64,
) -> Result<()> {
    let dataset_id = Uuid::parse_str(dataset_id_str)
        .map_err(|e| anyhow::anyhow!("Invalid dataset ID: {}", e))?;

    let spec = find(algorithm_id)
        .ok_or_else(|| anyhow::anyhow!("Unknown algorithm: {}", algorithm_id))?;

    if spec.task_type != *task {
        return Err(anyhow::anyhow!(
            "Algorithm {} is for task {}, not {}",
            algorithm_id, spec.task_type, task
        ));
    }

    let hyperparameters: Value = serde_json::from_str(hyperparameters_str)
        .map_err(|e| anyhow::anyhow!("Invalid hyperparameters JSON: {}", e))?;

    let feature_columns: Vec<String> = features_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if feature_columns.is_empty() {
        return Err(anyhow::anyhow!("At least one feature column is required"));
    }

    let label_column = label.map(|s| s.to_string());

    let db = Database::connect(&get_settings()).await?;
    let dataset = datasets::get_dataset(db.pool(), dataset_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Dataset not found: {}", dataset_id))?;

    let df = DataFrame::from_csv_path(&dataset_path(&dataset.storage_key))
        .map_err(|e| anyhow::anyhow!("Failed to load dataset: {}", e))?;

    let job = Job {
        id: Uuid::new_v4(),
        dataset_id,
        task_type: task.as_str().to_string(),
        algorithm_id: algorithm_id.to_string(),
        feature_columns,
        label_column,
        rl_columns: None,
        hyperparameters,
        split_ratio,
        random_seed,
        export_pkl: true,
        export_joblib: true,
        status: JobStatus::Queued.as_str().to_string(),
        error_code: None,
        error_message: None,
        library_versions: Some(json!({
            "rust": env!("CARGO_PKG_VERSION"),
            "llm": env!("CARGO_PKG_VERSION"),
            "onnx": "1.x",
        })),
        name: None,
        tags: vec![],
        attempts: 0,
        max_attempts: 1,
        next_attempt_at: None,
        created_at: utc_now(),
        updated_at: utc_now(),
    };

    jobs::insert_job(db.pool(), &job).await?;
    jobs::set_status(db.pool(), job.id, JobStatus::Training.as_str()).await?;

    let result: Result<()> = async {
        println!("Training job {}...", job.id);
        println!("  Task: {}", task);
        println!("  Algorithm: {}", algorithm_id);
        println!("  Features: {:?}", job.feature_columns);
        println!("  Label: {:?}", job.label_column);

        let output: TrainingOutput = train(&job, &df)
            .map_err(|e| anyhow::anyhow!("Training failed: {}", e))?;

        jobs::set_status(db.pool(), job.id, JobStatus::Exporting.as_str()).await?;

        println!("\nTraining completed!");
        println!("  Model kind: {:?}", output.model.kind);
        println!("\nMetrics:");
        let metrics_json = serde_json::to_string_pretty(&output.metrics)?;
        println!("  {}", metrics_json);

        if !output.charts.as_object().map_or(true, |m| m.is_empty()) {
            println!("\nCharts:");
            println!("  {}", serde_json::to_string_pretty(&output.charts)?);
        }

        let onnx_bytes = export::onnx::export_onnx(&output.model)
            .map_err(|e| anyhow::anyhow!("ONNX export failed: {}", e))?;

        ensure_dirs()?;
        let (storage_key, path, byte_size) =
            write_artefact(job.id, &onnx_bytes, &format!("{}.onnx", job.id))?;

        let artefact = llm_core::domain::Artefact {
            id: Uuid::new_v4(),
            job_id: job.id,
            format: ArtefactFormat::Onnx.as_str().to_string(),
            filename: format!("{}.onnx", job.id),
            byte_size,
            storage_key,
            created_at: utc_now(),
        };
        artefacts::insert_artefact(db.pool(), &artefact).await?;

        jobs::set_status(db.pool(), job.id, JobStatus::Completed.as_str()).await?;

        let metric = llm_core::domain::JobMetric {
            job_id: job.id,
            metrics: output.metrics,
            charts: Some(output.charts),
            created_at: utc_now(),
        };
        jobs::record_metrics(db.pool(), &metric).await?;

        jobs::record_completion(db.pool(), job.id, Some(metric.metrics)).await?;

        println!("\nONNX exported to: {}", path.display());
        println!("  Size: {} bytes", onnx_bytes.len());
        println!("  Job {} completed", job.id);

        Ok(())
    }
    .await;

    if let Err(err) = &result {
        let _ = jobs::record_failure(
            db.pool(),
            job.id,
            Some("TRAINING_FAILED".to_string()),
            Some(err.to_string()),
        )
        .await;
    }
    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn module_exists() {
        assert!(true);
    }
}


