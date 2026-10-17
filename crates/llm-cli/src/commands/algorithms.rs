//! Algorithm listing command.

use anyhow::Result;
use llm_core::catalogue::algorithms;
use llm_core::domain::TaskType;

pub fn cmd_algorithms(task: Option<TaskType>) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd_algorithms_no_filter() {
        let result = cmd_algorithms(None);
        assert!(result.is_ok());
    }

    #[test]
    fn cmd_algorithms_with_filter() {
        let result = cmd_algorithms(Some(TaskType::Classification));
        assert!(result.is_ok());
    }

    #[test]
    fn cmd_algorithms_regression() {
        let result = cmd_algorithms(Some(TaskType::Regression));
        assert!(result.is_ok());
    }
}


