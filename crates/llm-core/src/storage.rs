//! Filesystem storage for datasets and training artefacts.
//!
//! The layout matches the Python service so existing volume contents keep
//! working:
//!
//! ```text
//! <data_dir>/datasets/datasets/<id>/<id>.csv
//! <data_dir>/artefacts/artefacts/<job>/<job>.<ext>
//! ```
//!
//! Everything is resolved from the process-wide settings, which makes the
//! module safe to use from the server and from Tauri embeds.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::config::get_settings;
use crate::error::{LlmError, LlmResult};

pub fn data_dir() -> PathBuf {
    get_settings().data_dir
}

pub fn datasets_dir() -> PathBuf {
    data_dir().join("datasets")
}

pub fn artefacts_dir() -> PathBuf {
    data_dir().join("artefacts")
}

pub fn ensure_dirs() -> LlmResult<()> {
    std::fs::create_dir_all(datasets_dir())?;
    std::fs::create_dir_all(artefacts_dir())?;
    Ok(())
}

/// Build a deterministic, safe storage key.
pub fn storage_key_for(prefix: &str, entity_id: Uuid, suffix: &str) -> String {
    format!("{prefix}/{entity_id}/{entity_id}{suffix}")
}

pub fn sha256_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

pub fn sha256_of_file(path: &Path) -> LlmResult<String> {
    let bytes = std::fs::read(path)?;
    Ok(sha256_bytes(&bytes))
}

/// Persist an uploaded dataset and return `(id, storage_key, sha256, size)`.
pub fn write_dataset(bytes: &[u8]) -> LlmResult<(Uuid, String, String, i64)> {
    ensure_dirs()?;
    let dataset_id = Uuid::new_v4();
    let hash = sha256_bytes(bytes);
    let key = storage_key_for("datasets", dataset_id, ".csv");
    let dest = datasets_dir().join(&key);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&dest, bytes)?;
    Ok((dataset_id, key, hash, bytes.len() as i64))
}

pub fn dataset_path(storage_key: &str) -> PathBuf {
    datasets_dir().join(storage_key)
}

pub fn artefact_dir(job_id: Uuid) -> PathBuf {
    artefacts_dir().join(job_id.to_string())
}

/// Write an artefact and return `(storage_key, path, byte_size)`.
pub fn write_artefact(job_id: Uuid, data: &[u8], filename: &str) -> LlmResult<(String, PathBuf, i64)> {
    ensure_dirs()?;
    let dir = artefact_dir(job_id);
    std::fs::create_dir_all(&dir)?;
    let extension = filename.rsplit('.').next().unwrap_or("bin");
    let storage_key = storage_key_for("artefacts", job_id, &format!(".{extension}"));
    let dest = dir.join(storage_key.replace('/', "_"));
    std::fs::write(&dest, data)?;
    Ok((storage_key, dest, data.len() as i64))
}

pub fn artefact_path(storage_key: &str, job_id: Uuid) -> PathBuf {
    artefact_dir(job_id).join(storage_key.replace('/', "_"))
}

pub fn remove_job_files(job_id: Uuid) {
    let dir = artefact_dir(job_id);
    if dir.exists() {
        let _ = std::fs::remove_dir_all(dir);
    }
}

/// Read a stored dataset, returning a friendly error when it is gone.
pub fn read_dataset(storage_key: &str) -> LlmResult<Vec<u8>> {
    let path = dataset_path(storage_key);
    std::fs::read(&path).map_err(|err| {
        LlmError::new(
            404,
            "DATASET_NOT_FOUND",
            format!("Dataset file is missing: {}", err),
        )
    })
}
