//! Helper utilities for job queries and serialization.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use llm_core::error::{LlmError, LlmResult};

pub const MAX_QUERY_LENGTH: usize = 200;
pub const DEFAULT_MAX_ATTEMPTS: i64 = 2;

pub const ACTIVE_STATUSES: [&str; 4] = ["validating", "training", "evaluating", "exporting"];

/// Sortable fields and the direction used when the caller omits `:asc`/`:desc`.
pub const SORT_FIELDS: [(&str, &str); 4] = [
    ("createdAt", "desc"),
    ("updatedAt", "desc"),
    ("status", "asc"),
    ("algorithmId", "asc"),
];

pub fn invalid(field: &str, message: impl Into<String>) -> LlmError {
    LlmError::validation(message.into()).with_details(serde_json::json!({"field": field}))
}

pub fn split(value: Option<&str>) -> Vec<String> {
    match value {
        Some(raw) if raw.trim().is_empty() => Vec::new(),
        Some(raw) => raw
            .split(',')
            .map(|part| part.trim().to_string())
            .filter(|part| !part.is_empty())
            .collect(),
        None => Vec::new(),
    }
}

pub fn parse_query_timestamp(field: &str, value: Option<&str>) -> LlmResult<Option<DateTime<Utc>>> {
    match value {
        Some(raw) if raw.trim().is_empty() => Ok(None),
        Some(raw) => {
            let trimmed = raw.trim();
            if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
                return Ok(Some(dt.with_timezone(&Utc)));
            }
            if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S") {
                return Ok(Some(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)));
            }
            if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S") {
                return Ok(Some(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)));
            }
            Err(invalid(field, "Invalid timestamp"))
        }
        None => Ok(None),
    }
}

pub fn parse_sort(value: Option<&str>) -> LlmResult<(String, String)> {
    match value {
        Some(raw) if raw.trim().is_empty() => Ok((SORT_FIELDS[0].0.to_string(), SORT_FIELDS[0].1.to_string())),
        Some(raw) => {
            let trimmed = raw.trim();
            let (field, direction) = if let Some(idx) = trimmed.rfind(':') {
                (trimmed[..idx].trim(), trimmed[idx + 1..].trim())
            } else {
                (trimmed, "")
            };
            let default_direction = SORT_FIELDS
                .iter()
                .find(|(candidate, _)| *candidate == field)
                .map(|(_, dir)| *dir)
                .unwrap_or("asc");
            let resolved_dir = match direction.to_ascii_lowercase().as_str() {
                "asc" => "asc",
                "desc" => "desc",
                "" => default_direction,
                _ => return Err(invalid("sort", "Invalid sort")),
            };
            let allowed = SORT_FIELDS.iter().any(|(candidate, _)| *candidate == field);
            if !allowed {
                return Err(invalid("sort", "Invalid sort"));
            }
            Ok((field.to_string(), resolved_dir.to_string()))
        }
        None => Ok((SORT_FIELDS[0].0.to_string(), SORT_FIELDS[0].1.to_string())),
    }
}

pub fn parse_cursor_timestamp(value: &str) -> LlmResult<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Ok(dt.with_timezone(&Utc));
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S") {
        return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc));
    }
    Err(LlmError::validation("Invalid cursor"))
}

#[derive(Debug, Clone)]
pub enum Comparable {
    Text(String),
    Numeric(i64),
    Timestamp(DateTime<Utc>),
}

pub fn comparable_value(filters: &crate::jobs::filters::JobFilters, value: &str) -> LlmResult<Comparable> {
    match filters.sort_field.as_str() {
        "createdAt" | "updatedAt" => parse_cursor_timestamp(value).map(Comparable::Timestamp),
        "status" | "algorithmId" => Ok(Comparable::Text(value.to_string())),
        _ => Err(LlmError::validation("Invalid cursor")),
    }
}

pub fn sort_expression(filters: &crate::jobs::filters::JobFilters) -> String {
    match filters.sort_field.as_str() {
        "createdAt" => "j.created_at".to_string(),
        "updatedAt" => "j.updated_at".to_string(),
        "status" => "j.status".to_string(),
        "algorithmId" => "j.algorithm_id".to_string(),
        _ => "j.created_at".to_string(),
    }
}

pub fn push_bound(
    builder: &mut sqlx::QueryBuilder<'_, sqlx::Sqlite>,
    value: &Comparable,
) {
    match value {
        Comparable::Text(v) => {
            builder.push_bind(v.clone());
        }
        Comparable::Numeric(v) => {
            builder.push_bind(*v);
        }
        Comparable::Timestamp(v) => {
            builder.push_bind(crate::convert::timestamp(*v));
        }
    }
}

pub fn push_in(
    builder: &mut sqlx::QueryBuilder<'_, sqlx::Sqlite>,
    column: &str,
    values: &[String],
) {
    if values.is_empty() {
        return;
    }
    builder.push(" AND ");
    builder.push(column);
    builder.push(" IN (");
    let mut separated = builder.separated(", ");
    for value in values {
        separated.push_bind(value.clone());
    }
    builder.push(")");
}

pub fn encode<T: serde::Serialize>(value: &T, field: &str) -> LlmResult<String> {
    let bytes = serde_json::to_vec(value).map_err(|err| {
        LlmError::validation("Invalid job payload").with_details(serde_json::json!({"field": field, "error": err.to_string()}))
    })?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

pub fn job_id_from_hash(value: &str) -> Uuid {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

pub fn job_key(
    dataset_id: &str,
    task_type: &str,
    algorithm_id: &str,
    features: &[String],
    label: Option<&str>,
    hyperparameters: &Value,
    split_ratio: f64,
    random_seed: i64,
) -> String {
    let mut canonical = serde_json::Map::new();
    canonical.insert("datasetId".to_string(), Value::String(dataset_id.to_string()));
    canonical.insert("taskType".to_string(), Value::String(task_type.to_string()));
    canonical.insert("algorithmId".to_string(), Value::String(algorithm_id.to_string()));
    let mut feature_list = features.to_vec();
    feature_list.sort();
    canonical.insert("features".to_string(), Value::Array(feature_list.into_iter().map(Value::String).collect()));
    canonical.insert("label".to_string(), match label {
        Some(value) => Value::String(value.to_string()),
        None => Value::Null,
    });
    let mut hyper = hyperparameters.clone();
    if let Value::Object(map) = &mut hyper {
        let mut keys: Vec<String> = map.keys().cloned().collect();
        keys.sort();
        let mut sorted = serde_json::Map::new();
        for key in keys {
            if let Some(value) = map.remove(&key) {
                sorted.insert(key, value);
            }
        }
        hyper = Value::Object(sorted);
    }
    canonical.insert("hyperparameters".to_string(), hyper);
    canonical.insert("splitRatio".to_string(), Value::from(split_ratio));
    canonical.insert("randomSeed".to_string(), Value::from(random_seed));
    let canonical = Value::Object(canonical);
    let bytes = serde_json::to_vec(&canonical).expect("canonical job key");
    URL_SAFE_NO_PAD.encode(&bytes)
}
