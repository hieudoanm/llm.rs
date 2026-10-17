//! ZIP bundles of job artefacts (FEATURE-15).
//!
//! The archive is built in memory and returned as bytes; the caller attaches
//! the `Content-Disposition` header and streams it. Nothing is written to the
//! artefact volume.

use std::io::{Cursor, Write};

use serde_json::{json, Value};
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use mlrs_core::domain::{Artefact, Job};
use mlrs_core::error::{MlrsError, MlrsResult};
use mlrs_core::storage::artefact_path;

pub const SKIPPED_NO_ARTEFACTS: &str = "NO_ARTEFACTS";
pub const SKIPPED_NOT_COMPLETED: &str = "NOT_COMPLETED";

#[derive(Debug, Clone)]
pub struct BundleJob {
    pub job: Job,
    pub artefacts: Vec<Artefact>,
    pub metrics: Option<Value>,
}

#[derive(Debug, Default)]
pub struct BundleResult {
    pub bytes: Vec<u8>,
    pub missing_files: Vec<String>,
    pub skipped: Vec<Value>,
}

pub fn skipped_entry(job_id: Uuid, reason: &str) -> Value {
    json!({ "jobId": job_id, "reason": reason })
}

fn manifest(bundle_job: &BundleJob, files: &[Value]) -> Value {
    let job = &bundle_job.job;
    json!({
        "jobId": job.id,
        "name": job.name,
        "tags": job.tags,
        "taskType": job.task_type,
        "algorithmId": job.algorithm_id,
        "status": job.status,
        "createdAt": crate::schemas::timestamp(job.created_at),
        "libraryVersions": job.library_versions.clone().unwrap_or_else(|| json!({})),
        "files": files,
    })
}

pub fn build_bundle(bundle_jobs: &[BundleJob], skipped: Vec<Value>) -> MlrsResult<BundleResult> {
    let mut missing = Vec::new();
    let mut buffer = Vec::new();
    {
        let mut archive = ZipWriter::new(Cursor::new(&mut buffer));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        for bundle_job in bundle_jobs {
            let job = &bundle_job.job;
            let folder = job.id.to_string();
            let mut files: Vec<Value> = Vec::new();

            for artefact in &bundle_job.artefacts {
                let source = artefact_path(&artefact.storage_key, job.id);
                let Ok(bytes) = std::fs::read(&source) else {
                    missing.push(format!("{folder}/{}", artefact.filename));
                    continue;
                };
                let name = format!("{folder}/{}", artefact.filename);
                archive
                    .start_file(&name, options)
                    .map_err(|err| MlrsError::internal(format!("failed to add {name}: {err}")))?;
                archive
                    .write_all(&bytes)
                    .map_err(|err| MlrsError::internal(format!("failed to write {name}: {err}")))?;
                files.push(json!({
                    "path": artefact.filename,
                    "format": artefact.format,
                    "byteSize": artefact.byte_size,
                }));
            }

            if let Some(metrics) = &bundle_job.metrics {
                let payload = serde_json::to_vec_pretty(metrics)
                    .map_err(|err| MlrsError::internal(format!("failed to encode metrics: {err}")))?;
                let name = format!("{folder}/metrics.json");
                archive
                    .start_file(&name, options)
                    .map_err(|err| MlrsError::internal(format!("failed to add {name}: {err}")))?;
                archive
                    .write_all(&payload)
                    .map_err(|err| MlrsError::internal(format!("failed to write {name}: {err}")))?;
                files.push(json!({
                    "path": "metrics.json",
                    "format": "metrics",
                    "byteSize": payload.len(),
                }));
            }

            let manifest = serde_json::to_vec_pretty(&manifest(bundle_job, &files))
                .map_err(|err| MlrsError::internal(format!("failed to encode manifest: {err}")))?;
            let name = format!("{folder}/manifest.json");
            archive
                .start_file(&name, options)
                .map_err(|err| MlrsError::internal(format!("failed to add {name}: {err}")))?;
            archive
                .write_all(&manifest)
                .map_err(|err| MlrsError::internal(format!("failed to write {name}: {err}")))?;
        }

        if !skipped.is_empty() {
            let payload = serde_json::to_vec_pretty(&json!({ "skipped": skipped }))
                .map_err(|err| MlrsError::internal(format!("failed to encode skipped: {err}")))?;
            archive
                .start_file("skipped.json", options)
                .map_err(|err| MlrsError::internal(format!("failed to add skipped.json: {err}")))?;
            archive.write_all(&payload).map_err(|err| {
                MlrsError::internal(format!("failed to write skipped.json: {err}"))
            })?;
        }

        archive
            .finish()
            .map_err(|err| MlrsError::internal(format!("failed to finish the bundle: {err}")))?;
    }

    Ok(BundleResult {
        bytes: buffer,
        missing_files: missing,
        skipped,
    })
}
