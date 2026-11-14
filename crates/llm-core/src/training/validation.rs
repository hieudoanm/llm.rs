//! Hyperparameter validation mirroring `src/training/core/validation.py`.

use serde_json::Value;

use crate::catalogue::allowed_hyperparameters;
use crate::error::{LlmError, LlmResult};

fn invalid(message: impl Into<String>) -> LlmError {
    LlmError::new(400, "INVALID_HYPERPARAMETERS", message)
}

fn int(value: Option<&Value>) -> Option<i64> {
    value.and_then(Value::as_i64)
}

fn number(value: Option<&Value>) -> Option<f64> {
    match value {
        Some(Value::Number(number)) => number.as_f64(),
        _ => None,
    }
}

pub fn validate_hyperparameters(
    algorithm_id: &str,
    hyperparameters: &Value,
    row_count: Option<i64>,
) -> LlmResult<()> {
    let object = hyperparameters
        .as_object()
        .ok_or_else(|| invalid("INVALID_HYPERPARAMETERS: hyperparameters must be an object"))?;

    let allowed = allowed_hyperparameters(algorithm_id);
    let unknown: Vec<&String> = object
        .keys()
        .filter(|key| !allowed.iter().any(|allowed_key| allowed_key == *key))
        .collect();
    if !unknown.is_empty() {
        let mut unknown: Vec<String> = unknown.into_iter().cloned().collect();
        unknown.sort();
        return Err(invalid(format!("unknown keys: {unknown:?}")));
    }

    let value = |key: &str| object.get(key);

    match algorithm_id {
        "self_training_classifier" => {
            let threshold = number(value("threshold")).unwrap_or(0.75);
            if !(0.5..=1.0).contains(&threshold) {
                return Err(invalid("threshold must be between 0.5 and 1.0"));
            }
            let max_iter = int(value("max_iter")).unwrap_or(10);
            if max_iter < 1 {
                return Err(invalid("max_iter must be an integer >= 1"));
            }
        }
        "kmeans" | "mini_batch_kmeans" => {
            validate_clusters(
                "n_clusters",
                int(value("n_clusters")).unwrap_or(3),
                row_count,
            )?;
        }
        "gaussian_mixture" => {
            validate_clusters(
                "n_components",
                int(value("n_components")).unwrap_or(3),
                row_count,
            )?;
        }
        "isolation_forest" => {
            if let Some(contamination) = value("contamination") {
                if contamination.as_str() != Some("auto") {
                    let parsed = number(Some(contamination))
                        .ok_or_else(|| invalid("contamination must be \"auto\" or in (0, 0.5]"))?;
                    if !(parsed > 0.0 && parsed <= 0.5) {
                        return Err(invalid("contamination must be \"auto\" or in (0, 0.5]"));
                    }
                }
            }
            let estimators = int(value("n_estimators")).unwrap_or(100);
            if estimators < 1 {
                return Err(invalid("n_estimators must be an integer >= 1"));
            }
        }
        "mlp_classifier" | "mlp_regressor" => validate_mlp(object)?,
        "fitted_q_iteration" => {
            let iterations = int(value("n_iterations")).unwrap_or(10);
            if !(1..=50).contains(&iterations) {
                return Err(invalid("n_iterations must be an integer between 1 and 50"));
            }
            let gamma = number(value("gamma")).unwrap_or(0.9);
            if !(0.0..=1.0).contains(&gamma) {
                return Err(invalid("gamma must be between 0 and 1"));
            }
            let depth = int(value("max_depth")).unwrap_or(3);
            if !(1..=20).contains(&depth) {
                return Err(invalid("max_depth must be an integer between 1 and 20"));
            }
            let learning_rate = number(value("learning_rate")).unwrap_or(0.1);
            if !(learning_rate > 0.0 && learning_rate <= 1.0) {
                return Err(invalid("learning_rate must be greater than 0 and <= 1"));
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_clusters(key: &str, value: i64, row_count: Option<i64>) -> LlmResult<()> {
    if value < 2 {
        return Err(invalid(format!("{key} must be an integer >= 2")));
    }
    if let Some(rows) = row_count {
        if value > rows - 1 {
            return Err(invalid(format!("{key} must be <= rows - 1 ({})", rows - 1)));
        }
    }
    Ok(())
}

fn validate_mlp(object: &serde_json::Map<String, Value>) -> LlmResult<()> {
    let value = |key: &str| object.get(key);
    let sizes = match value("hidden_layer_sizes") {
        None => vec![100i64],
        Some(Value::Array(items)) => {
            if items.is_empty() || items.len() > 5 {
                return Err(invalid(
                    "hidden_layer_sizes must be a list of 1 to 5 integers",
                ));
            }
            items
                .iter()
                .map(|item| {
                    item.as_i64()
                        .filter(|size| *size >= 1)
                        .ok_or_else(|| invalid("hidden_layer_sizes must be integers >= 1"))
                })
                .collect::<LlmResult<Vec<_>>>()?
        }
        Some(_) => {
            return Err(invalid(
                "hidden_layer_sizes must be a list of 1 to 5 integers",
            ))
        }
    };
    if sizes.len() > 5 {
        return Err(invalid(
            "hidden_layer_sizes must be a list of 1 to 5 integers",
        ));
    }

    let activation = value("activation")
        .and_then(Value::as_str)
        .unwrap_or("relu");
    if !matches!(activation, "identity" | "logistic" | "tanh" | "relu") {
        return Err(invalid(
            "activation must be one of [\"identity\", \"logistic\", \"relu\", \"tanh\"]",
        ));
    }
    let solver = value("solver").and_then(Value::as_str).unwrap_or("adam");
    if !matches!(solver, "lbfgs" | "sgd" | "adam") {
        return Err(invalid(
            "solver must be one of [\"adam\", \"lbfgs\", \"sgd\"]",
        ));
    }
    let alpha = number(value("alpha")).unwrap_or(0.0001);
    if alpha < 0.0 {
        return Err(invalid("alpha must be a number >= 0"));
    }
    let learning_rate = number(value("learning_rate_init")).unwrap_or(0.001);
    if !(learning_rate > 0.0 && learning_rate <= 1.0) {
        return Err(invalid("learning_rate_init must be in (0, 1]"));
    }
    let max_iter = int(value("max_iter")).unwrap_or(300);
    if max_iter < 1 {
        return Err(invalid("max_iter must be an integer >= 1"));
    }
    if let Some(batch_size) = value("batch_size") {
        if batch_size.as_str() != Some("auto") {
            match batch_size.as_i64() {
                Some(size) if size >= 1 => {}
                _ => return Err(invalid("batch_size must be \"auto\" or an integer >= 1")),
            }
        }
    }
    let validation_fraction = number(value("validation_fraction")).unwrap_or(0.1);
    if !(validation_fraction > 0.0 && validation_fraction < 1.0) {
        return Err(invalid("validation_fraction must be in (0, 1)"));
    }
    if let Some(early_stopping) = value("early_stopping") {
        if !early_stopping.is_boolean() {
            return Err(invalid("early_stopping must be a boolean"));
        }
    }
    Ok(())
}
