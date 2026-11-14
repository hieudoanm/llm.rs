//! Dataset commands.

use anyhow::Result;
use llm_core::config::get_settings;
use llm_core::csv::parse_csv;
use llm_core::domain::utc_now;
use llm_core::storage::{datasets_dir, write_dataset};
use llm_db::{datasets, Database};
use std::fs;
use std::path::Path;

pub async fn cmd_dataset(path: &Path) -> Result<()> {
    let bytes = fs::read(path).map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;

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
        created_at: utc_now(),
    };
    datasets::insert_dataset(db.pool(), &dataset).await?;

    println!("Dataset uploaded:");
    println!("  id: {}", dataset_id);
    println!("  storage_key: {}", storage_key);
    println!("  sha256: {}", sha256);
    println!("  size: {} bytes", size);
    Ok(())
}

pub fn cmd_datasets() -> Result<()> {
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

#[cfg(test)]
mod tests {
    #[test]
    fn module_exists() {
        assert!(true);
    }
}
