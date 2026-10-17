//! Algorithm catalogue endpoint.

use axum::extract::Query;
use axum::http::StatusCode;
use axum::Json;

use crate::error::ApiResult;
use mlrs_core::catalogue;

#[derive(serde::Deserialize, Default)]
pub struct AlgorithmsQuery {
    #[serde(rename = "taskType")]
    pub task_type: Option<String>,
}

pub async fn list_algorithms(
    query: Query<AlgorithmsQuery>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    let task = query.task_type.as_deref().map(str::to_ascii_lowercase);
    let algorithms = catalogue::algorithms()
        .iter()
        .filter(|spec| {
            task.as_ref()
                .map_or(true, |task| task.as_str() == spec.task_type.as_str())
        })
        .map(|spec| {
            serde_json::json!({
                "algorithmId": spec.id,
                "name": spec.name,
                "taskType": spec.task_type.as_str(),
                "family": spec.family,
                "description": spec.description,
                "defaultHyperparameters": spec.default_hyperparameters,
                "exports": spec.exports.iter().map(|format| format.as_str()).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "algorithms": algorithms })),
    ))
}
