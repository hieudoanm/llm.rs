# FEATURE-09: Semi-Supervised Learning

## Priority

P0 (FR-080..FR-086). Planned in Phase 9.

## Description

Semi-supervised learning lets a user train a classifier when only part of the label column is filled. Empty label cells mean "unlabelled": those rows contribute features during training but are never scored. The first shipped estimator is scikit-learn's `SelfTrainingClassifier` with a logistic-regression base estimator.

## User-visible behaviour

- Task select adds **Semi-supervised**.
- The label select remains required, with the helper "Leave a label cell empty to treat that row as unlabelled."
- The mapping step reports the number of labelled and unlabelled rows after parsing
  (from the upload's per-column `nullCount`).
- The model step offers `self_training_classifier` with threshold (0.5–1.0) and max-iterations (≥ 1) controls.
- The job status screen shows the standard classification metrics plus labelled rows, unlabelled rows, and self-training iterations.
- The algorithm catalogue marks the task as semi-supervised.

## Inputs

| Field           | Type     | Required | Constraints                                 |
| --------------- | -------- | -------- | ------------------------------------------- |
| datasetId       | UUID     | Yes      | Must exist                                  |
| featureColumns  | string[] | Yes      | Subset of header, ≥ 1                       |
| labelColumn     | string   | Yes      | Exactly one header name; cells may be empty |
| taskType        | enum     | Yes      | `semi_supervised`                           |
| algorithmId     | string   | Yes      | `self_training_classifier`                  |
| hyperparameters | object   | No       | `threshold` (0.5–1.0), `max_iter` (≥ 1)     |
| splitRatio      | number   | No       | Default 0.2; applied to labelled rows only  |
| randomSeed      | integer  | No       | Default 42                                  |

## Label rules

A row is **unlabelled** when its label cell is empty, whitespace-only, `NaN`, or `null`. Every other value is a class label. The label column may be numeric or categorical; classes are compared as values, not by dtype.

A job requires:

- at least two distinct labelled classes;
- at least 10 labelled rows;
- at least 2 labelled rows per class.

Otherwise job create returns `400 INSUFFICIENT_LABELS` with details. A dataset with zero unlabelled rows is valid and degenerates to supervised self-training.

## Behaviour

1. Parse the label column and derive the labelled/unlabelled mask.
2. Validate the label minimums.
3. Split the **labelled** rows into train and test using `splitRatio` and `randomSeed`, stratified by class.
4. Build the training set from the labelled-train rows plus every unlabelled row; unlabelled rows carry the `-1` sentinel.
5. Fit the shared preprocessor on all training rows, then fit `SelfTrainingClassifier`.
6. Evaluate on the labelled test rows only.
7. Export:
   - ONNX: preprocessor + the fitted base estimator (`SelfTrainingClassifier.estimator_`), returning the original class labels.
   - `pkl`/`joblib`: the fitted `SelfTrainingClassifier` wrapper.
8. Record `labelledRows`, `unlabelledRows`, `iterations` (`n_iter_`), and the classification metrics.

## Metrics

| Metric           | Meaning                                                   |
| ---------------- | --------------------------------------------------------- |
| `accuracy`       | Share of held-out labelled rows classified correctly.     |
| `precisionMacro` | Macro-averaged precision over the held-out labelled rows. |
| `recallMacro`    | Macro-averaged recall over the held-out labelled rows.    |
| `f1Macro`        | Macro-averaged F1 over the held-out labelled rows.        |
| `labelledRows`   | Rows with a non-empty label.                              |
| `unlabelledRows` | Rows treated as unlabelled.                               |
| `iterations`     | Self-training iterations performed before stopping.       |

## Edge cases

- No unlabelled rows → valid; behaves as supervised training.
- All labels empty → `INSUFFICIENT_LABELS`.
- One class only → `INSUFFICIENT_LABELS`.
- A class with one labelled row → `INSUFFICIENT_LABELS`.
- Empty strings and whitespace are never class labels.
- ONNX export must not include the wrapper (no converter); the base estimator is the inference contract.

## Acceptance conditions

- A CSV with blank labels completes with classification metrics and ONNX.
- Metrics are computed on labelled hold-out rows only.
- ONNX predictions match the fitted wrapper on the full feature matrix.
- `pkl`/`joblib` (when requested) load and predict identically to the wrapper.
