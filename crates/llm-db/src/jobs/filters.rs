//! Job filtering and cursor types.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use llm_core::error::LlmResult;

use super::helpers::{parse_query_timestamp, parse_sort, split, invalid};

#[derive(Debug, Clone)]
pub struct JobFilters {
    pub statuses: Vec<String>,
    pub task_types: Vec<String>,
    pub algorithm_ids: Vec<String>,
    pub tags: Vec<String>,
    pub q: Option<String>,
    pub created_from: Option<DateTime<Utc>>,
    pub created_to: Option<DateTime<Utc>>,
    pub sort_field: String,
    pub sort_direction: String,
}

impl Default for JobFilters {
    fn default() -> Self {
        Self {
            statuses: Vec::new(),
            task_types: Vec::new(),
            algorithm_ids: Vec::new(),
            tags: Vec::new(),
            q: None,
            created_from: None,
            created_to: None,
            sort_field: "createdAt".to_string(),
            sort_direction: "desc".to_string(),
        }
    }
}

impl JobFilters {
    pub fn ascending(&self) -> bool {
        self.sort_direction == "asc"
    }

    pub fn sort_key(&self) -> String {
        format!("{}:{}", self.sort_field, self.sort_direction)
    }
}

/// An opaque pagination position: the sort value of the last row plus its id.
#[derive(Debug, Clone)]
pub struct JobCursor {
    pub value: String,
    pub id: Uuid,
}

#[derive(Debug, Clone)]
pub struct WorkerCounts {
    pub validating: i64,
    pub training: i64,
    pub evaluating: i64,
    pub exporting: i64,
}

#[derive(Default)]
pub struct ParseFiltersInput<'a> {
    pub statuses: Option<&'a str>,
    pub task_types: Option<&'a str>,
    pub algorithm_ids: Option<&'a str>,
    pub tags: Option<&'a str>,
    pub q: Option<&'a str>,
    pub created_from: Option<&'a str>,
    pub created_to: Option<&'a str>,
    pub sort: Option<&'a str>,
}

pub fn parse_filters(input: ParseFiltersInput) -> LlmResult<JobFilters> {
    let (sort_field, sort_direction) = parse_sort(input.sort)?;
    let q_trimmed = input.q.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            let truncated: String = trimmed.chars().take(super::helpers::MAX_QUERY_LENGTH).collect();
            Some(truncated)
        }
    });
    if let Some(query) = &q_trimmed {
        if query.chars().any(|ch| ch.is_control()) {
            return Err(invalid("q", "Invalid query"));
        }
    }
    Ok(JobFilters {
        statuses: split(input.statuses),
        task_types: split(input.task_types),
        algorithm_ids: split(input.algorithm_ids),
        tags: split(input.tags),
        q: q_trimmed,
        created_from: parse_query_timestamp("createdFrom", input.created_from)?,
        created_to: parse_query_timestamp("createdTo", input.created_to)?,
        sort_field,
        sort_direction,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::helpers::SORT_FIELDS;

    #[test]
    fn test_job_filters_ascending() {
        let mut filters = JobFilters::default();
        assert!(!filters.ascending());

        filters.sort_direction = "asc".to_string();
        assert!(filters.ascending());

        filters.sort_direction = "desc".to_string();
        assert!(!filters.ascending());
    }

    #[test]
    fn test_job_filters_sort_key() {
        let mut filters = JobFilters::default();
        assert_eq!(filters.sort_key(), "createdAt:desc");

        filters.sort_field = "updatedAt".to_string();
        filters.sort_direction = "asc".to_string();
        assert_eq!(filters.sort_key(), "updatedAt:asc");

        filters.sort_field = "status".to_string();
        filters.sort_direction = "desc".to_string();
        assert_eq!(filters.sort_key(), "status:desc");

        filters.sort_field = "algorithmId".to_string();
        filters.sort_direction = "asc".to_string();
        assert_eq!(filters.sort_key(), "algorithmId:asc");
    }

    #[test]
    fn test_job_cursor_creation() {
        let id = uuid::Uuid::new_v4();
        let cursor = JobCursor { value: "2023-01-01T00:00:00Z".to_string(), id };
        assert_eq!(cursor.value, "2023-01-01T00:00:00Z");
        assert_eq!(cursor.id, id);
    }

    #[test]
    fn test_worker_counts() {
        let counts = WorkerCounts {
            validating: 1,
            training: 2,
            evaluating: 3,
            exporting: 4,
        };
        assert_eq!(counts.validating, 1);
        assert_eq!(counts.training, 2);
        assert_eq!(counts.evaluating, 3);
        assert_eq!(counts.exporting, 4);
    }

    #[test]
    fn test_parse_filters_basic() {
        let filters = parse_filters(ParseFiltersInput::default()).unwrap();
        assert!(filters.statuses.is_empty());
        assert!(filters.task_types.is_empty());
        assert!(filters.algorithm_ids.is_empty());
        assert!(filters.tags.is_empty());
        assert!(filters.q.is_none());
        assert!(filters.created_from.is_none());
        assert!(filters.created_to.is_none());
        assert_eq!(filters.sort_field, "createdAt");
        assert_eq!(filters.sort_direction, "desc");
    }

    #[test]
    fn test_parse_filters_with_query() {
        let filters = parse_filters(ParseFiltersInput {
            q: Some("test query"),
            ..Default::default()
        }).unwrap();
        assert_eq!(filters.q, Some("test query".to_string()));
    }

    #[test]
    fn test_parse_filters_with_query_trimmed() {
        let filters = parse_filters(ParseFiltersInput {
            q: Some("  test query  "),
            ..Default::default()
        }).unwrap();
        assert_eq!(filters.q, Some("test query".to_string()));
    }

    #[test]
    fn test_parse_filters_query_length_limit() {
        let long_query: String = "a".repeat(300);
        let filters = parse_filters(ParseFiltersInput {
            q: Some(&long_query),
            ..Default::default()
        }).unwrap();
        assert_eq!(filters.q.as_ref().map(|s| s.len()), Some(200));
    }

    #[test]
    fn test_parse_filters_query_control_chars() {
        let filters = parse_filters(ParseFiltersInput {
            q: Some("query\x01control"),
            ..Default::default()
        });
        assert!(filters.is_err());
    }

    #[test]
    fn test_parse_filters_with_statuses() {
        let filters = parse_filters(ParseFiltersInput {
            statuses: Some("validating,training"),
            ..Default::default()
        }).unwrap();
        assert_eq!(filters.statuses.len(), 2);
        assert_eq!(filters.statuses[0], "validating");
        assert_eq!(filters.statuses[1], "training");
    }

    #[test]
    fn test_parse_filters_with_statuses_empty() {
        let filters = parse_filters(ParseFiltersInput {
            statuses: Some("  "),
            ..Default::default()
        }).unwrap();
        assert!(filters.statuses.is_empty());
    }

    #[test]
    fn test_parse_filters_with_task_types() {
        let filters = parse_filters(ParseFiltersInput {
            task_types: Some("classification,regression"),
            ..Default::default()
        }).unwrap();
        assert_eq!(filters.task_types.len(), 2);
        assert_eq!(filters.task_types[0], "classification");
        assert_eq!(filters.task_types[1], "regression");
    }

    #[test]
    fn test_parse_filters_with_algorithm_ids() {
        let filters = parse_filters(ParseFiltersInput {
            algorithm_ids: Some("algo1,algo2"),
            ..Default::default()
        }).unwrap();
        assert_eq!(filters.algorithm_ids.len(), 2);
        assert_eq!(filters.algorithm_ids[0], "algo1");
        assert_eq!(filters.algorithm_ids[1], "algo2");
    }

    #[test]
    fn test_parse_filters_with_tags() {
        let filters = parse_filters(ParseFiltersInput {
            tags: Some("tag1,tag2"),
            ..Default::default()
        }).unwrap();
        assert_eq!(filters.tags.len(), 2);
        assert_eq!(filters.tags[0], "tag1");
        assert_eq!(filters.tags[1], "tag2");
    }

    #[test]
    fn test_parse_filters_with_created_from() {
        let filters = parse_filters(ParseFiltersInput {
            created_from: Some("2023-01-01T00:00:00Z"),
            ..Default::default()
        }).unwrap();
        assert!(filters.created_from.is_some());
    }

    #[test]
    fn test_parse_filters_with_created_to() {
        let filters = parse_filters(ParseFiltersInput {
            created_to: Some("2023-12-31T23:59:59Z"),
            ..Default::default()
        }).unwrap();
        assert!(filters.created_to.is_some());
    }

    #[test]
    fn test_parse_filters_with_sort_asc() {
        let filters = parse_filters(ParseFiltersInput {
            sort: Some("createdAt:asc"),
            ..Default::default()
        }).unwrap();
        assert_eq!(filters.sort_field, "createdAt");
        assert_eq!(filters.sort_direction, "asc");
    }

    #[test]
    fn test_parse_filters_with_sort_desc() {
        let filters = parse_filters(ParseFiltersInput {
            sort: Some("status:desc"),
            ..Default::default()
        }).unwrap();
        assert_eq!(filters.sort_field, "status");
        assert_eq!(filters.sort_direction, "desc");
    }

    #[test]
    fn test_parse_filters_with_invalid_sort() {
        let filters = parse_filters(ParseFiltersInput {
            sort: Some("invalid:field"),
            ..Default::default()
        });
        assert!(filters.is_err());
    }

    #[test]
    fn test_parse_filters_with_invalid_sort_direction() {
        let filters = parse_filters(ParseFiltersInput {
            sort: Some("createdAt:invalid"),
            ..Default::default()
        });
        assert!(filters.is_err());
    }

    #[test]
    fn test_split_empty_string() {
        let mut filters = JobFilters::default();
        filters.statuses = split(Some(""));
        assert!(filters.statuses.is_empty());
    }

    #[test]
    fn test_split_whitespace() {
        let mut filters = JobFilters::default();
        filters.statuses = split(Some("  "));
        assert!(filters.statuses.is_empty());
    }

    #[test]
    fn test_split_comma_separated() {
        let mut filters = JobFilters::default();
        filters.statuses = split(Some("validating,training,exporting"));
        assert_eq!(filters.statuses.len(), 3);
        assert_eq!(filters.statuses[0], "validating");
        assert_eq!(filters.statuses[1], "training");
        assert_eq!(filters.statuses[2], "exporting");
    }

    #[test]
    fn test_split_with_spaces() {
        let mut filters = JobFilters::default();
        filters.statuses = split(Some("validating,  training , exporting"));
        assert_eq!(filters.statuses.len(), 3);
        assert_eq!(filters.statuses[0], "validating");
        assert_eq!(filters.statuses[1], "training");
        assert_eq!(filters.statuses[2], "exporting");
    }

    #[test]
    fn test_job_cursor_cloning() {
        let id = uuid::Uuid::new_v4();
        let original = JobCursor { value: "2023-01-01T00:00:00Z".to_string(), id };
        let cloned = original.clone();
        assert_eq!(original.value, cloned.value);
        assert_eq!(original.id, cloned.id);
    }

    #[test]
    fn test_worker_counts_default_zero() {
        let counts = WorkerCounts {
            validating: 0,
            training: 0,
            evaluating: 0,
            exporting: 0,
        };
        assert_eq!(counts.validating, 0);
        assert_eq!(counts.training, 0);
        assert_eq!(counts.evaluating, 0);
        assert_eq!(counts.exporting, 0);
    }

    #[test]
    fn parse_query_timestamp_basic_and_empty() {
        assert!(parse_query_timestamp("createdAt", Some("2025-06-15T12:30:00Z")).is_ok());
        assert!(parse_query_timestamp("createdAt", None).unwrap().is_none());
        assert!(parse_query_timestamp("createdAt", Some("")).unwrap().is_none());
    }

    #[test]
    fn parse_sort_all_fields() {
        for (field, dir) in [
            ("createdAt", "desc"),
            ("updatedAt", "desc"),
            ("status", "asc"),
            ("algorithmId", "asc"),
        ] {
            assert_eq!(parse_sort(Some(format!("{field}:{dir}").as_str())).unwrap().0, field);
        }
    }

    #[test]
    fn parse_sort_no_direction() {
        assert_eq!(
            parse_sort(Some("status")).unwrap(),
            ("status".to_string(), SORT_FIELDS[2].1.to_string())
        );
    }

    #[test]
    fn parse_sort_case_insensitive() {
        assert_eq!(
            parse_sort(Some("STATUS:ASC")).unwrap(),
            ("status".to_string(), "asc".to_string())
        );
        assert_eq!(
            parse_sort(Some("createdat:DESC")).unwrap(),
            ("createdAt".to_string(), "desc".to_string())
        );
    }

    #[test]
    fn sort_expression_mapping() {
        use crate::jobs::helpers::sort_expression;
        assert_eq!(sort_expression(&JobFilters {
            statuses: Vec::new(),
            task_types: Vec::new(),
            algorithm_ids: Vec::new(),
            tags: Vec::new(),
            q: None,
            created_from: None,
            created_to: None,
            sort_field: "createdAt".to_string(),
            sort_direction: "desc".to_string(),
        }), "j.created_at");
        assert_eq!(
            sort_expression(&JobFilters {
                statuses: Vec::new(),
                task_types: Vec::new(),
                algorithm_ids: Vec::new(),
                tags: Vec::new(),
                q: None,
                created_from: None,
                created_to: None,
                sort_field: "algorithmId".to_string(),
                sort_direction: "asc".to_string(),
            }),
            "j.algorithm_id"
        );
    }
}
