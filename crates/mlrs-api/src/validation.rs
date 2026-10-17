//! Job-request validation and metadata normalisation.
//!
//! Mirrors `src/api/jobs_validation.py` and `src/api/jobs_metadata.py`.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use mlrs_core::catalogue::{
    self, MIN_CLASSES, MIN_LABELLED_PER_CLASS, MIN_LABELLED_ROWS,
};
use mlrs_core::dataframe::{parse_label, parse_number, DataFrame};
use mlrs_core::domain::{Dataset, RLColumns, TaskType};
use mlrs_core::error::MlrsError;
use mlrs_core::storage::read_dataset;
use mlrs_db::Database;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::schemas::CreateJobRequest;

pub const MAX_NAME_LENGTH: usize = 120;
pub const MAX_TAGS: usize = 10;
pub const MAX_TAG_LENGTH: usize = 32;

#[derive(Debug, Clone)]
pub struct ValidatedJob {
    pub dataset: Dataset,
    pub feature_columns: Vec<String>,
    pub label_column: Option<String>,
    pub rl_columns: Option<RLColumns>,
    pub hyperparameters: Value,
    pub split_ratio: f64,
    pub random_seed: i64,
    pub export_pkl: bool,
    pub export_joblib: bool,
}

fn label_required(task: TaskType) -> bool {
    matches!(
        task,
        TaskType::Classification | TaskType::Regression | TaskType::SemiSupervised
    )
}

fn load_frame(dataset: &Dataset) -> Result<DataFrame, ApiError> {
    let bytes = read_dataset(&dataset.storage_key).map_err(ApiError::from)?;
    let text = String::from_utf8(bytes).map_err(|err| {
        ApiError::validation(format!("Dataset is not valid UTF-8: {err}"))
    })?;
    DataFrame::from_csv_str(&text).map_err(ApiError::from)
}

pub async fn validate_job_request(
    db: &Database,
    body: &CreateJobRequest,
) -> ApiResult<ValidatedJob> {
    if body.feature_columns.is_empty() {
        return Err(ApiError::validation(
            "featureColumns must contain at least one column",
        ));
    }

    let dataset = mlrs_db::datasets::get_dataset(db.pool(), body.dataset_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| {
            ApiError::not_found(
                "DATASET_NOT_FOUND",
                format!("Dataset {} not found", body.dataset_id),
            )
        })?;

    let spec = catalogue::find(&body.algorithm_id).ok_or_else(|| {
        ApiError::new(
            400,
            "UNKNOWN_ALGORITHM",
            format!("Unknown algorithm: {}", body.algorithm_id),
        )
    })?;
    if spec.task_type != body.task_type {
        return Err(ApiError::new(
            400,
            "TASK_ALGORITHM_MISMATCH",
            format!(
                "Algorithm {} is for {}, not {}",
                body.algorithm_id,
                spec.task_type.as_str(),
                body.task_type.as_str()
            ),
        ));
    }

    let header: Vec<&str> = dataset.columns.iter().map(|c| c.name.as_str()).collect();

    // --- label / RL rules ---
    let mut label_column = body.label_column.clone();
    let mut rl_columns = body.rl_columns.clone();
    if body.task_type == TaskType::ReinforcementLearning {
        if label_column.as_deref().is_some_and(|value| !value.is_empty()) {
            return Err(ApiError::new(
                400,
                "INVALID_COLUMN_MAP",
                "reinforcement_learning jobs must not set a label column",
            )
            .with_details(json!({ "labelColumn": label_column })));
        }
        label_column = None;
        if rl_columns.is_none() {
            return Err(ApiError::new(
                400,
                "INVALID_COLUMN_MAP",
                "rlColumns required for reinforcement_learning",
            ));
        }
    } else if rl_columns.is_some() {
        return Err(ApiError::new(
            400,
            "INVALID_COLUMN_MAP",
            "rlColumns only allowed for reinforcement_learning",
        ));
    } else {
        rl_columns = None;
    }

    if label_required(body.task_type) && label_column.as_deref().unwrap_or("").is_empty() {
        return Err(ApiError::new(
            400,
            "INVALID_COLUMN_MAP",
            format!(
                "A label column is required for {}",
                body.task_type.as_str()
            ),
        )
        .with_details(json!({ "taskType": body.task_type.as_str() })));
    }
    if !label_required(body.task_type) && label_column.is_some() {
        return Err(ApiError::new(
            400,
            "INVALID_COLUMN_MAP",
            format!(
                "{} jobs must not set a label column",
                body.task_type.as_str()
            ),
        )
        .with_details(json!({ "labelColumn": label_column })));
    }

    let missing: Vec<&String> = body
        .feature_columns
        .iter()
        .filter(|column| !header.contains(&column.as_str()))
        .collect();
    if !missing.is_empty() {
        return Err(ApiError::new(
            400,
            "INVALID_COLUMN_MAP",
            format!("Feature columns not in dataset: {missing:?}"),
        )
        .with_details(json!({ "missing": missing })));
    }
    if let Some(label) = &label_column {
        if !header.contains(&label.as_str()) {
            return Err(ApiError::new(
                400,
                "INVALID_COLUMN_MAP",
                format!("Label column not in dataset: {label}"),
            )
            .with_details(json!({ "labelColumn": label })));
        }
        if body.feature_columns.iter().any(|column| column == label) {
            return Err(ApiError::new(
                400,
                "INVALID_COLUMN_MAP",
                format!("Label column '{label}' is also a feature column"),
            ));
        }
    }
    if let Some(columns) = &rl_columns {
        for name in std::iter::once(&columns.action_column)
            .chain(std::iter::once(&columns.reward_column))
            .chain(columns.next_state_columns.iter())
            .chain(columns.done_column.iter())
            .chain(columns.episode_column.iter())
        {
            if !header.contains(&name.as_str()) {
                return Err(ApiError::new(
                    400,
                    "INVALID_COLUMN_MAP",
                    format!("RL column not in dataset: {name}"),
                ));
            }
        }
    }

    let frame = load_frame(&dataset)?;

    // --- regression label must be numeric ---
    if body.task_type == TaskType::Regression {
        if let Some(label) = &label_column {
            if let Some(index) = frame.column_index(label) {
                let non_numeric = (0..frame.row_count().min(10)).any(|row| {
                    frame.get(row, index).is_some_and(|value| {
                        !value.trim().is_empty() && parse_number(value.trim()).is_none()
                    })
                });
                if non_numeric {
                    return Err(ApiError::new(
                        400,
                        "INVALID_LABEL_TYPE",
                        format!("Label column '{label}' must be numeric for regression"),
                    ));
                }
            }
        }
    }

    // --- hyperparameters ---
    if let Err(error) = mlrs_core::training::validation::validate_hyperparameters(
        &body.algorithm_id,
        &body.hyperparameters,
        Some(dataset.row_count),
    ) {
        return Err(ApiError::from(strip_prefix(error)));
    }

    // --- semi-supervised minimums ---
    if body.task_type == TaskType::SemiSupervised {
        let label = label_column.as_deref().unwrap_or_default();
        let index = frame.column_index(label).ok_or_else(|| {
            ApiError::new(
                400,
                "INVALID_COLUMN_MAP",
                format!("Label column not in dataset: {label}"),
            )
        })?;

        let mut labelled = 0_i64;
        let mut unlabelled = 0_i64;
        let mut class_counts: BTreeMap<String, i64> = BTreeMap::new();
        for row in 0..frame.row_count() {
            match frame.get(row, index).and_then(|raw| parse_label(&raw)) {
                Some(label) => {
                    labelled += 1;
                    *class_counts.entry(label).or_insert(0) += 1;
                }
                None => unlabelled += 1,
            }
        }
        let stats = json!({
            "labelledRows": labelled,
            "unlabelledRows": unlabelled,
            "classes": class_counts.len(),
            "smallestClass": class_counts.values().copied().min().unwrap_or(0),
        });
        if let Some(problem) = semi_supervised_problem(labelled, class_counts.len(), &class_counts)
        {
            return Err(ApiError::new(400, "INSUFFICIENT_LABELS", problem)
                .with_details(stats));
        }
    }

    Ok(ValidatedJob {
        dataset,
        feature_columns: body.feature_columns.clone(),
        label_column,
        rl_columns,
        hyperparameters: body.hyperparameters.clone(),
        split_ratio: body.split_ratio,
        random_seed: body.random_seed,
        export_pkl: body.export_pkl,
        export_joblib: body.export_joblib,
    })
}

fn strip_prefix(error: MlrsError) -> ApiError {
    let prefix = "INVALID_HYPERPARAMETERS: ";
    let message = error
        .message
        .strip_prefix(prefix)
        .unwrap_or(&error.message)
        .to_string();
    ApiError::new(400, "INVALID_HYPERPARAMETERS", message)
}

fn semi_supervised_problem(
    labelled_rows: i64,
    classes: usize,
    class_counts: &BTreeMap<String, i64>,
) -> Option<String> {
    if labelled_rows < MIN_LABELLED_ROWS as i64 {
        return Some(format!(
            "At least {MIN_LABELLED_ROWS} labelled rows are required; found {labelled_rows}."
        ));
    }
    if classes < MIN_CLASSES {
        return Some(format!(
            "At least {MIN_CLASSES} classes are required in the label column."
        ));
    }
    let smallest = class_counts.values().copied().min().unwrap_or(0);
    if smallest < MIN_LABELLED_PER_CLASS as i64 {
        return Some(format!(
            "Each class needs at least {MIN_LABELLED_PER_CLASS} labelled rows; \
             the smallest class has {smallest}."
        ));
    }
    None
}

pub fn normalise_name(name: Option<String>) -> ApiResult<Option<String>> {
    let Some(name) = name else {
        return Ok(None);
    };
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > MAX_NAME_LENGTH {
        return Err(ApiError::validation(format!(
            "name must be at most {MAX_NAME_LENGTH} characters"
        ))
        .with_details(json!({ "field": "name", "limit": MAX_NAME_LENGTH })));
    }
    Ok(Some(trimmed.to_string()))
}

fn valid_tag_char(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '-' | '_' | '.' | ' ')
}

pub fn normalise_tags(tags: Vec<String>) -> ApiResult<Vec<String>> {
    let mut normalised = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for raw in tags {
        let tag = raw.trim().to_lowercase();
        if tag.is_empty() {
            return Err(ApiError::validation("A tag must not be empty")
                .with_details(json!({ "field": "tags" })));
        }
        if tag.chars().count() > MAX_TAG_LENGTH {
            return Err(ApiError::validation(format!(
                "A tag must be at most {MAX_TAG_LENGTH} characters"
            ))
            .with_details(json!({ "field": "tags", "limit": MAX_TAG_LENGTH })));
        }
        if !tag.chars().all(valid_tag_char) {
            return Err(ApiError::validation(format!(
                "Invalid characters in tag: {raw}"
            ))
            .with_details(json!({ "field": "tags" })));
        }
        if seen.insert(tag.clone()) {
            normalised.push(tag);
        }
    }
    if normalised.len() > MAX_TAGS {
        return Err(ApiError::validation(format!(
            "A job may carry at most {MAX_TAGS} tags"
        ))
        .with_details(json!({ "field": "tags", "limit": MAX_TAGS })));
    }
    Ok(normalised)
}

/// Parse the comma-separated ``ids`` query parameter (comparison / bundle).
pub fn parse_job_ids(raw: Option<&str>, minimum: usize, maximum: usize) -> ApiResult<Vec<Uuid>> {
    let raw = raw.map(str::trim).filter(|value| !value.is_empty());
    let Some(raw) = raw else {
        return Err(ApiError::new(
            400,
            "VALIDATION_ERROR",
            format!("ids must contain between {minimum} and {maximum} job ids"),
        )
        .with_details(serde_json::json!({ "field": "ids", "min": minimum, "max": maximum })));
    };

    let mut ids = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for part in raw.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let Ok(id) = Uuid::parse_str(part) else {
            return Err(ApiError::new(
                400,
                "VALIDATION_ERROR",
                format!("Malformed job id: {part}"),
            )
            .with_details(serde_json::json!({ "field": "ids" })));
        };
        if seen.insert(id) {
            ids.push(id);
        }
    }

    if ids.len() < minimum || ids.len() > maximum {
        return Err(ApiError::new(
            400,
            "VALIDATION_ERROR",
            format!("ids must contain between {minimum} and {maximum} job ids"),
        )
        .with_details(serde_json::json!({ "field": "ids", "min": minimum, "max": maximum })));
    }

    Ok(ids)
}
