//! CLI argument parsing.

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "llm",
    author,
    version,
    about = "Local Learning Machine - train ML models from CSV and export ONNX"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    /// List available algorithms
    Algorithms {
        #[arg(long)]
        task: Option<String>,
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
        task: String,
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
    /// Run an MCP server over stdio JSON-RPC
    Mcp,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_algorithms() {
        let cli = Cli::parse_from(["llm", "algorithms"]);
        assert!(matches!(cli.command, Commands::Algorithms { task: None }));
    }

    #[test]
    fn parse_algorithms_with_task() {
        let cli = Cli::parse_from(["llm", "algorithms", "--task", "classification"]);
        match cli.command {
            Commands::Algorithms { task } => {
                assert_eq!(task.as_deref(), Some("classification"));
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn parse_dataset() {
        let cli = Cli::parse_from(["llm", "dataset", "data.csv"]);
        match cli.command {
            Commands::Dataset { path } => {
                assert_eq!(path.to_str().unwrap(), "data.csv");
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn parse_train() {
        let cli = Cli::parse_from([
            "llm",
            "train",
            "--dataset-id",
            "123e4567-e89b-12d3-a456-426614174000",
            "--task",
            "classification",
            "--algorithm",
            "logistic_regression",
            "--features",
            "a,b",
        ]);
        match cli.command {
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
                assert_eq!(dataset_id, "123e4567-e89b-12d3-a456-426614174000");
                assert_eq!(task, "classification");
                assert_eq!(algorithm, "logistic_regression");
                assert_eq!(features, "a,b");
                assert!(label.is_none());
                assert_eq!(hyperparameters, "{}");
                assert_eq!(split_ratio, 0.2);
                assert_eq!(random_seed, 42);
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn parse_train_with_label() {
        let cli = Cli::parse_from([
            "llm",
            "train",
            "--dataset-id",
            "123e4567-e89b-12d3-a456-426614174000",
            "--task",
            "classification",
            "--algorithm",
            "logistic_regression",
            "--features",
            "a,b",
            "--label",
            "y",
            "--hyperparameters",
            "{\"C\":1.0}",
            "--split-ratio",
            "0.3",
            "--random-seed",
            "123",
        ]);
        match cli.command {
            Commands::Train {
                label,
                hyperparameters,
                split_ratio,
                random_seed,
                ..
            } => {
                assert_eq!(label.as_deref(), Some("y"));
                assert_eq!(hyperparameters, "{\"C\":1.0}");
                assert_eq!(split_ratio, 0.3);
                assert_eq!(random_seed, 123);
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn parse_job() {
        let cli = Cli::parse_from(["llm", "job", "123e4567-e89b-12d3-a456-426614174000"]);
        match cli.command {
            Commands::Job { id } => {
                assert_eq!(id, "123e4567-e89b-12d3-a456-426614174000");
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn parse_export() {
        let cli = Cli::parse_from([
            "llm",
            "export",
            "123e4567-e89b-12d3-a456-426614174000",
            "model.onnx",
        ]);
        match cli.command {
            Commands::Export {
                job_id,
                output,
                format,
            } => {
                assert_eq!(job_id, "123e4567-e89b-12d3-a456-426614174000");
                assert_eq!(output.to_str().unwrap(), "model.onnx");
                assert_eq!(format, "onnx");
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn parse_export_with_format() {
        let cli = Cli::parse_from([
            "llm",
            "export",
            "123e4567-e89b-12d3-a456-426614174000",
            "model.pkl",
            "--format",
            "pkl",
        ]);
        match cli.command {
            Commands::Export { format, .. } => {
                assert_eq!(format, "pkl");
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn parse_jobs() {
        let cli = Cli::parse_from(["llm", "jobs"]);
        assert!(matches!(cli.command, Commands::Jobs));
    }

    #[test]
    fn parse_datasets() {
        let cli = Cli::parse_from(["llm", "datasets"]);
        assert!(matches!(cli.command, Commands::Datasets));
    }
}
