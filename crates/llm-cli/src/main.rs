//! CLI entry point.

mod cli;
mod commands;

use clap::Parser;
use cli::{Cli, Commands};
use commands::{cmd_algorithms, cmd_dataset, cmd_datasets, cmd_export, cmd_job, cmd_jobs, cmd_train};
use llm_core::config::get_settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = get_settings();
    tracing_subscriber::fmt()
        .with_env_filter(settings.log_level.clone())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Algorithms { task } => {
            let task = task.as_deref().map(|t| t.parse()).transpose()?;
            cmd_algorithms(task)
        }
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
        } => {
            let task = task.parse::<llm_core::domain::TaskType>()?;
            cmd_train(
                &dataset_id,
                &task,
                &algorithm,
                &features,
                label.as_deref(),
                &hyperparameters,
                split_ratio,
                random_seed,
            )
            .await
        }
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
