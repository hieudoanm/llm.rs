# FEATURE-05: Model Export

## Priority

P0 ONNX (FR-050, FR-051, FR-055..057, FR-085, FR-095, FR-105); P1 optional pickle/joblib (FR-052, FR-053)

## Description

Export is part of a successful job. **ONNX is required.** Every shipped model — scikit-learn estimators, the configurable MLPs, and the FQI Q-function — exports through a single `mlrs2onnx` path. Sklearn jobs (including MLP jobs) may also write `.pkl` or `.joblib`. Downloads use artefact ids, never filesystem paths.

## User-visible behaviour

- Download button for ONNX (always enabled on `completed`).
- Secondary buttons for pickle / joblib when those artefacts exist.
- **Download metrics.json** button for completed jobs with metrics (FEATURE-12).
- **Chart download dropdowns** (SVG/PNG) on each Model report chart card (FEATURE-12).
- Disabled buttons with tooltip while job is in progress.

## Outputs

| format | Produced when                        | Filename pattern |
| ------ | ------------------------------------ | ---------------- |
| onnx   | Every successful job                 | `{jobId}.onnx`   |
| pkl    | Sklearn job and `exportPkl: true`    | `{jobId}.pkl`    |
| joblib | Sklearn job and `exportJoblib: true` | `{jobId}.joblib` |

## Behaviour

1. After metrics, convert the fitted pipeline (preprocessor + estimator) to ONNX.
2. Set the default ONNX opset to 15 and pin `ai.onnx.ml` to **3** so IsolationForest and other ML-domain operators load in the test runtime.
3. Write files under `{DATA_DIR}/artefacts/{jobId}/`.
4. Insert `artefacts` rows.
5. Only then mark job `completed`.

If ONNX export fails, the job is `failed` even if fit succeeded. Optional formats failing while ONNX succeeded: job remains `completed` and the optional artefact is omitted (log warning).

### Task-specific ONNX contents

| Task              | ONNX graph contains                                       | Inference output                                  |
| ----------------- | --------------------------------------------------------- | ------------------------------------------------- |
| classification    | Fitted preprocessor + fitted classifier                   | Original class labels                             |
| regression        | Fitted preprocessor + fitted regressor                    | Numeric prediction                                |
| semi_supervised   | Fitted preprocessor + `SelfTrainingClassifier.estimator_` | Original class labels; matches the fitted wrapper |
| clustering        | Fitted preprocessor + fitted clusterer                    | `label` (cluster index) and scores/probabilities  |
| anomaly_detection | Fitted preprocessor + fitted IsolationForest              | `label` (`1` inlier, `-1` outlier) and scores     |

For semi-supervised jobs, `pkl`/`joblib` contain the fitted `SelfTrainingClassifier` wrapper for reproducibility; ONNX contains the fitted base estimator because `mlrs2onnx` has no converter for the wrapper. Self-training only changes training labels, so the two produce identical predictions.

## Edge cases

- Partial write: temp file + atomic rename.
- Download of another job's artefact id → `404`.
- An estimator without a verified converter is rejected at catalogue integrity time; it never reaches export.

## Acceptance conditions

- CI loads every P0 ONNX with ONNX Runtime and runs one dummy inference matching mlrs predict shape and labels.
- Semi-supervised ONNX predictions match the fitted wrapper.
- Clustering and anomaly ONNX outputs match mlrs.
- API responses never include absolute paths.
