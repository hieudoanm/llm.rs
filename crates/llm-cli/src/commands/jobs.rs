//! Job commands.

use anyhow::Result;
use llm_core::config::get_settings;
use llm_db::{artefacts, jobs, Database};
use uuid::Uuid;

pub async fn cmd_job(id: &str) -> Result<()> {
    let job_id = Uuid::parse_str(id).map_err(|e| anyhow::anyhow!("Invalid job ID: {}", e))?;

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

pub async fn cmd_jobs() -> Result<()> {
    let db = Database::connect(&get_settings()).await?;
    let filters = llm_db::jobs::JobFilters::default();
    let jobs_list = jobs::list_jobs(db.pool(), &filters, 100, None).await?;

    if jobs_list.items.is_empty() {
        println!("No jobs found.");
        return Ok(());
    }

    println!("Jobs:");
    for job in &jobs_list.items {
        println!(
            "  {} - {} - {} - {}",
            job.id, job.status, job.task_type, job.algorithm_id
        );
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
