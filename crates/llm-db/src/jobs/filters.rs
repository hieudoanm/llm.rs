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

pub fn parse_filters(
    statuses: Option<&str>,
    task_types: Option<&str>,
    algorithm_ids: Option<&str>,
    tags: Option<&str>,
    q: Option<&str>,
    created_from: Option<&str>,
    created_to: Option<&str>,
    sort: Option<&str>,
) -> LlmResult<JobFilters> {
    let (sort_field, sort_direction) = parse_sort(sort)?;
    let q_trimmed = q.and_then(|value| {
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
        statuses: split(statuses),
        task_types: split(task_types),
        algorithm_ids: split(algorithm_ids),
        tags: split(tags),
        q: q_trimmed,
        created_from: parse_query_timestamp("createdFrom", created_from)?,
        created_to: parse_query_timestamp("createdTo", created_to)?,
        sort_field,
        sort_direction,
    })
}
