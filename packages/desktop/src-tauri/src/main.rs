#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use llm_core::catalogue::{algorithms, find};
use llm_core::config::get_settings;
use llm_core::csv::parse_csv;
use llm_core::dataframe::DataFrame;
use llm_core::domain::{Job, JobStatus, TaskType};
use llm_core::storage::{artefact_path, ensure_dirs, write_artefact, write_dataset};
use llm_core::training::{train, TrainingOutput};
use llm_db::{artefacts, datasets, jobs, Database};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::str::FromStr;
use std::sync::Mutex;
use uuid::Uuid;
use base64::Engine;

struct AppState {
    db: Mutex<Option<Database>>,
}

#[derive(Serialize, Deserialize)]
struct TrainRequest {
    dataset_id: String,
    task: String,
    algorithm: String,
    features: String,
    label: Option<String>,
    hyperparameters: Value,
    split_ratio: f64,
    random_seed: i64,
}

#[derive(Serialize, Deserialize)]
struct TrainResponse {
    job_id: String,
    status: String,
    metrics: Value,
    charts: Value,
}

#[tauri::command]
fn list_algorithms(task: Option<String>) -> Vec<serde_json::Value> {
    let specs = algorithms();
    specs
        .iter()
        .filter(|s| task.as_ref().map_or(true, |t| s.task_type.as_str() == t))
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "task_type": s.task_type.as_str(),
                "description": s.description,
                "default_hyperparameters": s.default_hyperparameters,
            })
        })
        .collect()
}

#[tauri::command]
async fn upload_dataset(state: tauri::State<'_, AppState>, path: String) -> Result<String, String> {
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    parse_csv(&String::from_utf8_lossy(&bytes)).map_err(|e| e.to_string())?;

    let (dataset_id, storage_key, sha256, size) = write_dataset(&bytes).map_err(|e| e.to_string())?;

    let db = state.db.lock().unwrap().as_ref().unwrap().clone();
    let dataset = llm_core::domain::Dataset {
        id: dataset_id,
        original_filename: std::path::Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string(),
        sha256,
        byte_size: size,
        storage_key,
        columns: Vec::new(),
        row_count: 0,
        created_at: llm_core::domain::utc_now(),
    };
    datasets::insert_dataset(db.pool(), &dataset).await.map_err(|e| e.to_string())?;

    Ok(dataset_id.to_string())
}

#[tauri::command]
async fn upload_dataset_from_bytes(
    state: tauri::State<'_, AppState>,
    bytes: Vec<u8>,
    filename: String,
) -> Result<String, String> {
    parse_csv(&String::from_utf8_lossy(&bytes)).map_err(|e| e.to_string())?;

    let (dataset_id, storage_key, sha256, size) = write_dataset(&bytes).map_err(|e| e.to_string())?;

    let db = state.db.lock().unwrap().as_ref().unwrap().clone();
    let dataset = llm_core::domain::Dataset {
        id: dataset_id,
        original_filename: filename,
        sha256,
        byte_size: size,
        storage_key,
        columns: Vec::new(),
        row_count: 0,
        created_at: llm_core::domain::utc_now(),
    };
    datasets::insert_dataset(db.pool(), &dataset).await.map_err(|e| e.to_string())?;

    Ok(dataset_id.to_string())
}

#[tauri::command]
async fn train_model(
    state: tauri::State<'_, AppState>,
    request: TrainRequest,
) -> Result<TrainResponse, String> {
    let task = TaskType::from_str(&request.task).map_err(|e| e.to_string())?;
    let spec = find(&request.algorithm)
        .ok_or_else(|| format!("Unknown algorithm: {}", request.algorithm))?;

    if spec.task_type != task {
        return Err(format!(
            "Algorithm {} is for task {}, not {}",
            request.algorithm, spec.task_type, task
        ));
    }

    let dataset_id = Uuid::parse_str(&request.dataset_id)
        .map_err(|e| e.to_string())?;

    let db = state.db.lock().unwrap().as_ref().unwrap().clone();
    let dataset = datasets::get_dataset(db.pool(), dataset_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Dataset not found: {}", dataset_id))?;

    let df = DataFrame::from_csv_path(&llm_core::storage::dataset_path(&dataset.storage_key))
        .map_err(|e| e.to_string())?;

    let feature_columns: Vec<String> = request
        .features
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if feature_columns.is_empty() {
        return Err("At least one feature column is required".to_string());
    }

    let job = Job {
        id: Uuid::new_v4(),
        dataset_id,
        task_type: task.as_str().to_string(),
        algorithm_id: request.algorithm.clone(),
        feature_columns,
        label_column: request.label.clone(),
        rl_columns: None,
        hyperparameters: request.hyperparameters,
        split_ratio: request.split_ratio,
        random_seed: request.random_seed,
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

    jobs::insert_job(db.pool(), &job).await.map_err(|e| e.to_string())?;
    jobs::set_status(db.pool(), job.id, JobStatus::Training.as_str())
        .await
        .map_err(|e| e.to_string())?;

    let output: TrainingOutput = train(&job, &df).map_err(|e| e.to_string())?;

    jobs::set_status(db.pool(), job.id, JobStatus::Exporting.as_str())
        .await
        .map_err(|e| e.to_string())?;

    let onnx_bytes = llm_core::export::onnx::export_onnx(&output.model)
        .map_err(|e| e.to_string())?;

    ensure_dirs().map_err(|e| e.to_string())?;
    let (storage_key, _path, byte_size) = write_artefact(
        job.id,
        &onnx_bytes,
        &format!("{}.onnx", job.id),
    )
    .map_err(|e| e.to_string())?;

    let artefact = llm_core::domain::Artefact {
        id: Uuid::new_v4(),
        job_id: job.id,
        format: llm_core::domain::ArtefactFormat::Onnx.as_str().to_string(),
        filename: format!("{}.onnx", job.id),
        byte_size,
        storage_key,
        created_at: llm_core::domain::utc_now(),
    };
    artefacts::insert_artefact(db.pool(), &artefact).await.map_err(|e| e.to_string())?;

    jobs::set_status(db.pool(), job.id, JobStatus::Completed.as_str())
        .await
        .map_err(|e| e.to_string())?;

    let metric = llm_core::domain::JobMetric {
        job_id: job.id,
        metrics: output.metrics.clone(),
        charts: Some(output.charts.clone()),
        created_at: llm_core::domain::utc_now(),
    };
    jobs::record_metrics(db.pool(), &metric).await.map_err(|e| e.to_string())?;
    jobs::record_completion(db.pool(), job.id, &output.metrics)
        .await
        .map_err(|e| e.to_string())?;

    Ok(TrainResponse {
        job_id: job.id.to_string(),
        status: "completed".to_string(),
        metrics: output.metrics,
        charts: output.charts,
    })
}

#[tauri::command]
async fn get_job(state: tauri::State<'_, AppState>, job_id: String) -> Result<Value, String> {
    let uuid = Uuid::parse_str(&job_id).map_err(|e| e.to_string())?;
    let db = state.db.lock().unwrap().as_ref().unwrap().clone();
    let job = jobs::get_job(db.pool(), uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Job not found: {}", job_id))?;
    Ok(serde_json::to_value(&job).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn list_jobs(state: tauri::State<'_, AppState>) -> Result<Value, String> {
    use llm_db::jobs::JobFilters;
    let db = state.db.lock().unwrap().as_ref().unwrap().clone();
    let filters = JobFilters::default();
    let job_list = jobs::list_jobs(db.pool(), &filters, None, 100)
        .await
        .map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(&job_list).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn patch_job(
    state: tauri::State<'_, AppState>,
    job_id: String,
    name: Option<String>,
    tags: Vec<String>,
) -> Result<Value, String> {
    let uuid = Uuid::parse_str(&job_id).map_err(|e| e.to_string())?;
    let db = state.db.lock().unwrap().as_ref().unwrap().clone();
    let mut job = jobs::get_job(db.pool(), uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Job not found: {}", job_id))?;
    job.name = name;
    job.tags = tags;
    jobs::insert_job(db.pool(), &job).await.map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(&job).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn get_dataset_insights(
    state: tauri::State<'_, AppState>,
    dataset_id: String,
) -> Result<Value, String> {
    let uuid = Uuid::parse_str(&dataset_id).map_err(|e| e.to_string())?;
    let db = state.db.lock().unwrap().as_ref().unwrap().clone();
    let insight = datasets::get_insight(db.pool(), uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Dataset insights not found: {}", dataset_id))?;
    Ok(serde_json::to_value(&insight).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn compare_jobs(state: tauri::State<'_, AppState>, ids: String) -> Result<Value, String> {
    let job_ids: Vec<Uuid> = ids
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| Uuid::parse_str(s.trim()).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let db = state.db.lock().unwrap().as_ref().unwrap().clone();
    let jobs_vec = jobs::jobs_by_ids(db.pool(), &job_ids)
        .await
        .map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(&jobs_vec).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn list_artefacts(
    state: tauri::State<'_, AppState>,
    job_id: String,
) -> Result<Value, String> {
    let uuid = Uuid::parse_str(&job_id).map_err(|e| e.to_string())?;
    let db = state.db.lock().unwrap().as_ref().unwrap().clone();
    let artefacts = artefacts::list_artefacts(db.pool(), uuid)
        .await
        .map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(&artefacts).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn download_artefact(
    state: tauri::State<'_, AppState>,
    job_id: String,
    artefact_id: String,
) -> Result<String, String> {
    let job_uuid = Uuid::parse_str(&job_id).map_err(|e| e.to_string())?;
    let artefact_uuid = Uuid::parse_str(&artefact_id).map_err(|e| e.to_string())?;
    let db = state.db.lock().unwrap().as_ref().unwrap().clone();
    let artefact = artefacts::get_artefact(db.pool(), job_uuid, artefact_uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Artefact not found: {}", artefact_id))?;
    let path = artefact_path(&artefact.storage_key, job_uuid);
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(get_settings().log_level.clone())
        .init();

    let db = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(Database::connect(&get_settings()))
        .expect("Failed to connect to database");

    tauri::Builder::default()
        .manage(AppState {
            db: Mutex::new(Some(db)),
        })
        .invoke_handler(tauri::generate_handler![
            list_algorithms,
            upload_dataset,
            upload_dataset_from_bytes,
            train_model,
            get_job,
            list_jobs,
            patch_job,
            get_dataset_insights,
            compare_jobs,
            list_artefacts,
            download_artefact,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
