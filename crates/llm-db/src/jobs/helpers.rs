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
                .find(|(candidate, _)| candidate.eq_ignore_ascii_case(field))
                .map(|(_, dir)| *dir)
                .unwrap_or("asc");
            let resolved_dir = match direction.to_ascii_lowercase().as_str() {
                "asc" => "asc",
                "desc" => "desc",
                "" => default_direction,
                _ => return Err(invalid("sort", "Invalid sort")),
            };
            let canonical_field = SORT_FIELDS
                .iter()
                .find(|(candidate, _)| candidate.eq_ignore_ascii_case(field))
                .map(|(candidate, _)| candidate.to_string())
                .ok_or_else(|| invalid("sort", "Invalid sort"))?;
            Ok((canonical_field, resolved_dir.to_string()))
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
    builder: &mut sqlx::QueryBuilder<sqlx::Sqlite>,
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
    builder: &mut sqlx::QueryBuilder<sqlx::Sqlite>,
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

#[allow(clippy::too_many_arguments)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use base64::prelude::BASE64_URL_SAFE_NO_PAD;

    #[test]
    fn encode_and_decode_roundtrip() {
        let value = serde_json::json!({"name": "foo", "count": 5});
        let encoded = encode(&value, "payload").unwrap();
        assert!(!encoded.is_empty());
        let bytes = BASE64_URL_SAFE_NO_PAD.decode(encoded).unwrap();
        let decoded: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn encode_serializes_with_sorted_json_keys() {
        let mut canonical = serde_json::Map::new();
        canonical.insert("z".to_string(), serde_json::Value::String("first".to_string()));
        canonical.insert("a".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.2).unwrap()));
        let value = Value::Object(canonical);
        let encoded = encode(&value, "tag").unwrap();

        let bytes = BASE64_URL_SAFE_NO_PAD.decode(encoded).unwrap();
        let decoded: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn encode_handles_null_label_option() {
        let value = serde_json::json!(null);
        assert!(encode(&value, "label").is_ok());
    }

    #[test]
    fn job_id_from_hash_is_valid_uuid() {
        let id = job_id_from_hash("some-unique-value");
        assert!(Uuid::parse_str(&id.to_string()).is_ok());
        assert_ne!(id, job_id_from_hash("another-different-value"));
    }

    #[test]
    fn job_id_from_hash_deterministic() {
        let id1 = job_id_from_hash("hello");
        let id2 = job_id_from_hash("hello");
        assert_eq!(id1, id2);
    }

    // Additional tests for helper functions
    #[test]
    fn test_split_empty_string() {
        let result = split(Some(""));
        assert!(result.is_empty());
    }

    #[test]
    fn test_split_whitespace() {
        let result = split(Some("  "));
        assert!(result.is_empty());
    }

    #[test]
    fn test_split_none() {
        let result = split(None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_split_comma_separated() {
        let result = split(Some("validating,training,exporting"));
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "validating");
        assert_eq!(result[1], "training");
        assert_eq!(result[2], "exporting");
    }

    #[test]
    fn test_split_with_spaces() {
        let result = split(Some("validating,  training , exporting"));
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "validating");
        assert_eq!(result[1], "training");
        assert_eq!(result[2], "exporting");
    }

    #[test]
    fn test_split_single_value() {
        let result = split(Some("validating"));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "validating");
    }

    #[test]
    fn test_parse_query_timestamp_rfc3339() {
        let result = parse_query_timestamp("test", Some("2023-01-01T12:34:56Z"));
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_parse_query_timestamp_rfc3339_with_tz() {
        let result = parse_query_timestamp("test", Some("2023-01-01T12:34:56+05:30"));
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_parse_query_timestamp_iso_format() {
        let result = parse_query_timestamp("test", Some("2023-01-01T12:34:56"));
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_parse_query_timestamp_space_format() {
        let result = parse_query_timestamp("test", Some("2023-01-01 12:34:56"));
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_parse_query_timestamp_empty_string() {
        let result = parse_query_timestamp("test", Some("  "));
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_parse_query_timestamp_none() {
        let result = parse_query_timestamp("test", None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_parse_query_timestamp_invalid() {
        let result = parse_query_timestamp("test", Some("invalid-timestamp"));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_sort_default() {
        let result = parse_sort(None);
        assert!(result.is_ok());
        let (field, direction) = result.unwrap();
        assert_eq!(field, "createdAt");
        assert_eq!(direction, "desc");
    }

    #[test]
    fn test_parse_sort_field_only() {
        let result = parse_sort(Some("status"));
        assert!(result.is_ok());
        let (field, direction) = result.unwrap();
        assert_eq!(field, "status");
        assert_eq!(direction, "asc"); // default for status
    }

    #[test]
    fn test_parse_sort_field_with_asc() {
        let result = parse_sort(Some("createdAt:asc"));
        assert!(result.is_ok());
        let (field, direction) = result.unwrap();
        assert_eq!(field, "createdAt");
        assert_eq!(direction, "asc");
    }

    #[test]
    fn test_parse_sort_field_with_desc() {
        let result = parse_sort(Some("algorithmId:desc"));
        assert!(result.is_ok());
        let (field, direction) = result.unwrap();
        assert_eq!(field, "algorithmId");
        assert_eq!(direction, "desc");
    }

    #[test]
    fn test_parse_sort_invalid_field() {
        let result = parse_sort(Some("invalid:asc"));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_sort_invalid_direction() {
        let result = parse_sort(Some("createdAt:invalid"));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_sort_empty_string() {
        let result = parse_sort(Some("  "));
        assert!(result.is_ok());
        let (field, direction) = result.unwrap();
        assert_eq!(field, "createdAt");
        assert_eq!(direction, "desc");
    }

    #[test]
    fn test_parse_cursor_timestamp_rfc3339() {
        let result = parse_cursor_timestamp("2023-01-01T12:34:56Z");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_cursor_timestamp_iso_format() {
        let result = parse_cursor_timestamp("2023-01-01T12:34:56");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_cursor_timestamp_invalid() {
        let result = parse_cursor_timestamp("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_comparable_value_created_at() {
        let filters = crate::jobs::filters::JobFilters {
            sort_field: "createdAt".to_string(),
            sort_direction: "desc".to_string(),
            ..Default::default()
        };
        let result = comparable_value(&filters, "2023-01-01T12:34:56Z");
        assert!(result.is_ok());
        matches!(result.unwrap(), Comparable::Timestamp(_));
    }

    #[test]
    fn test_comparable_value_updated_at() {
        let filters = crate::jobs::filters::JobFilters {
            sort_field: "updatedAt".to_string(),
            sort_direction: "desc".to_string(),
            ..Default::default()
        };
        let result = comparable_value(&filters, "2023-01-01T12:34:56Z");
        assert!(result.is_ok());
        matches!(result.unwrap(), Comparable::Timestamp(_));
    }

    #[test]
    fn test_comparable_value_status() {
        let filters = crate::jobs::filters::JobFilters {
            sort_field: "status".to_string(),
            sort_direction: "asc".to_string(),
            ..Default::default()
        };
        let result = comparable_value(&filters, "completed");
        assert!(result.is_ok());
        matches!(result.unwrap(), Comparable::Text(s) if s == "completed");
    }

    #[test]
    fn test_comparable_value_algorithm_id() {
        let filters = crate::jobs::filters::JobFilters {
            sort_field: "algorithmId".to_string(),
            sort_direction: "asc".to_string(),
            ..Default::default()
        };
        let result = comparable_value(&filters, "my-algo");
        assert!(result.is_ok());
        matches!(result.unwrap(), Comparable::Text(s) if s == "my-algo");
    }

    #[test]
    fn test_comparable_value_invalid_field() {
        let filters = crate::jobs::filters::JobFilters {
            sort_field: "invalid".to_string(),
            sort_direction: "asc".to_string(),
            ..Default::default()
        };
        let result = comparable_value(&filters, "value");
        assert!(result.is_err());
    }

    #[test]
    fn test_sort_expression_created_at() {
        let filters = crate::jobs::filters::JobFilters {
            sort_field: "createdAt".to_string(),
            ..Default::default()
        };
        assert_eq!(sort_expression(&filters), "j.created_at");
    }

    #[test]
    fn test_sort_expression_updated_at() {
        let filters = crate::jobs::filters::JobFilters {
            sort_field: "updatedAt".to_string(),
            ..Default::default()
        };
        assert_eq!(sort_expression(&filters), "j.updated_at");
    }

    #[test]
    fn test_sort_expression_status() {
        let filters = crate::jobs::filters::JobFilters {
            sort_field: "status".to_string(),
            ..Default::default()
        };
        assert_eq!(sort_expression(&filters), "j.status");
    }

    #[test]
    fn test_sort_expression_algorithm_id() {
        let filters = crate::jobs::filters::JobFilters {
            sort_field: "algorithmId".to_string(),
            ..Default::default()
        };
        assert_eq!(sort_expression(&filters), "j.algorithm_id");
    }

    #[test]
    fn test_sort_expression_unknown_defaults() {
        let filters = crate::jobs::filters::JobFilters {
            sort_field: "unknown".to_string(),
            ..Default::default()
        };
        assert_eq!(sort_expression(&filters), "j.created_at");
    }

    #[test]
    fn test_push_bound_text() {
        use sqlx::QueryBuilder;
        use sqlx::Sqlite;
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("");
        let value = Comparable::Text("test".to_string());
        push_bound(&mut builder, &value);
        // Just verify it doesn't panic
    }

    #[test]
    fn test_push_bound_numeric() {
        use sqlx::QueryBuilder;
        use sqlx::Sqlite;
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("");
        let value = Comparable::Numeric(42);
        push_bound(&mut builder, &value);
    }

    #[test]
    fn test_push_bound_timestamp() {
        use sqlx::QueryBuilder;
        use sqlx::Sqlite;
        use chrono::{DateTime, Utc};
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("");
        let value = Comparable::Timestamp(DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z").unwrap().with_timezone(&Utc));
        push_bound(&mut builder, &value);
    }

    #[test]
    fn test_push_in_empty() {
        use sqlx::QueryBuilder;
        use sqlx::Sqlite;
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("");
        push_in(&mut builder, "test_col", &[]);
        // Should not add anything for empty slice
    }

    #[test]
    fn test_push_in_with_values() {
        use sqlx::QueryBuilder;
        use sqlx::Sqlite;
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("SELECT * FROM test WHERE ");
        push_in(&mut builder, "status", &["a".to_string(), "b".to_string()]);
        // Just verify it doesn't panic
    }

    #[test]
    fn test_job_key_deterministic() {
        let features = vec!["f1".to_string(), "f2".to_string()];
        let hyper = serde_json::json!({"param": 1});
        let key1 = job_key("ds1", "classification", "algo1", &features, Some("label"), &hyper, 0.8, 42);
        let key2 = job_key("ds1", "classification", "algo1", &features, Some("label"), &hyper, 0.8, 42);
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_job_key_different_inputs() {
        let features = vec!["f1".to_string()];
        let hyper = serde_json::json!({"param": 1});
        let key1 = job_key("ds1", "classification", "algo1", &features, Some("label"), &hyper, 0.8, 42);
        let key2 = job_key("ds2", "classification", "algo1", &features, Some("label"), &hyper, 0.8, 42);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_job_key_sorted_features() {
        let hyper = serde_json::json!({"param": 1});
        let key1 = job_key("ds1", "classification", "algo1", &["b".to_string(), "a".to_string()], None, &hyper, 0.8, 42);
        let key2 = job_key("ds1", "classification", "algo1", &["a".to_string(), "b".to_string()], None, &hyper, 0.8, 42);
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_job_key_sorted_hyperparameters() {
        let hyper1 = serde_json::json!({"b": 2, "a": 1});
        let hyper2 = serde_json::json!({"a": 1, "b": 2});
        let features = vec!["f1".to_string()];
        let key1 = job_key("ds1", "classification", "algo1", &features, None, &hyper1, 0.8, 42);
        let key2 = job_key("ds1", "classification", "algo1", &features, None, &hyper2, 0.8, 42);
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_max_query_length() {
        assert_eq!(MAX_QUERY_LENGTH, 200);
    }

    #[test]
    fn test_default_max_attempts() {
        assert_eq!(DEFAULT_MAX_ATTEMPTS, 2);
    }

    #[test]
    fn test_active_statuses() {
        assert_eq!(ACTIVE_STATUSES.len(), 4);
        assert!(ACTIVE_STATUSES.contains(&"validating"));
        assert!(ACTIVE_STATUSES.contains(&"training"));
        assert!(ACTIVE_STATUSES.contains(&"evaluating"));
        assert!(ACTIVE_STATUSES.contains(&"exporting"));
    }

    #[test]
    fn test_sort_fields() {
        assert_eq!(SORT_FIELDS.len(), 4);
        assert_eq!(SORT_FIELDS[0], ("createdAt", "desc"));
        assert_eq!(SORT_FIELDS[1], ("updatedAt", "desc"));
        assert_eq!(SORT_FIELDS[2], ("status", "asc"));
        assert_eq!(SORT_FIELDS[3], ("algorithmId", "asc"));
    }

    #[test]
    fn test_sort_fields_used_in_parse_sort() {
        let test_fields = [
            ("createdAt", "desc"),
            ("updatedAt", "desc"),
            ("status", "asc"),
            ("algorithmId", "asc"),
        ];
        for (field, expected_dir) in test_fields {
            let result = parse_sort(Some(field)).unwrap();
            assert_eq!(result.0, field);
            assert_eq!(result.1, expected_dir);
        }
    }

    #[test]
    fn test_parse_sort_case_insensitive_direction() {
        let result = parse_sort(Some("STATUS:ASC")).unwrap();
        assert_eq!(result.0, "status");
        assert_eq!(result.1, "asc");

        let result = parse_sort(Some("STATUS:DESC")).unwrap();
        assert_eq!(result.0, "status");
        assert_eq!(result.1, "desc");

        let result = parse_sort(Some("createdat:DESC")).unwrap();
        assert_eq!(result.0, "createdAt");
        assert_eq!(result.1, "desc");
    }

    #[test]
    fn test_push_in_with_values_2() {
        use sqlx::QueryBuilder;
        use sqlx::Sqlite;
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("SELECT * FROM test WHERE ");
        push_in(&mut builder, "status", &["a".to_string(), "b".to_string()]);
        // Just verify it doesn't panic
    }
}
