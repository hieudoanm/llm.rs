//! Local Learning Machine (llm.rs) CLI.
//!
//! Train ML models from CSV files and export ONNX artefacts, all from the
//! command line. Uses `llm-core` for training logic and `llm-db` for
//! persistence.

use clap::Parser;
use llm_core::catalogue::{algorithms, find};
use llm_core::config::get_settings;
use llm_core::csv::parse_csv;
use llm_core::dataframe::DataFrame;
use llm_core::domain::{ArtefactFormat, Job, JobStatus, TaskType};
use llm_core::storage::{
    artefact_path, datasets_dir, dataset_path, ensure_dirs, write_artefact, write_dataset,
};
use llm_core::training::{train, TrainingOutput};
use llm_db::{artefacts, datasets, jobs, Database};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(name = "llm", author, version, about = "Local Learning Machine - train ML models from CSV and export ONNX")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// List available algorithms
    Algorithms {
        #[arg(long)]
        task: Option<TaskType>,
    },
    /// Upload/register a CSV dataset
    Dataset {
        /// Path to CSV file
        path: PathBuf,
    },
    /// Train a model from a dataset
    Train {
        /// Dataset ID (from `llm dataset`)
        #[arg(long)]
        dataset_id: String,
        /// Task type (classification, regression, semi_supervised, clustering, anomaly_detection, reinforcement_learning)
        #[arg(long)]
        task: TaskType,
        /// Algorithm ID (use `llm algorithms` to see options)
        #[arg(long)]
        algorithm: String,
        /// Feature column names (comma-separated)
        #[arg(long)]
        features: String,
        /// Label column name (required for classification/regression/semi_supervised)
        #[arg(long)]
        label: Option<String>,
        /// Hyperparameters as JSON string
        #[arg(long, default_value = "{}")]
        hyperparameters: String,
        /// Train/test split ratio (0.0-1.0)
        #[arg(long, default_value = "0.2")]
        split_ratio: f64,
        /// Random seed
        #[arg(long, default_value = "42")]
        random_seed: i64,
    },
    /// Show job status and metrics
    Job {
        /// Job ID
        id: String,
    },
    /// Export a job's artefact to a file
    Export {
        /// Job ID
        job_id: String,
        /// Output file path
        output: PathBuf,
        /// Format (onnx, pkl, joblib)
        #[arg(long, default_value = "onnx")]
        format: String,
    },
    /// List all jobs
    Jobs,
    /// List all datasets
    Datasets,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = get_settings();
    tracing_subscriber::fmt()
        .with_env_filter(settings.log_level.clone())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Algorithms { task } => cmd_algorithms(task),
        Commands::Dataset { path } => cmd_dataset(&path).await,
        Commands::Train {
            dataset_id,
            task,
            algorithm,
            features,
            label,
            hyperparameters,
            split_ratio,
            random_seed,
        } => cmd_train(
            &dataset_id, &task, &algorithm, &features, label.as_deref(), &hyperparameters,
            split_ratio, random_seed,
        )
        .await,
        Commands::Job { id } => cmd_job(&id).await,
        Commands::Export {
            job_id,
            output,
            format,
        } => cmd_export(&job_id, &output, &format).await,
        Commands::Jobs => cmd_jobs().await,
        Commands::Datasets => cmd_datasets(),
    }
}

fn cmd_algorithms(task: Option<TaskType>) -> anyhow::Result<()> {
    let specs = algorithms();
    let filtered: Vec<&llm_core::catalogue::AlgorithmSpec> = if let Some(t) = task {
        specs.iter().filter(|s| s.task_type == t).collect()
    } else {
        specs.iter().collect()
    };

    println!("Available algorithms:\n");
    for spec in &filtered {
        println!(
            "  {} ({}) - {}",
            spec.id, spec.task_type, spec.description
        );
        println!("    Default hyperparameters: {}", spec.default_hyperparameters);
    }
    Ok(())
}

async fn cmd_dataset(path: &Path) -> anyhow::Result<()> {
    let bytes = fs::read(path)
        .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;

    let _ = parse_csv(&String::from_utf8_lossy(&bytes))?;

    let (dataset_id, storage_key, sha256, size) = write_dataset(&bytes)?;

    let db = Database::connect(&get_settings()).await?;
    let dataset = llm_core::domain::Dataset {
        id: dataset_id,
        original_filename: path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string(),
        sha256: sha256.clone(),
        byte_size: size,
        storage_key: storage_key.clone(),
        columns: Vec::new(),
        row_count: 0,
        created_at: llm_core::domain::utc_now(),
    };
    datasets::insert_dataset(db.pool(), &dataset).await?;

    println!("Dataset uploaded:");
    println!("  id: {}", dataset_id);
    println!("  storage_key: {}", storage_key);
    println!("  sha256: {}", sha256);
    println!("  size: {} bytes", size);
    Ok(())
}

async fn cmd_train(
    dataset_id_str: &str,
    task: &TaskType,
    algorithm_id: &str,
    features_str: &str,
    label: Option<&str>,
    hyperparameters_str: &str,
    split_ratio: f64,
    random_seed: i64,
) -> anyhow::Result<()> {
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
        created_at: llm_core::domain::utc_now(),
        updated_at: llm_core::domain::utc_now(),
    };

    jobs::insert_job(db.pool(), &job).await?;
    jobs::set_status(db.pool(), job.id, JobStatus::Training.as_str()).await?;

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

    // Export ONNX
    let onnx_bytes = llm_core::export::onnx::export_onnx(&output.model)
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
        created_at: llm_core::domain::utc_now(),
    };
    artefacts::insert_artefact(db.pool(), &artefact).await?;

    jobs::set_status(db.pool(), job.id, JobStatus::Completed.as_str()).await?;

    let metric = llm_core::domain::JobMetric {
        job_id: job.id,
        metrics: output.metrics,
        charts: Some(output.charts),
        created_at: llm_core::domain::utc_now(),
    };
    jobs::record_metrics(db.pool(), &metric).await?;

    jobs::record_completion(db.pool(), job.id, &metric.metrics).await?;

    println!("\nONNX exported to: {}", path.display());
    println!("  Size: {} bytes", onnx_bytes.len());
    println!("  Job {} completed", job.id);

    Ok(())
}

async fn cmd_job(id: &str) -> anyhow::Result<()> {
    let job_id = Uuid::parse_str(id)
        .map_err(|e| anyhow::anyhow!("Invalid job ID: {}", e))?;

    let db = Database::connect(&get_settings()).await?;
    let job = jobs::get_job(db.pool(), job_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Job not found: {}", id))?;

    let metrics = jobs::get_metric(db.pool(), job_id).await?;

    println!("Job: {}", job.id);
    println!("  Status: {}", job.status);
    println!("  Task: {}", job.task_type);
    println!("  Algorithm: {}", job.algorithm_id);
    println!("  Features: {:?}", job.feature_columns);
    println!("  Label: {:?}", job.label_column);
    println!("  Created: {}", job.created_at);
    println!("  Updated: {}", job.updated_at);

    if let Some(metrics) = metrics {
        println!("\nMetrics:");
        println!("  {}", serde_json::to_string_pretty(&metrics.metrics)?);
    }

    let artefacts = artefacts::list_artefacts(db.pool(), job_id).await?;
    if !artefacts.is_empty() {
        println!("\nArtefacts:");
        for a in &artefacts {
            println!("  {} ({}) - {} bytes", a.format, a.filename, a.byte_size);
        }
    }

    Ok(())
}

async fn cmd_export(job_id: &str, output: &Path, format: &str) -> anyhow::Result<()> {
    let job_uuid = Uuid::parse_str(job_id)
        .map_err(|e| anyhow::anyhow!("Invalid job ID: {}", e))?;

    let db = Database::connect(&get_settings()).await?;
    let artefacts = artefacts::list_artefacts(db.pool(), job_uuid).await?;

    let artefact = artefacts.iter().find(|a| a.format == format).ok_or_else(|| {
        anyhow::anyhow!(
            "No {} artefact found for job {}",
            format,
            job_id
        )
    })?;

    let path = artefact_path(&artefact.storage_key, job_uuid);
    let bytes = fs::read(&path)
        .map_err(|e| anyhow::anyhow!("Failed to read artefact: {}", e))?;

    fs::write(output, &bytes)?;
    println!("Exported {} artefact to: {}", format, output.display());
    println!("  Size: {} bytes", bytes.len());
    Ok(())
}

async fn cmd_jobs() -> anyhow::Result<()> {
    let db = Database::connect(&get_settings()).await?;
    let filters = llm_db::jobs::JobFilters::default();
    let jobs_list = jobs::list_jobs(db.pool(), &filters, None, 100).await?;

    if jobs_list.is_empty() {
        println!("No jobs found.");
        return Ok(());
    }

    println!("Jobs:");
    for job in &jobs_list {
        println!("  {} - {} - {} - {}", job.id, job.status, job.task_type, job.algorithm_id);
    }
    Ok(())
}

fn cmd_datasets() -> anyhow::Result<()> {
    let datasets_dir = datasets_dir();
    if !datasets_dir.exists() {
        println!("No datasets found.");
        return Ok(());
    }

    let entries = fs::read_dir(&datasets_dir)?;
    println!("Datasets:");
    for entry in entries {
        let entry = entry?;
        let dir_name = entry.file_name().to_string_lossy().to_string();
        let dataset_dir = entry.path();
        if dataset_dir.is_dir() {
            let csv_path = dataset_dir.join(format!("{}.csv", dir_name));
            if csv_path.exists() {
                println!("  {}", dir_name);
            }
        }
    }
    Ok(())
}
