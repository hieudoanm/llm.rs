# FEATURE-02: Column Mapping

## Priority

P0 (FR-010..FR-015)

## Description

After upload, the user declares which columns are features (**X**) and, when the chosen task requires one, which single column is the label (**Y**). The mapping is validated against the dataset header, the task type, and the label rules for that task.

## User-visible behaviour

- Multi-select for feature columns.
- Task select: Classification, Regression, Semi-supervised, Clustering, Anomaly detection.
- Single-select for the label column; shown only for classification, regression, and semi-supervised tasks.
- Semi-supervised helper: "Leave a label cell empty to treat that row as unlabelled."
- Unsupervised helper: "No label column is used."
- Disabled state until a dataset is loaded.
- Inline errors if Y is in X, if nothing is selected, if Y is missing for a label task, if Y is supplied for an unsupervised task, or if a regression Y is non-numeric.

## Inputs

| Field          | Type           | Required    | Constraints                                                                                                         |
| -------------- | -------------- | ----------- | ------------------------------------------------------------------------------------------------------------------- |
| datasetId      | UUID           | Yes         | Must exist                                                                                                          |
| featureColumns | string[]       | Yes         | Subset of header, ≥ 1                                                                                               |
| labelColumn    | string \| null | Conditional | Required for `classification`, `regression`, `semi_supervised`; absent/`null` for `clustering`, `anomaly_detection` |
| taskType       | enum           | Yes         | `classification` \| `regression` \| `semi_supervised` \| `clustering` \| `anomaly_detection`                        |

## Outputs

Mapping is not a standalone resource; it is stored on the job (`jobs.feature_columns`, `jobs.label_column`). `label_column` is `NULL` for unsupervised jobs.

## Behaviour

1. Load the dataset header.
2. Ensure every named column exists.
3. Apply the task rules:
   - `classification` / `regression` / `semi_supervised`: exactly one label column.
   - `clustering` / `anomaly_detection`: no label column. A supplied label is a mapping error.
4. Ensure `labelColumn ∉ featureColumns`.
5. For regression, coerce Y to numeric; fail if more than a documented null ratio cannot parse.
6. For semi-supervised, parse the label column and count labelled rows. Empty, whitespace-only, `NaN`, and `null` cells are unlabelled. Apply the FR-082 minimums before accepting the job.
7. Persist with job create.

## Edge cases

- All columns selected as X and none left for Y → validation error for a label-required task.
- A label column supplied for clustering or anomaly detection → `400 INVALID_COLUMN_MAP`.
- A label column missing for classification, regression, or semi-supervised → `400 INVALID_COLUMN_MAP`.
- Fewer than 10 labelled rows, fewer than two classes, or a class with fewer than two labelled rows → `400 INSUFFICIENT_LABELS`.
- A legitimate class label cannot be the empty string; empty cells always mean unlabelled.
- Duplicate names in the CSV header → upload should already have failed.

## Acceptance conditions

- A valid mapping proceeds to job create for all five task types.
- Invalid mapping returns `400 INVALID_COLUMN_MAP` with field details.
- Insufficient semi-supervised labels return `400 INSUFFICIENT_LABELS`.
