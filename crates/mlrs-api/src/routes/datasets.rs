//! Dataset upload, metadata, and insights endpoints.

use std::collections::HashSet;

use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::error::{ApiError, ApiResult};
use crate::schemas;
use crate::state::AppState;
use mlrs_core::csv;
use mlrs_core::dataframe::{self, DataFrame};
use mlrs_core::domain::{self as domain, ColumnPreview, DatasetInsight};
use mlrs_core::insights;
use mlrs_db::datasets;

pub async fn root() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "service": "mlrs-backend",
            "version": "0.1.0",
        })),
    )
}

/// Upload a CSV dataset (multipart field name: `file`).
pub async fn upload_dataset(
    state: State<AppState>,
    mut multipart: Multipart,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    let settings = state.settings.clone();

    let field = multipart
        .next_field()
        .await
        .map_err(|err| ApiError::new(400, "MALFORMED_CSV", err.to_string()))?
        .ok_or_else(|| ApiError::new(400, "VALIDATION_ERROR", "Missing file field: file"))?;
    if field.name().as_deref() != Some("file") {
        return Err(ApiError::new(400, "VALIDATION_ERROR", "Missing file field: file"));
    }
    let filename = csv::validate_extension(field.file_name().as_deref()).map_err(ApiError::from)?;
    let content_type = field.content_type().map(str::trim);
    csv::validate_content_type(content_type).map_err(ApiError::from)?;

    let bytes = field
        .bytes()
        .await
        .map_err(|err| ApiError::new(400, "MALFORMED_CSV", err.to_string()))?;

    if bytes.len() > settings.max_upload_bytes() as usize {
        return Err(ApiError::new(
            413,
            "UPLOAD_TOO_LARGE",
            format!(
                "File exceeds {} MB",
                settings.max_upload_bytes() / (1024 * 1024)
            ),
        ));
    }
    if bytes.is_empty() {
        return Err(ApiError::new(400, "VALIDATION_ERROR", "Empty file uploaded"));
    }

    let binary = csv::sniff_binary(&bytes);
    if binary.is_some() {
        return Err(
            ApiError::new(
                400,
                "UNSUPPORTED_FILE_TYPE",
                format!("File appears to be {}, not a CSV", binary.unwrap()),
            )
            .with_details(serde_json::json!({ "filename": filename })),
        );
    }

    let text = csv::decode_csv(&bytes).map_err(ApiError::from)?;
    let (_header, row_count) = csv::parse_csv(&text).map_err(ApiError::from)?;

    let (dataset_id, storage_key, sha256, byte_size) =
        mlrs_core::storage::write_dataset(&bytes).map_err(ApiError::from)?;
    let frame = DataFrame::from_csv_str(&text).map_err(ApiError::from)?;
    let profile = insights::profile_dataframe(&frame);

    let columns = frame
        .column_names()
        .iter()
        .map(|column| {
            let values = frame.column(column).unwrap_or_default();
            let missing = values.iter().filter(|value| value.is_none()).count() as i64;
            let present: Vec<&str> = values
                .iter()
                .flatten()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .collect();
            let inferred = infer_dtype(&present);
            let sample_values = present
                .iter()
                .take(5)
                .map(|value| match dataframe::parse_number(value) {
                    Some(number) => serde_json::json!(number),
                    None => serde_json::json!(value),
                })
                .collect::<Vec<_>>();
            serde_json::json!({
                "name": column,
                "inferredDtype": inferred,
                "nullCount": missing,
                "sampleValues": sample_values,
            })
        })
        .collect::<Vec<_>>();

    let columns: Vec<ColumnPreview> = serde_json::from_value(serde_json::Value::Array(columns))
        .map_err(|error| ApiError::new(500, "INTERNAL_ERROR", error.to_string()))?;

    let dataset = domain::Dataset {
        id: dataset_id,
        original_filename: filename,
        sha256,
        byte_size,
        storage_key,
        columns,
        row_count: row_count as i64,
        created_at: domain::utc_now(),
    };

    datasets::insert_dataset(state.db.pool(), &dataset)
        .await
        .map_err(ApiError::from)?;

    let insight = DatasetInsight {
        dataset_id,
        row_count: profile.row_count as i64,
        column_count: profile.column_count as i64,
        total_missing: profile.total_missing as i64,
        columns: serde_json::to_value(&profile.columns)
            .map_err(|error| ApiError::new(500, "INTERNAL_ERROR", error.to_string()))?,
        warnings: serde_json::to_value(&profile.warnings)
            .map_err(|error| ApiError::new(500, "INTERNAL_ERROR", error.to_string()))?,
        created_at: domain::utc_now(),
    };
    datasets::insert_insight(state.db.pool(), &insight)
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(schemas::dataset_value(&dataset))))
}

pub async fn get_dataset(
    state: State<AppState>,
    axum::extract::Path(dataset_id): axum::extract::Path<uuid::Uuid>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    let dataset = datasets::get_dataset(state.db.pool(), dataset_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::not_found("DATASET_NOT_FOUND", "Dataset not found"))?;

    Ok((StatusCode::OK, Json(schemas::dataset_value(&dataset))))
}

pub async fn get_dataset_insights(
    state: State<AppState>,
    axum::extract::Path(dataset_id): axum::extract::Path<uuid::Uuid>,
    axum::extract::Query(query): axum::extract::Query<InsightsQuery>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    if query.preview_limit < 1 || query.preview_limit > 100 {
        return Err(
            ApiError::validation("previewLimit must be between 1 and 100").with_details(
                serde_json::json!({ "field": "previewLimit" }),
            ),
        );
    }
    if query.preview_offset < 0 {
        return Err(ApiError::validation("previewOffset must not be negative").with_details(
            serde_json::json!({ "field": "previewOffset" }),
        ));
    }

    let dataset = datasets::get_dataset(state.db.pool(), dataset_id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::not_found("DATASET_NOT_FOUND", "Dataset not found"))?;

    let insight = match datasets::get_insight(state.db.pool(), dataset_id).await {
        Ok(insight) => insight,
        Err(error) => return Err(ApiError::from(error)),
    };
    let Some(insight) = insight else {
        return Err(ApiError::not_found(
            "DATASET_NOT_FOUND",
            "Dataset insights were not found",
        ));
    };

    let bytes = mlrs_core::storage::read_dataset(&dataset.storage_key)
        .map_err(ApiError::from)?;
    let text = String::from_utf8(bytes).map_err(|error| {
        ApiError::new(400, "MALFORMED_CSV", format!("File is not valid UTF-8: {error}"))
    })?;
    let frame = DataFrame::from_csv_str(&text).map_err(ApiError::from)?;
    let rows = insights::preview_rows(
        &frame,
        query.preview_offset as usize,
        query.preview_limit as usize,
    );
    let cell_count = (insight.row_count.max(0) as f64) * (insight.column_count.max(0) as f64);
    let missing_rate = if cell_count > 0.0 {
        insight.total_missing as f64 / cell_count
    } else {
        0.0
    };

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "datasetId": dataset.id,
            "sha256": dataset.sha256,
            "filename": dataset.original_filename,
            "rowCount": insight.row_count,
            "columnCount": insight.column_count,
            "totalMissingCells": insight.total_missing,
            "missingRate": missing_rate,
            "columns": insight.columns,
            "warnings": insight.warnings,
            "preview": {
                "offset": query.preview_offset,
                "limit": query.preview_limit,
                "totalRows": insight.row_count,
                "columns": frame.column_names(),
                "rows": rows,
            },
        })),
    ))
}

#[derive(Debug, serde::Deserialize, Default)]
pub struct InsightsQuery {
    #[serde(default = "default_preview_limit", rename = "previewLimit")]
    pub preview_limit: i64,
    #[serde(default, rename = "previewOffset")]
    pub preview_offset: i64,
}

fn default_preview_limit() -> i64 {
    10
}

fn infer_dtype(values: &[&str]) -> String {
    if values.is_empty() {
        return "categorical".to_string();
    }
    if values.iter().all(|value| dataframe::parse_number(value).is_some()) {
        if values.iter().all(|value| value.trim().parse::<i64>().is_ok()) {
            return "int64".to_string();
        }
        return "float64".to_string();
    }
    let lowered: HashSet<String> = values
        .iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .collect();
    if lowered.len() <= 50 {
        "categorical".to_string()
    } else {
        "text".to_string()
    }
}
