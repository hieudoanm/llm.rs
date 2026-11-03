# FEATURE-03: Model Selection

## Priority

P0 (FR-020..FR-027); configurable MLP hyperparameters are P1 (FR-023)

## Description

Users pick a learning task and an allow-listed algorithm. The UI loads the catalogue from the API so the frontend never hard-codes hidden estimators.

## Catalogue

The catalogue ships 25 estimators across six task types:

| Task                    | Count | Algorithm ids                                     |
| ----------------------- | ----- | ------------------------------------------------- |
| Classification          | 9     | `logistic_regression` … `mlp_classifier`          |
| Regression              | 10    | `linear_regression` … `mlp_regressor`             |
| Semi-supervised         | 1     | `self_training_classifier`                        |
| Clustering              | 3     | `kmeans`, `mini_batch_kmeans`, `gaussian_mixture` |
| Anomaly detection       | 1     | `isolation_forest`                                |
| Reinforcement learning  | 1     | `fitted_q_iteration`                              |

The shipped ids remain authoritative in [`01-requirements`](../01-requirements/functional-requirements.md); the backend registry lives in `crates/llm-api/src/training/catalogue/` (assembled in `catalogue/algorithms.py`) and the integrity tests in `crates/llm-api/tests/training/catalogue/` assert the two stay in sync. Families: linear models, trees and ensembles, k-NN, Gaussian Naive Bayes, SVM, MLP, self-training, centroid/probabilistic clustering, and isolation forests. Every entry must fit the pipeline, score, and convert to ONNX with predictions matching llm (`test_trains_and_exports_onnx`).

## User-visible behaviour

- Task toggle: Classification / Regression / Semi-supervised / Clustering / Anomaly detection, grouped by supervision mode.
- Algorithm select filtered by task.
- Short description and default hyperparameters displayed.
- Task-critical hyperparameter controls:
  - Semi-supervised: self-training threshold (0.5–1.0) and max iterations (≥ 1).
  - Clustering: number of clusters/components (2 to rows − 1, default 3).
  - Anomaly detection: contamination (`auto` or 0 < value ≤ 0.5).
- Advanced accordion (P1) to override any remaining defaults as JSON or typed fields.

## Inputs

| Field           | Type   | Required | Constraints          |
| --------------- | ------ | -------- | -------------------- |
| taskType        | enum   | Yes      | Matches algorithm    |
| algorithmId     | string | Yes      | Catalogue allow-list |
| hyperparameters | object | No       | Merged over defaults |

## Outputs

Catalogue: `GET /api/v1/algorithms` → list of algorithms with `exports` (`onnx` always; `pkl`/`joblib` for llm).

## Behaviour

1. Filter algorithms by `taskType`.
2. Reject unknown ids.
3. Deep-merge client hyperparameters over defaults; unknown keys or out-of-range values → `VALIDATION_ERROR` / `INVALID_HYPERPARAMETERS`.
4. Record resolved hyperparameters on the job (not only the client overlay).
5. Reject algorithms without a verified ONNX export at catalogue integrity time, before they can be selected.

## Edge cases

- Out-of-range architecture hyperparameter (`hidden_layer_sizes` of the wrong shape, `max_iter` of 0) → `INVALID_HYPERPARAMETERS`.
- Hyperparameter type mismatch (string for `n_estimators`) → `VALIDATION_ERROR`.
- `n_clusters` greater than or equal to the dataset row count → `INVALID_HYPERPARAMETERS`.
- Labelled data insufficient for self-training → `INSUFFICIENT_LABELS` (mapping validation, FEATURE-02).

## Acceptance conditions

- P0 llm ids work for the matching task.
- Mismatched task/algorithm returns `400 TASK_ALGORITHM_MISMATCH`.
- Every catalogue entry exports ONNX and matches llm predictions.
