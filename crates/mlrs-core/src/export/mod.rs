//! Artefact exporters: real ONNX protobuf and Rust-native model snapshots.
//!
//! The Python service used `pickle`/`joblib` for `.pkl` and `.joblib`. No Python
//! runtime exists here, so those formats carry a deterministic JSON snapshot of
//! the fitted [`TrainedModel`]. The bytes are stable and can be reloaded with
//! [`load_snapshot`], which keeps the artefact contract (a downloadable file
//! per format) without pretending to be a Python pickle.

pub mod onnx;

use crate::error::{MlrsError, MlrsResult};
use crate::training::model::TrainedModel;

/// Serialize a fitted model as a JSON snapshot for `.pkl`/`.joblib`.
pub fn export_snapshot(trained: &TrainedModel) -> MlrsResult<Vec<u8>> {
    serde_json::to_vec_pretty(trained).map_err(|err| {
        MlrsError::new(
            500,
            "EXPORT_FAILED",
            format!("failed to serialize model snapshot: {err}"),
        )
    })
}

/// Restore a model snapshot produced by [`export_snapshot`].
pub fn load_snapshot(bytes: &[u8]) -> MlrsResult<TrainedModel> {
    serde_json::from_slice(bytes).map_err(|err| {
        MlrsError::new(
            400,
            "INVALID_ARTEFACT",
            format!("failed to parse model snapshot: {err}"),
        )
    })
}
