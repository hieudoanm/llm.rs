//! Export command.

use anyhow::Result;
use llm_core::config::get_settings;
use llm_core::storage::artefact_path;
use llm_db::{artefacts, Database};
use std::fs;
use std::path::Path;
use uuid::Uuid;

pub async fn cmd_export(job_id: &str, output: &Path, format: &str) -> Result<()> {
    let job_uuid = Uuid::parse_str(job_id).map_err(|e| anyhow::anyhow!("Invalid job ID: {}", e))?;

    let db = Database::connect(&get_settings()).await?;
    let artefacts = artefacts::list_artefacts(db.pool(), job_uuid).await?;

    let artefact = artefacts
        .iter()
        .find(|a| a.format == format)
        .ok_or_else(|| anyhow::anyhow!("No {} artefact found for job {}", format, job_id))?;

    let path = artefact_path(&artefact.storage_key, job_uuid);
    let bytes = fs::read(&path).map_err(|e| anyhow::anyhow!("Failed to read artefact: {}", e))?;

    fs::write(output, &bytes)?;
    println!("Exported {} artefact to: {}", format, output.display());
    println!("  Size: {} bytes", bytes.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn module_exists() {
        assert!(true);
    }
}
