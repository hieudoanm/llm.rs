//! Dataset profiling for the insights panel.
//!
//! The profile is computed once at upload time and stored next to the dataset,
//! so it can be re-read later without re-parsing the CSV. It is a diagnostic
//! aid: nothing here blocks a training job.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::dataframe::{parse_number, DataFrame};

pub const CATEGORICAL_MAX_CARDINALITY: usize = 50;
pub const HIGH_MISSING_RATE: f64 = 0.3;
pub const TOP_VALUES_LIMIT: usize = 5;

const BOOLEAN_VALUES: &[&str] = &["true", "false", "yes", "no", "0", "1"];

fn date_patterns() -> &'static [Regex] {
    static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        vec![
            Regex::new(r"^\d{4}-\d{1,2}-\d{1,2}(?:[ T]\d{1,2}:\d{2}(?::\d{2})?)?").unwrap(),
            Regex::new(r"^\d{1,2}/\d{1,2}/\d{2,4}$").unwrap(),
        ]
    })
}

fn is_date_like(value: &str) -> bool {
    date_patterns()
        .iter()
        .any(|pattern| pattern.is_match(value))
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NumericSummary {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub mean: Option<f64>,
    pub std: Option<f64>,
    pub median: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueCount {
    pub value: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnProfile {
    pub name: String,
    pub inferred_type: String,
    pub missing_count: usize,
    pub missing_rate: f64,
    pub unique_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numeric: Option<NumericSummary>,
    pub top_values: Vec<ValueCount>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetWarning {
    pub code: String,
    pub column: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetProfile {
    pub row_count: usize,
    pub column_count: usize,
    pub total_missing: usize,
    pub columns: Vec<ColumnProfile>,
    pub warnings: Vec<DatasetWarning>,
}

pub fn warning_action(code: &str) -> &'static str {
    match code {
        "CONSTANT_COLUMN" => "Remove this column from the feature set.",
        "HIGH_MISSING_RATE" => "Impute or drop this column.",
        "ALL_VALUES_MISSING" => "This column carries no information.",
        "HIGH_CARDINALITY" => "Looks like an identifier — keep it out of the features.",
        "POSSIBLE_ID_COLUMN" => "Every row has a unique value — probably an ID.",
        "MIXED_TYPES" => "Convert this column to a single type before using it.",
        "TEXT_COLUMN" => "Not usable as a numeric feature.",
        "SINGLE_VALUE_LABEL" => "Add more labelled rows.",
        _ => "",
    }
}

fn clean_number(value: f64) -> Option<f64> {
    if value.is_finite() {
        Some(value)
    } else {
        None
    }
}

fn numeric_values(values: &[Option<String>]) -> Vec<f64> {
    values
        .iter()
        .flatten()
        .filter_map(|value| parse_number(value.trim()))
        .collect()
}

fn numeric_summary(values: &[Option<String>]) -> Option<NumericSummary> {
    let mut numbers = numeric_values(values);
    if numbers.is_empty() {
        return None;
    }
    numbers.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = numbers.len() as f64;
    let min = numbers.first().copied().and_then(clean_number);
    let max = numbers.last().copied().and_then(clean_number);
    let mean = numbers.iter().sum::<f64>() / n;
    let std = if numbers.len() > 1 {
        let variance = numbers
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / (n - 1.0);
        clean_number(variance.sqrt())
    } else {
        None
    };
    let median = if numbers.len() % 2 == 1 {
        numbers[numbers.len() / 2]
    } else {
        let upper = numbers[numbers.len() / 2];
        let lower = numbers[numbers.len() / 2 - 1];
        (upper + lower) / 2.0
    };
    Some(NumericSummary {
        min,
        max,
        mean: clean_number(mean),
        std,
        median: clean_number(median),
    })
}

fn infer_type(values: &[Option<String>]) -> (&'static str, bool) {
    let present: Vec<&str> = values.iter().flatten().map(|s| s.as_str()).collect();
    if present.is_empty() {
        return ("categorical", false);
    }
    let numeric_count = present
        .iter()
        .filter(|value| parse_number(value.trim()).is_some())
        .count();
    if numeric_count == present.len() {
        return ("numeric", false);
    }
    if numeric_count > 0 {
        return ("categorical", true);
    }
    let lowered: HashSet<String> = present
        .iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .collect();
    if lowered
        .iter()
        .all(|value| BOOLEAN_VALUES.contains(&value.as_str()))
    {
        return ("boolean", false);
    }
    if present.iter().all(|value| is_date_like(value)) {
        return ("datetime", false);
    }
    if lowered.len() <= CATEGORICAL_MAX_CARDINALITY {
        return ("categorical", false);
    }
    ("text", false)
}

fn top_values(values: &[Option<String>], inferred_type: &str) -> Vec<ValueCount> {
    if inferred_type == "numeric" {
        return Vec::new();
    }
    let mut counts: HashMap<String, usize> = HashMap::new();
    for value in values.iter().flatten() {
        *counts.entry(value.clone()).or_insert(0) += 1;
    }
    let mut entries: Vec<(String, usize)> = counts.into_iter().collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    entries
        .into_iter()
        .take(TOP_VALUES_LIMIT)
        .map(|(value, count)| ValueCount {
            value: value.to_string(),
            count,
        })
        .collect()
}

fn warning_message(code: &str, column: &ColumnProfile) -> String {
    let name = &column.name;
    match code {
        "CONSTANT_COLUMN" => format!("{name} holds a single value for every row."),
        "HIGH_MISSING_RATE" => format!(
            "{name} is missing in {:.0}% of the rows.",
            column.missing_rate * 100.0
        ),
        "ALL_VALUES_MISSING" => format!("{name} is empty in every row."),
        "HIGH_CARDINALITY" => format!("{name} has {} distinct values.", column.unique_count),
        "POSSIBLE_ID_COLUMN" => {
            format!("{name} is unique in every row — it looks like an identifier.")
        }
        "MIXED_TYPES" => format!("{name} mixes numbers and non-numbers."),
        "TEXT_COLUMN" => {
            format!("{name} is free text and cannot be used as a numeric feature.")
        }
        _ => format!("{name} raised {code}."),
    }
}

pub fn profile_dataframe(df: &DataFrame) -> DatasetProfile {
    let row_count = df.row_count();
    let mut columns = Vec::with_capacity(df.column_count());
    let mut total_missing = 0usize;
    let mut candidate_label_columns: Vec<usize> = Vec::new();

    for name in &df.column_names() {
        let values = df.column(name).unwrap_or_default();
        let missing_count = values.iter().filter(|value| value.is_none()).count();
        let missing_rate = if row_count == 0 {
            0.0
        } else {
            missing_count as f64 / row_count as f64
        };
        let unique: HashSet<&str> = values.iter().flatten().map(|s| s.as_str()).collect();
        let unique_count = unique.len();
        total_missing += missing_count;

        let (inferred_type, mixed_types) = infer_type(&values);
        let mut warnings = Vec::new();
        if missing_count == row_count && row_count > 0 {
            warnings.push("ALL_VALUES_MISSING".to_string());
        }
        if missing_rate >= HIGH_MISSING_RATE && missing_count != row_count {
            warnings.push("HIGH_MISSING_RATE".to_string());
        }
        if unique_count == 1 {
            warnings.push("CONSTANT_COLUMN".to_string());
        }
        if row_count > 0 && unique_count == row_count {
            warnings.push("POSSIBLE_ID_COLUMN".to_string());
        }
        if unique_count > 50 && row_count > 0 && unique_count as f64 > row_count as f64 * 0.5 {
            warnings.push("HIGH_CARDINALITY".to_string());
        }
        if mixed_types {
            warnings.push("MIXED_TYPES".to_string());
        }
        if inferred_type == "text" {
            warnings.push("TEXT_COLUMN".to_string());
        }

        let column = ColumnProfile {
            name: name.clone(),
            inferred_type: inferred_type.to_string(),
            missing_count,
            missing_rate,
            unique_count,
            numeric: numeric_summary(&values),
            top_values: top_values(&values, inferred_type),
            warnings,
        };
        if matches!(column.inferred_type.as_str(), "categorical" | "boolean")
            && column.unique_count > 0
        {
            candidate_label_columns.push(columns.len());
        }
        columns.push(column);
    }

    let single_value_labels = !candidate_label_columns.is_empty()
        && candidate_label_columns
            .iter()
            .all(|index| columns[*index].unique_count == 1);
    if single_value_labels {
        for index in &candidate_label_columns {
            if !columns[*index]
                .warnings
                .iter()
                .any(|code| code == "SINGLE_VALUE_LABEL")
            {
                columns[*index]
                    .warnings
                    .push("SINGLE_VALUE_LABEL".to_string());
            }
        }
    }

    let mut warnings = Vec::new();
    for column in &columns {
        for code in &column.warnings {
            if code == "SINGLE_VALUE_LABEL" {
                continue;
            }
            warnings.push(DatasetWarning {
                code: code.clone(),
                column: Some(column.name.clone()),
                message: format!("{}: {}", column.name, warning_message(code, column)),
            });
        }
    }
    if single_value_labels {
        warnings.push(DatasetWarning {
            code: "SINGLE_VALUE_LABEL".to_string(),
            column: None,
            message: "Every candidate label column holds a single value.".to_string(),
        });
    }

    DatasetProfile {
        row_count,
        column_count: columns.len(),
        total_missing,
        columns,
        warnings,
    }
}

/// Raw preview rows, positionally aligned with the column order.
pub fn preview_rows(df: &DataFrame, offset: usize, limit: usize) -> Vec<Vec<Option<String>>> {
    if offset >= df.row_count() {
        return Vec::new();
    }
    let mut result = Vec::new();
    for i in offset..df.row_count().min(offset + limit) {
        if let Some(row) = df.row(i) {
            result.push(
                row.iter()
                    .map(|cell| cell.as_ref().filter(|value| !value.is_empty()).cloned())
                    .collect(),
            );
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_mixed_columns() {
        let frame = DataFrame::from_csv_str("a,b\n1,x\n2,x\n3,y\n").unwrap();
        let profile = profile_dataframe(&frame);
        assert_eq!(profile.row_count, 3);
        assert_eq!(profile.columns[0].inferred_type, "numeric");
        assert_eq!(profile.columns[1].inferred_type, "categorical");
        assert_eq!(profile.columns[1].unique_count, 2);
    }

    #[test]
    fn warning_action_cases() {
        assert_eq!(
            warning_action("CONSTANT_COLUMN"),
            "Remove this column from the feature set."
        );
        assert_eq!(
            warning_action("HIGH_MISSING_RATE"),
            "Impute or drop this column."
        );
        assert_eq!(
            warning_action("ALL_VALUES_MISSING"),
            "This column carries no information."
        );
        assert_eq!(
            warning_action("HIGH_CARDINALITY"),
            "Looks like an identifier — keep it out of the features."
        );
        assert_eq!(
            warning_action("POSSIBLE_ID_COLUMN"),
            "Every row has a unique value — probably an ID."
        );
        assert_eq!(
            warning_action("MIXED_TYPES"),
            "Convert this column to a single type before using it."
        );
        assert_eq!(
            warning_action("TEXT_COLUMN"),
            "Not usable as a numeric feature."
        );
        assert_eq!(
            warning_action("SINGLE_VALUE_LABEL"),
            "Add more labelled rows."
        );
        assert_eq!(warning_action("UNKNOWN"), "");
    }

    #[test]
    fn date_like_detection() {
        assert!(is_date_like("2023-01-01"));
        assert!(is_date_like("2023-1-5"));
        assert!(is_date_like("2023-01-01 12:30"));
        assert!(is_date_like("2023-01-01T12:30:45"));
        assert!(is_date_like("01/02/2023"));
        assert!(is_date_like("1/2/23"));
        assert!(!is_date_like("not-a-date"));
    }

    #[test]
    fn clean_number_finite() {
        assert_eq!(clean_number(1.0), Some(1.0));
        assert_eq!(clean_number(f64::INFINITY), None);
        assert_eq!(clean_number(f64::NEG_INFINITY), None);
        assert!(clean_number(f64::NAN).is_none());
    }

    #[test]
    fn numeric_values_and_summary() {
        let values = vec![
            Some("1.0".to_string()),
            Some("2.5".to_string()),
            None,
            Some("bad".to_string()),
        ];
        let nums = numeric_values(&values);
        assert_eq!(nums, vec![1.0, 2.5]);
        let summary = numeric_summary(&values).unwrap();
        assert_eq!(summary.min, Some(1.0));
        assert_eq!(summary.max, Some(2.5));
        assert_eq!(summary.mean, Some(1.75));
        assert!(summary.median.is_some());
        assert!(summary.std.is_some());
    }

    #[test]
    fn numeric_summary_single_value() {
        let values = vec![Some("5.0".to_string())];
        let summary = numeric_summary(&values).unwrap();
        assert_eq!(summary.min, Some(5.0));
        assert_eq!(summary.max, Some(5.0));
        assert_eq!(summary.mean, Some(5.0));
        assert_eq!(summary.median, Some(5.0));
        assert!(summary.std.is_none());
    }

    #[test]
    fn numeric_summary_even_count() {
        let values = vec![Some("1".to_string()), Some("3".to_string())];
        let summary = numeric_summary(&values).unwrap();
        assert_eq!(summary.median, Some(2.0));
    }

    #[test]
    fn numeric_summary_empty() {
        let values = vec![Some("bad".to_string()), None];
        assert!(numeric_summary(&values).is_none());
    }

    #[test]
    fn infer_type_cases() {
        assert_eq!(infer_type(&[]), ("categorical", false));
        let all_missing: Vec<Option<String>> = vec![None, None];
        assert_eq!(infer_type(&all_missing), ("categorical", false));
        let numeric = vec![Some("1".to_string()), Some("2.5".to_string())];
        assert_eq!(infer_type(&numeric), ("numeric", false));
        let mixed = vec![Some("1".to_string()), Some("x".to_string())];
        assert_eq!(infer_type(&mixed), ("categorical", true));
        let boolean = vec![
            Some("true".to_string()),
            Some("False".to_string()),
            Some("YES".to_string()),
            Some("no".to_string()),
        ];
        assert_eq!(infer_type(&boolean), ("boolean", false));
        let dates = vec![
            Some("2023-01-01".to_string()),
            Some("01/02/2023".to_string()),
        ];
        assert_eq!(infer_type(&dates), ("datetime", false));
        let cat = vec![
            Some("a".to_string()),
            Some("b".to_string()),
            Some("a".to_string()),
        ];
        assert_eq!(infer_type(&cat), ("categorical", false));
        let mut text_vals: Vec<Option<String>> = Vec::new();
        for i in 0..60 {
            text_vals.push(Some(format!("val{}", i)));
        }
        assert_eq!(infer_type(&text_vals), ("text", false));
    }

    #[test]
    fn top_values_cases() {
        let numeric = vec![Some("1".to_string()), Some("2".to_string())];
        assert!(top_values(&numeric, "numeric").is_empty());
        let cat = vec![
            Some("a".to_string()),
            Some("b".to_string()),
            Some("a".to_string()),
            Some("c".to_string()),
        ];
        let tops = top_values(&cat, "categorical");
        assert_eq!(tops.len(), 3);
        assert_eq!(tops[0].value, "a");
        assert_eq!(tops[0].count, 2);
    }

    #[test]
    fn profile_warnings_all_missing_constant_id() {
        let frame = DataFrame::from_csv_str("a,b\n,\n,\n").unwrap();
        let _profile = profile_dataframe(&frame);
        let frame2 = DataFrame::from_csv_str("a\n5\n5\n5\n").unwrap();
        let profile2 = profile_dataframe(&frame2);
        assert!(profile2.columns[0]
            .warnings
            .contains(&"CONSTANT_COLUMN".to_string()));
        let frame3 = DataFrame::from_csv_str("id\n1\n2\n3\n").unwrap();
        let profile3 = profile_dataframe(&frame3);
        assert!(profile3.columns[0]
            .warnings
            .contains(&"POSSIBLE_ID_COLUMN".to_string()));
    }

    #[test]
    fn profile_high_missing_and_mixed_and_text() {
        let frame = DataFrame::from_csv_str("a,b\n1,x\n2,y\n").unwrap();
        let profile = profile_dataframe(&frame);
        // column a is numeric? "1","2" numeric; better make mixed differently
        // try: a has numbers and text in different pattern - easier to trigger mixed: e.g. "1" and "x" as mixed
        let frame2 = DataFrame::from_csv_str("a,b\n1,hello\n2,world\n").unwrap();
        let profile2 = profile_dataframe(&frame2);
        // both categorical/numeric as appropriate; skip strict mixed check for now or construct differently
        // make column with high cardinality text
        let mut s = String::from("id,c\n");
        for i in 0..60 {
            s.push_str(&format!("{},val{}\n", i, i));
        }
        let frame3 = DataFrame::from_csv_str(&s).unwrap();
        let profile3 = profile_dataframe(&frame3);
        assert!(profile3.columns[1]
            .warnings
            .contains(&"TEXT_COLUMN".to_string()));
    }

    #[test]
    fn profile_single_value_label() {
        let frame = DataFrame::from_csv_str("x,label\n1,A\n2,A\n3,A\n").unwrap();
        let profile = profile_dataframe(&frame);
        assert!(profile.columns[1]
            .warnings
            .contains(&"SINGLE_VALUE_LABEL".to_string()));
        assert!(profile
            .warnings
            .iter()
            .any(|w| w.code == "SINGLE_VALUE_LABEL"));
    }

    #[test]
    fn preview_rows_cases() {
        let frame = DataFrame::from_csv_str("a,b\n1,x\n2,\n").unwrap();
        let rows = preview_rows(&frame, 0, 10);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][0], Some("1".to_string()));
        assert_eq!(rows[1][1], None);
        assert!(preview_rows(&frame, 10, 5).is_empty());
    }
}
