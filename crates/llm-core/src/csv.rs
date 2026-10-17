//! Upload validation for dataset CSVs.
//!
//! Three independent checks guard the upload endpoint:
//! 1. the file extension must be `.csv`;
//! 2. the declared content type must be one a browser sends for a CSV;
//! 3. the sniffed bytes must not belong to a known binary format, and the
//!    payload must decode as UTF-8 and parse as delimited text.

use std::collections::HashSet;

use polars::prelude::*;
use serde_json::{json, Value};

use crate::error::{LlmError, LlmResult};

pub const ALLOWED_CONTENT_TYPES: &[&str] = &[
    "",
    "application/csv",
    "application/excel",
    "application/octet-stream",
    "application/vnd.ms-excel",
    "application/x-csv",
    "text/comma-separated-values",
    "text/csv",
    "text/plain",
];

pub const BINARY_SIGNATURES: &[(&[u8], &str)] = &[
    (b"PK\x03\x04", "a ZIP archive"),
    (b"PK\x05\x06", "a ZIP archive"),
    (b"PK\x07\x08", "a ZIP archive"),
    (b"\x1f\x8b", "a gzip stream"),
    (b"\x1f\x9d", "a compressed archive"),
    (b"BZh", "a bzip2 archive"),
    (b"\xfd7zXZ\x00", "an xz archive"),
    (b"Rar!\x1a\x07", "a RAR archive"),
    (b"\x89PNG\r\n\x1a\n", "a PNG image"),
    (b"\xff\xd8\xff", "a JPEG image"),
    (b"GIF87a", "a GIF image"),
    (b"GIF89a", "a GIF image"),
    (b"%PDF-", "a PDF document"),
    (b"\x7fELF", "an ELF binary"),
    (b"\xca\xfe\xba\xbe", "a Java class file"),
    (b"MZ", "a Windows executable"),
];

pub const HTML_PREFIXES: &[&[u8]] = &[
    b"<!doctype html",
    b"<html",
    b"<?xml",
    b"<svg",
    b"<script",
];

pub const UTF16_BOMS: &[&[u8]] = &[b"\xff\xfe", b"\xfe\xff"];
pub const SNIFF_WINDOW: usize = 4096;
pub const MAX_COLUMNS: usize = 512;

pub fn validate_extension(filename: Option<&str>) -> LlmResult<String> {
    let name = filename.unwrap_or("upload.csv");
    if !name.to_ascii_lowercase().ends_with(".csv") {
        return Err(LlmError::new(
            400,
            "UNSUPPORTED_FILE_TYPE",
            "Only .csv uploads are supported",
        )
        .with_details(json!({"filename": name, "expectedExtension": ".csv"})));
    }
    Ok(name.to_string())
}

pub fn validate_content_type(content_type: Option<&str>) -> LlmResult<String> {
    let declared = content_type
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if !ALLOWED_CONTENT_TYPES.contains(&declared.as_str()) {
        let mut allowed = ALLOWED_CONTENT_TYPES.to_vec();
        allowed.sort_unstable();
        let label = if declared.is_empty() {
            "unknown".to_string()
        } else {
            declared.clone()
        };
        return Err(LlmError::new(
            400,
            "UNSUPPORTED_FILE_TYPE",
            format!("Unsupported content type: {label}"),
        )
        .with_details(json!({"contentType": declared, "allowed": allowed})));
    }
    Ok(declared)
}

/// Return a label when the payload looks like a known binary format.
pub fn sniff_binary(data: &[u8]) -> Option<&'static str> {
    for (signature, label) in BINARY_SIGNATURES {
        if data.starts_with(signature) {
            return Some(label);
        }
    }
    for bom in UTF16_BOMS {
        if data.starts_with(bom) {
            return Some("a UTF-16 encoded file");
        }
    }
    let window = &data[..data.len().min(SNIFF_WINDOW)];
    let lowered = window
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .map(|start| &window[start..])
        .unwrap_or(&[]);
    let lowered: Vec<u8> = lowered.iter().map(|b| b.to_ascii_lowercase()).collect();
    for prefix in HTML_PREFIXES {
        if lowered.starts_with(prefix) {
            return Some("an HTML document");
        }
    }
    if window.contains(&0) {
        return Some("a binary file");
    }
    None
}

/// Decode a payload as UTF-8, permitting a single leading BOM.
pub fn decode_csv(data: &[u8]) -> LlmResult<String> {
    let text = std::str::from_utf8(data).map_err(|err| {
        LlmError::new(
            400,
            "MALFORMED_CSV",
            format!("File is not valid UTF-8: {err}"),
        )
    })?;
    Ok(text.strip_prefix('\u{feff}').unwrap_or(text).to_string())
}

/// Parse and validate CSV text, returning the header and data-row count.
pub fn parse_csv(text: &str) -> LlmResult<(Vec<String>, usize)> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);

    // Guard against duplicate column names in the raw header before Polars
    // auto-renames them (e.g. "a,a" -> "a, a_1").
    if let Some(first_line) = text.lines().next() {
        let raw_header: Vec<&str> = first_line.split(',').map(|c| c.trim()).collect();
        let mut seen = HashSet::new();
        for name in &raw_header {
            if !name.is_empty() && !seen.insert(name) {
                return Err(LlmError::validation("Duplicate column name in header")
                    .with_details(json!({"field": "columns"})));
            }
        }
    }

    let df = CsvReader::new(std::io::Cursor::new(text.as_bytes()))
        .has_header(true)
        .finish()
        .map_err(|err| {
            LlmError::new(
                400,
                "MALFORMED_CSV",
                format!("File cannot be parsed as CSV: {err}"),
            )
        })?;

    let header: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|name| name.trim().to_string())
        .collect();

    if header.is_empty() || header.iter().all(|cell| cell.is_empty()) {
        return Err(LlmError::validation("CSV has no header row"));
    }
    if header.len() < 2 {
        return Err(LlmError::validation(
            "CSV must have a header and at least one data column",
        )
        .with_details(json!({"field": "columns", "count": header.len()})));
    }
    if header.len() > MAX_COLUMNS {
        return Err(LlmError::validation(format!(
            "CSV must have at most {MAX_COLUMNS} columns"
        ))
        .with_details(json!({"field": "columns", "limit": MAX_COLUMNS, "count": header.len()})));
    }
    for (index, name) in header.iter().enumerate() {
        if name.is_empty() {
            return Err(LlmError::validation(format!(
                "Column {} has an empty name",
                index + 1
            ))
            .with_details(json!({"field": "columns", "index": index})));
        }
    }

    let data_rows = df.height();

    if data_rows == 0 {
        return Err(LlmError::validation("CSV has a header but no data rows")
            .with_details(json!({"field": "columns"})));
    }

    let mut empty_columns = Vec::new();
    for (_index, name) in header.iter().enumerate() {
        let col = df.column(name);
        if col.is_err() {
            continue;
        }
        let col = col.unwrap();
        let all_empty = col.null_count() == col.len();
        if all_empty {
            empty_columns.push(name.clone());
        }
    }
    if empty_columns.len() >= header.len() {
        return Err(LlmError::validation(
            "Every column is empty; the CSV has no usable data",
        )
        .with_details(json!({"field": "columns", "columns": empty_columns})));
    }
    if !empty_columns.is_empty() {
        return Err(LlmError::validation(format!(
            "Column(s) empty in every row: {}",
            empty_columns.join(", ")
        ))
        .with_details(json!({"field": "columns", "emptyColumns": empty_columns})));
    }

    // Guard against a header that repeats a name only after trimming.
    let unique: HashSet<&String> = header.iter().collect();
    if unique.len() != header.len() {
        return Err(LlmError::validation("Duplicate column name in header")
            .with_details(json!({"field": "columns"})));
    }

    Ok((header, data_rows))
}

pub fn unsupported_binary_error(label: &str, filename: &str) -> LlmError {
    LlmError::new(
        400,
        "UNSUPPORTED_FILE_TYPE",
        format!("File appears to be {label}, not a CSV"),
    )
    .with_details(Value::Object(
        [("filename".to_string(), json!(filename))]
            .into_iter()
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_binary_signatures() {
        assert_eq!(sniff_binary(b"PK\x03\x04rest"), Some("a ZIP archive"));
        assert_eq!(sniff_binary(b"a,b\n1,2\n"), None);
        assert_eq!(sniff_binary(b"<html><body>"), Some("an HTML document"));
    }

    #[test]
    fn validates_header() {
        assert!(parse_csv("a,b\n1,2\n").is_ok());
        assert!(parse_csv("a,a\n1,2\n").is_err());
        assert!(parse_csv("a\n1\n").is_err());
        assert!(parse_csv("a,b\n,\n").is_err());
    }

    #[test]
    fn parse_csv_binary_error() {
        let _ = parse_csv("PK\x03\x04rest");
    }

    #[test]
    fn parse_csv_html_error() {
        let _ = parse_csv("<html>hi</html>");
    }

    #[test]
    fn parse_csv_empty_header() {
        let _ = parse_csv(",\n1,2\n");
    }

    #[test]
    fn parse_csv_missing_header() {
        let _ = parse_csv("\n1,2\n");
    }

    #[test]
    fn parse_csv_too_few_columns() {
        let _ = parse_csv("a,b\n1\n");
    }

    #[test]
    fn parse_csv_malformed() {
        let _ = parse_csv("a,b\n\"unclosed");
    }

    #[test]
    fn unsupported_binary_error_details() {
        let err = unsupported_binary_error("a ZIP archive", "test.zip");
        assert_eq!(err.code, "UNSUPPORTED_FILE_TYPE");
    }
}
