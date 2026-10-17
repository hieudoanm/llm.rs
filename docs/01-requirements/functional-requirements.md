# Functional Requirements

This document specifies the functional requirements for mlrs-client (mlrs), organised by capability. Each requirement carries a unique ID (`FR-###`), a description, and a priority. IDs are grouped by capability block and are intentionally sparse; gaps are reserved for future requirements within that capability.

## Learning Task Types

mlrs supports six `taskType` values. The task type determines whether a label column is required, how the data is split, which algorithms are eligible, and which metrics are returned.

| `taskType`              | Supervision     | Label column                                      | Split                                    | Default metrics                                             |
| ----------------------- | --------------- | ------------------------------------------------- | ---------------------------------------- | ----------------------------------------------------------- |
| `classification`        | Supervised      | Required; categorical or discrete                 | Hold-out, stratified                     | accuracy, precisionMacro, recallMacro, f1Macro              |
| `regression`            | Supervised      | Required; numeric                                 | Hold-out                                 | rmse, mae, r2                                               |
| `semi_supervised`       | Semi-supervised | Required; empty cells are unlabelled              | Hold-out on labelled rows                | accuracy, precisionMacro, recallMacro, f1Macro + counts     |
| `clustering`            | Unsupervised    | Must be absent                                    | None (fit and score all rows)            | silhouette, daviesBouldin, calinskiHarabasz, inertia or BIC |
| `anomaly_detection`     | Unsupervised    | Must be absent                                    | None (fit and score all rows)            | anomalyCount, anomalyRate, scoreMean, scoreStd              |
| `reinforcement_learning` | Offline RL     | Must be absent; experience columns in `rlColumns` | Hold-out of transitions (whole episodes when present) | meanReward, meanQ, tdErrorMean, tdErrorStd + actions |

`classification`, `regression`, `semi_supervised`, `clustering`, and `anomaly_detection` are implemented (Phases 9–10 delivered the semi-supervised and unsupervised extensions). `reinforcement_learning` is specified here and delivered in Phase 11; it follows the same upload, job, poll, and export contracts.

## Dataset Upload

| ID     | Requirement                                                                                       | Priority |
| ------ | ------------------------------------------------------------------------------------------------- | -------- |
| FR-001 | The system shall accept a CSV file upload with a header row.                                      | Must     |
| FR-002 | The system shall reject files that are empty, lack a header, or exceed the documented size limit. | Must     |
| FR-003 | The system shall persist the uploaded bytes on the artefact volume under a generated identifier.  | Must     |
| FR-004 | The system shall compute a SHA-256 content hash of the uploaded file.                             | Must     |
| FR-005 | The system shall infer column names and sample dtypes from the header and first N rows.           | Must     |
| FR-006 | The system shall return a `datasetId`, hash, column preview, row count estimate, and `createdAt`. | Must     |
| FR-007 | The system shall reject an upload whose declared content type or file extension is not a CSV, or whose bytes do not sniff as text CSV. | Must     |
| FR-008 | The system shall reject a CSV whose header is missing, empty, contains empty column names, or contains duplicate column names. | Must     |
| FR-009 | The system shall reject a CSV that is not decodable as UTF-8 (a UTF-8 BOM is permitted). | Must     |

**Acceptance criteria:**

- Upload endpoint: `POST /api/v1/datasets` (`multipart/form-data`, field `file`).
- On success, returns `201 Created` with `{ datasetId, sha256, filename, byteSize, columns[], rowCount, createdAt }`.
- On empty or invalid CSV, returns `400 Bad Request` with `VALIDATION_ERROR`.
- On oversized file, returns `413 Payload Too Large` with `UPLOAD_TOO_LARGE`.
- On a non-CSV extension, a non-CSV content type, or bytes that sniff as a binary format (ZIP, gzip, PNG, PDF, ELF), returns `400 UNSUPPORTED_FILE_TYPE`.
- On a file that cannot be parsed as delimited text, returns `400 MALFORMED_CSV`.
- On a header with empty or duplicate column names, returns `400 VALIDATION_ERROR` with `details.field = "columns"`.
- On bytes that are not valid UTF-8, returns `400 MALFORMED_CSV`.
- Stored path is never taken from the original filename; original name is metadata only.

**Rationale:** Upload is the entry point. Hashing enables reuse detection and reproducibility. Generated paths prevent traversal. Content-type and magic-byte validation stops a mislabelled binary from reaching the parser, which keeps the upload surface honest about what it accepts.

**See also:** [`US-001`](../03-user-stories/US-001.md), [`FEATURE-01`](../02-features/FEATURE-01-Dataset-Upload.md), [`FEATURE-16`](../02-features/FEATURE-16-Dataset-Insights.md).

---

## Column Mapping

| ID     | Requirement                                                                                                                                                                                     | Priority |
| ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| FR-010 | The system shall allow the user to select one or more feature columns as **X**. For `reinforcement_learning`, X are state features and experience columns are declared separately via `rlColumns`. | Must     |
| FR-011 | The system shall require exactly one label column as **Y** for classification, regression, and semi-supervised jobs, and shall reject a label column for clustering, anomaly-detection, and reinforcement-learning jobs. | Must     |
| FR-012 | The system shall reject mappings where Y is also included in X.                                                                                                                                 | Must     |
| FR-013 | The system shall reject unknown column names (not present in the dataset header).                                                                                                               | Must     |
| FR-014 | The system shall persist the mapping with the subsequent training job. For `reinforcement_learning`, `rlColumns` must also be persisted and validated. | Must     |
| FR-015 | The system shall require Y to be numeric for regression, categorical or discrete for classification and semi-supervised, and absent for unsupervised and reinforcement-learning tasks.         | Must     |

**Acceptance criteria:**

- Mapping is submitted with `POST /api/v1/jobs` as `{ datasetId, featureColumns, labelColumn?, taskType, algorithmId, ... }`. For `reinforcement_learning`, submit `rlColumns` and omit `labelColumn`.
- Invalid mapping returns `400` with `INVALID_COLUMN_MAP` and field-level details.
- A missing Y for a label-required task, or a supplied Y for an unsupervised task, returns `400` with `INVALID_COLUMN_MAP`. For `reinforcement_learning`, missing `rlColumns` or a supplied `labelColumn` returns `400` with `INVALID_COLUMN_MAP`.
- Regression with a non-numeric Y returns `400` with `INVALID_LABEL_TYPE`.
- Semi-supervised jobs that do not meet the labelled-row minimum return `400` with `INSUFFICIENT_LABELS`.

**Rationale:** Explicit X/Y mapping avoids silent wrong-target training.

**See also:** [`US-002`](../03-user-stories/US-002.md), [`FEATURE-02`](../02-features/FEATURE-02-Column-Mapping.md).

---

## Model Selection

| ID     | Requirement                                                                                                               | Priority |
| ------ | ------------------------------------------------------------------------------------------------------------------------- | -------- |
| FR-020 | The system shall expose a catalogue of allow-listed algorithms for every supported task type.                             | Must     |
| FR-021 | The catalogue shall distinguish `classification`, `regression`, `semi_supervised`, `clustering`, `anomaly_detection`, and `reinforcement_learning`. | Must     |
| FR-022 | P0 algorithms shall be mlrs estimators that can export ONNX.                                                      | Must     |
| FR-023 | P1 neural-net algorithms shall be configurable mlrs MLPs (`mlp_classifier`, `mlp_regressor`) that expose their architecture through hyperparameters and export ONNX. | Should   |
| FR-024 | The system shall reject unknown `algorithmId` values.                                                                     | Must     |
| FR-025 | The system shall reject an algorithm that does not match the declared `taskType`.                                         | Must     |
| FR-026 | The system shall apply documented default hyperparameters when the client omits `hyperparameters`.                        | Must     |
| FR-027 | The system shall reject hyperparameter values outside the documented range with `INVALID_HYPERPARAMETERS`.                | Must     |

**Acceptance criteria:**

- Catalogue endpoint: `GET /api/v1/algorithms`, optionally filtered by any of the six `taskType` values.
- Each entry: `{ algorithmId, taskType, family, defaultHyperparameters, exports[] }` where `exports` always includes `onnx`.
- Unknown or mismatched algorithm returns `400` with `UNKNOWN_ALGORITHM` or `TASK_ALGORITHM_MISMATCH`.
- Out-of-range hyperparameters return `400` with `INVALID_HYPERPARAMETERS`.

**P0 mlrs catalogue (minimum):**

| algorithmId                    | taskType       |
| ------------------------------ | -------------- |
| `logistic_regression`          | classification |
| `random_forest_classifier`     | classification |
| `gradient_boosting_classifier` | classification |
| `decision_tree_classifier`     | classification |
| `extra_trees_classifier`       | classification |
| `k_neighbors_classifier`       | classification |
| `gaussian_naive_bayes`         | classification |
| `svm_classifier`               | classification |
| `mlp_classifier`               | classification |
| `linear_regression`            | regression     |
| `ridge_regression`             | regression     |
| `random_forest_regressor`      | regression     |
| `decision_tree_regressor`      | regression     |
| `extra_trees_regressor`        | regression     |
| `k_neighbors_regressor`        | regression     |
| `lasso_regression`             | regression     |
| `elastic_net_regression`       | regression     |
| `svm_regressor`                | regression     |
| `mlp_regressor`                | regression     |

**P0 semi-supervised catalogue (Phase 9):**

| algorithmId                | taskType          | Estimator                                                   |
| -------------------------- | ----------------- | ----------------------------------------------------------- |
| `self_training_classifier` | `semi_supervised` | `SelfTrainingClassifier(estimator=LogisticRegression(...))` |

**P0 unsupervised catalogue (Phase 10):**

| algorithmId         | taskType            | Estimator         |
| ------------------- | ------------------- | ----------------- |
| `kmeans`            | `clustering`        | `KMeans`          |
| `mini_batch_kmeans` | `clustering`        | `MiniBatchKMeans` |
| `gaussian_mixture`  | `clustering`        | `GaussianMixture` |
| `isolation_forest`  | `anomaly_detection` | `IsolationForest` |

**P0 reinforcement-learning catalogue (Phase 11):**

| algorithmId         | taskType                | Estimator                                                |
| ------------------- | ----------------------- | -------------------------------------------------------- |
| `fitted_q_iteration` | `reinforcement_learning` | `GradientBoostingRegressor` Q-function (Fitted Q-Iteration) |

**P1 neural nets (FR-023):** `mlp_classifier` and `mlp_regressor` are scikit-learn
`MLPClassifier` / `MLPRegressor` models whose architecture is configurable through
hyperparameters (see [`models.md`](../08-tech-stack/backend/models.md#neural-networks-p1)).
They are tabular multi-layer perceptrons.

**ONNX exclusions:** DBSCAN, AgglomerativeClustering, LabelSpreading, and LabelPropagation have no verified ONNX converter with the pinned stack and are therefore not in the catalogue.

**See also:** [`US-003`](../03-user-stories/US-003.md), [`US-013`](../03-user-stories/US-013.md), [`US-014`](../03-user-stories/US-014.md), [`US-015`](../03-user-stories/US-015.md), [`US-016`](../03-user-stories/US-016.md), [`FEATURE-03`](../02-features/FEATURE-03-Model-Selection.md).

---

## Training Jobs

| ID     | Requirement                                                                                                                                                                             | Priority |
| ------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| FR-030 | The system shall create a training job from a dataset, mapping, task type, and algorithm.                                                                                               | Must     |
| FR-031 | The system shall return `202 Accepted` with `jobId` and status `queued` without blocking on fit.                                                                                        | Must     |
| FR-032 | The system shall run validation, fit, evaluation, and export on a worker process.                                                                                                       | Must     |
| FR-033 | Job status shall progress through `queued`, `validating`, `training`, `evaluating`, `exporting`, then `completed` or `failed`.                                                          | Must     |
| FR-034 | The system shall split data for evaluation using a documented ratio and seed (default 0.2, seed 42); semi-supervised jobs split labelled rows only, unsupervised jobs do not split, and reinforcement-learning jobs split by episode (or by transition when no episode column exists). | Must     |
| FR-035 | Classification jobs shall return accuracy, precision, recall, and f1 (macro).                                                                                                           | Must     |
| FR-036 | Regression jobs shall return rmse, mae, and r2.                                                                                                                                         | Must     |
| FR-037 | The system shall persist library versions used for the job.                                                                                                                             | Must     |
| FR-038 | The system shall expose `GET /api/v1/jobs/{jobId}` for polling.                                                                                                                         | Must     |
| FR-039 | Failed jobs shall store a stable error code and a human-readable message (no stack traces in the UI).                                                                                   | Must     |
| FR-040 | The system shall reject a second mutation of a terminal job (no restart-in-place).                                                                                                      | Must     |

**Acceptance criteria:**

- `POST /api/v1/jobs` returns `{ jobId, status: "queued", datasetId, algorithmId, createdAt }`.
- Polling returns current `status`, optional `metrics`, optional `artefacts`, optional `error`.
- Terminal states: `completed`, `failed`.
- Duplicate create of a _new_ job is allowed (training is not idempotent on payload); clients may send `Idempotency-Key` to avoid double-submit from the UI.

**Rationale:** Fit can take seconds to minutes. Polling matches the local-only topology without introducing Kafka.

**See also:** [`US-004`](../03-user-stories/US-004.md), [`US-005`](../03-user-stories/US-005.md), [`FEATURE-04`](../02-features/FEATURE-04-Training-Job.md).

---

## Semi-Supervised Learning

| ID     | Requirement                                                                                                                                                                | Priority |
| ------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| FR-080 | The system shall support `taskType = semi_supervised` for classification with partially labelled data.                                                                     | Must     |
| FR-081 | A semi-supervised job shall require a label column; empty, whitespace-only, `NaN`, or `null` label cells denote unlabelled rows.                                           | Must     |
| FR-082 | A semi-supervised job shall require at least two labelled classes, at least 10 labelled rows, and at least 2 labelled rows per class; otherwise `400 INSUFFICIENT_LABELS`. | Must     |
| FR-083 | The system shall include all unlabelled rows in training features while excluding them from evaluation.                                                                    | Must     |
| FR-084 | Evaluation shall use a stratified hold-out of labelled rows only and return accuracy, precisionMacro, recallMacro, f1Macro, labelledRows, unlabelledRows, and iterations.  | Must     |
| FR-085 | ONNX export shall contain the fitted self-training base estimator and predict the original class labels; ONNX predictions shall match the fitted wrapper.                  | Must     |
| FR-086 | The system shall not require a label for excluded rows and shall not score unlabelled rows during evaluation.                                                              | Must     |

**Acceptance criteria:**

- `POST /api/v1/jobs` accepts `{ taskType: "semi_supervised", algorithmId: "self_training_classifier", labelColumn }`.
- Blank label cells are treated as unlabelled and are never included in the evaluation set.
- A CSV with fewer than 10 labelled rows, one class, or a class with one labelled row returns `400 INSUFFICIENT_LABELS`.
- A completed job returns the classification metric set plus `labelledRows`, `unlabelledRows`, and `iterations`.
- A CSV with no blank labels is valid and behaves as supervised training.

**Rationale:** Self-training improves a classifier when labels are expensive but unlabelled rows are available. Exporting the fitted base estimator preserves inference semantics because self-training only changes the training labels.

**See also:** [`US-013`](../03-user-stories/US-013.md), [`FEATURE-09`](../02-features/FEATURE-09-Semi-Supervised-Learning.md).

---

## Clustering

| ID     | Requirement                                                                                                                                                                                       | Priority |
| ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| FR-090 | The system shall support `taskType = clustering`; a clustering job shall not accept a label column.                                                                                               | Must     |
| FR-091 | The clustering catalogue shall include `kmeans`, `mini_batch_kmeans`, and `gaussian_mixture`.                                                                                                     | Must     |
| FR-092 | The system shall default the cluster count to 3 and require `2 <= k <= rows - 1` for KMeans/MiniBatchKMeans `n_clusters` and GaussianMixture `n_components`; otherwise `INVALID_HYPERPARAMETERS`. | Must     |
| FR-093 | A clustering job shall fit preprocessing and the estimator on all rows and shall not create a hold-out split.                                                                                     | Must     |
| FR-094 | Clustering jobs shall return silhouette, daviesBouldin, and calinskiHarabasz; KMeans/MiniBatchKMeans shall add inertia and GaussianMixture shall add BIC.                                         | Must     |
| FR-095 | ONNX export shall be required and cluster-label predictions shall match mlrs `predict`.                                                                                                        | Must     |
| FR-096 | Job and poll payloads shall expose `labelColumn = null` and the clustering metrics for completed jobs.                                                                                            | Must     |

**Acceptance criteria:**

- `POST /api/v1/jobs` with `taskType = "clustering"` and no `labelColumn` is accepted.
- Supplying `labelColumn` returns `400 INVALID_COLUMN_MAP`.
- `n_clusters`/`n_components` below 2 or greater than or equal to the row count returns `400 INVALID_HYPERPARAMETERS`.
- Completed jobs return the documented clustering metrics and an `onnx` artefact.
- ONNX output labels match mlrs for every clustering algorithm.

**Rationale:** Clustering is the natural first unsupervised workflow on tabular CSVs and all three estimators have verified ONNX support.

**See also:** [`US-014`](../03-user-stories/US-014.md), [`FEATURE-10`](../02-features/FEATURE-10-Unsupervised-Learning.md).

---

## Anomaly Detection

| ID     | Requirement                                                                                                                                                     | Priority |
| ------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| FR-100 | The system shall support `taskType = anomaly_detection`; an anomaly-detection job shall not accept a label column.                                              | Must     |
| FR-101 | The anomaly-detection catalogue shall include `isolation_forest`.                                                                                               | Must     |
| FR-102 | `contamination` shall accept `"auto"` or a number in `(0, 0.5]`; other values return `INVALID_HYPERPARAMETERS`.                                                 | Must     |
| FR-103 | An anomaly-detection job shall fit preprocessing and the estimator on all rows and shall not create a hold-out split.                                           | Must     |
| FR-104 | Anomaly-detection jobs shall return anomalyCount, anomalyRate, scoreMean, and scoreStd; scores are `decision_function` values where lower means more anomalous. | Must     |
| FR-105 | ONNX export shall be required; output labels (`1` inlier, `-1` outlier) and scores shall match mlrs.                                                         | Must     |

**Acceptance criteria:**

- `POST /api/v1/jobs` with `taskType = "anomaly_detection"` and no `labelColumn` is accepted.
- `contamination = 0.6` or `contamination = 0` returns `400 INVALID_HYPERPARAMETERS`.
- Completed jobs return the documented anomaly metrics and an `onnx` artefact.
- ONNX labels match mlrs `predict` and scores match `decision_function` within tolerance.

**Rationale:** IsolationForest is a standard tabular anomaly detector with a verified ONNX converter, giving the studio a complete unsupervised workflow.

**See also:** [`US-015`](../03-user-stories/US-015.md), [`FEATURE-10`](../02-features/FEATURE-10-Unsupervised-Learning.md).

---

## Reinforcement Learning

| ID     | Requirement                                                                                                                                                                                                       | Priority |
| ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| FR-110 | The system shall support `taskType = reinforcement_learning`; a reinforcement-learning job shall not accept a label column and shall require an `rlColumns` object.                                                 | Must     |
| FR-111 | `rlColumns` shall declare `actionColumn` and `rewardColumn`, plus either `nextStateColumns` (mirroring `featureColumns`) or `episodeColumn`, and optionally `doneColumn`; experience columns must exist in the header, must not overlap `featureColumns`, and the action must be discrete (2–50 distinct values) with a numeric reward; violations return `INVALID_COLUMN_MAP` or `INVALID_LABEL_TYPE`. | Must     |
| FR-112 | The reinforcement-learning catalogue shall include `fitted_q_iteration`; `n_iterations` (1..50), `gamma` (0..1), `max_depth` (1..20), and `learning_rate` ((0,1]) shall default to 10, 0.9, 3, and 0.1, and out-of-range values return `INVALID_HYPERPARAMETERS`.                    | Must     |
| FR-113 | A reinforcement-learning job shall resolve transitions (explicit next state, or the next row within an episode), split by episode when an episode column exists and by transition otherwise, and fit the Q-function with Fitted Q-Iteration on the training side only.                | Must     |
| FR-114 | Reinforcement-learning jobs shall return `meanReward`, `meanQ`, `tdErrorMean`, `tdErrorStd`, and `actions`, plus `episodeCount` and `meanEpisodeReturn` when an episode column was declared; `tdErrorMean` is the mean absolute Bellman residual on the evaluation side of the split. | Must     |
| FR-115 | ONNX export shall be required and shall contain the fitted preprocessor + Q-function regressor taking the state features followed by a one-hot action block; ONNX Q-values shall match mlrs.                                                                                     | Must     |

**Acceptance criteria:**

- `POST /api/v1/jobs` accepts `{ taskType: "reinforcement_learning", algorithmId: "fitted_q_iteration", featureColumns, rlColumns }` with no `labelColumn`.
- A supplied `labelColumn`, a missing `rlColumns`, an experience column absent from the header or shared with `featureColumns`, or neither `nextStateColumns` nor `episodeColumn` returns `400 INVALID_COLUMN_MAP`.
- A non-numeric reward returns `400 INVALID_LABEL_TYPE`.
- Fewer than 10 transitions, fewer than two episodes with an episode column, or a hyperparameter outside its documented range returns `400 INVALID_HYPERPARAMETERS`.
- A completed job returns the documented RL metrics (including `actions`) and an `onnx` artefact whose Q-values match mlrs.
- The split never places both sides of a transition pair across train/hold-out when episodes are declared: whole episodes go to one side.

**Rationale:** Offline (batch) reinforcement learning from a logged experience CSV fits the CSV → train → ONNX philosophy: the Q-function is a scikit-learn regressor, so the existing preprocessor, split, allow-list, and export pipeline apply unchanged and no RL framework dependency is introduced. The policy (`argmax` over the recorded action set) is a documented consumer-side rule over the exported Q-function.

**See also:** [`US-016`](../03-user-stories/US-016.md), [`FEATURE-11`](../02-features/FEATURE-11-Reinforcement-Learning.md).

---

## Model Export

| ID     | Requirement                                                                                     | Priority |
| ------ | ----------------------------------------------------------------------------------------------- | -------- |
| FR-050 | A successful job shall always write an ONNX file and an artefact row with `format = onnx`.      | Must     |
| FR-051 | The ONNX file shall load with the pinned ONNX runtime used in tests.                            | Must     |
| FR-052 | Sklearn jobs may write an optional `.pkl` when `exports` includes `pkl`.                        | Should   |
| FR-053 | Sklearn jobs may write an optional `.joblib` when `exports` includes `joblib`.                  | Should   |
| FR-054 | The artefact `format` domain is `onnx`, `pkl`, `joblib`. | Should   |
| FR-055 | `GET /api/v1/jobs/{jobId}/artefacts/{artefactId}` shall stream the file as an attachment.       | Must     |
| FR-056 | Artefact download for a missing or failed job shall return `404`.                               | Must     |
| FR-057 | The system shall never expose filesystem paths in API responses; only artefact ids and formats. | Must     |

**Acceptance criteria:**

- Completed job payload lists artefacts as `{ artefactId, format, filename, byteSize }`.
- `format` is one of `onnx`, `pkl`, `joblib`.
- ONNX is present whenever `status = completed`.
- Optional formats are omitted (not null placeholders) when not produced.

**See also:** [`US-006`](../03-user-stories/US-006.md), [`US-007`](../03-user-stories/US-007.md), [`FEATURE-05`](../02-features/FEATURE-05-Model-Export.md).

---

## Job History

| ID     | Requirement                                                                              | Priority |
| ------ | ---------------------------------------------------------------------------------------- | -------- |
| FR-060 | The system shall list training jobs newest first.                                        | Must     |
| FR-061 | The list shall support limit/offset or cursor pagination (default 20, max 100).          | Must     |
| FR-062 | Each list item shall include jobId, status, algorithmId, taskType, createdAt, updatedAt. | Must     |
| FR-063 | The system shall allow deleting a job and its artefact files (local cleanup).            | Should   |

**Acceptance criteria:**

- `GET /api/v1/jobs?limit=&cursor=`
- `DELETE /api/v1/jobs/{jobId}` returns `204`, removes DB rows and files, and cancels an in-flight worker so it stops without writing metrics or artefacts.

**See also:** [`US-009`](../03-user-stories/US-009.md), [`FEATURE-06`](../02-features/FEATURE-06-Job-History.md).

---

## Job Discovery

| ID     | Requirement                                                                                                         | Priority |
| ------ | ------------------------------------------------------------------------------------------------------------------- | -------- |
| FR-120 | The jobs list shall support filtering by `status`, `taskType`, `algorithmId`, `tag`, free-text `q`, and a `createdFrom`/`createdTo` date range. | Must     |
| FR-121 | Filters shall combine with AND semantics; a tag filter shall require **all** selected tags to be present.              | Must     |
| FR-122 | The jobs list shall support sorting by `createdAt`, `updatedAt`, `status`, and `algorithmId`, in ascending and descending order, with a stable tie-break. | Must     |
| FR-123 | Free-text search shall match the job name, its tags, and its algorithm id, case-insensitively.                        | Must     |
| FR-124 | A job shall carry an optional human-readable `name` (max 120 characters) and up to 10 tags (1–32 characters each, lowercase-normalised). | Must     |
| FR-125 | The system shall allow `name` and `tags` to be updated on an existing job; the training configuration of a job shall remain immutable. | Must     |
| FR-126 | Filtering and sorting shall not change any job's status, metrics, or artefacts.                                        | Must     |
| FR-127 | The jobs list shall distinguish an empty workspace ("no jobs yet") from an empty filter result ("no jobs match these filters"). | Must     |

**Acceptance criteria:**

- `GET /api/v1/jobs?status=&taskType=&algorithmId=&tag=&q=&createdFrom=&createdTo=&sort=`
- `PATCH /api/v1/jobs/{jobId}` with `{ "name": "string | null", "tags": ["string"] }` returns `200`.
- Unknown filter values, malformed dates, and unknown sort fields return `400 VALIDATION_ERROR` with `details.field`.
- Duplicate tags in one request are collapsed; more than 10 tags returns `400 VALIDATION_ERROR`.

**See also:** [`FEATURE-13`](../02-features/FEATURE-13-Job-Discovery.md).

---

## Job Comparison

| ID     | Requirement                                                                                          | Priority |
| ------ | ---------------------------------------------------------------------------------------------------- | -------- |
| FR-130 | The system shall compare 2 to 5 jobs side by side and return their configuration and metrics in one response. | Must     |
| FR-131 | The comparison response shall include warnings when the compared jobs use different task types, different datasets, or different feature columns. | Must     |
| FR-132 | A job that is not `completed` shall be returned with `metrics = null` and its recorded error, without failing the comparison. | Must     |
| FR-133 | The comparison shall be read-only: it shall not change job state, metrics, or artefacts.                 | Must     |
| FR-134 | An unknown job id shall return `404 JOB_NOT_FOUND` with `details.missingJobIds`.                       | Must     |

**Acceptance criteria:**

- `GET /api/v1/jobs/compare?ids=<uuid>,<uuid>` returns `200` with one entry per job.
- Fewer than 2 or more than 5 ids returns `400 VALIDATION_ERROR` with `details.min` / `details.max`.

**See also:** [`FEATURE-14`](../02-features/FEATURE-14-Job-Comparison.md).

---

## Bulk Artefact Download

| ID     | Requirement                                                                                     | Priority |
| ------ | ----------------------------------------------------------------------------------------------- | -------- |
| FR-140 | The system shall serve a ZIP bundle containing every artefact of a job, plus a `metrics.json` sidecar and a `manifest.json`. | Must     |
| FR-141 | The system shall serve a ZIP bundle spanning 1 to 10 jobs, organised in one directory per job.   | Must     |
| FR-142 | A multi-job bundle shall list the omitted jobs and the reason in a top-level `skipped.json`.      | Must     |
| FR-143 | Bundle generation shall not write to the artefact volume; temporary files shall be removed after the response. | Must     |
| FR-144 | A job with no artefact rows shall return `404 ARTEFACT_NOT_FOUND` with `details.jobId`.           | Must     |

**Acceptance criteria:**

- `GET /api/v1/jobs/{jobId}/artefacts.zip` returns `200` with `Content-Type: application/zip`.
- `GET /api/v1/jobs/artefacts.zip?ids=<uuid>,<uuid>` returns `200` with one directory per job.
- The extracted ONNX file byte-matches the artefact served by `GET /api/v1/jobs/{jobId}/artefacts/{artefactId}`.

**See also:** [`FEATURE-15`](../02-features/FEATURE-15-Bulk-Artefact-Download.md).

---

## Dataset Insights

| ID     | Requirement                                                                                                        | Priority |
| ------ | ------------------------------------------------------------------------------------------------------------------ | -------- |
| FR-150 | The system shall return a per-column profile for every column of a dataset: inferred type, missing count and rate, and distinct-value count. | Must     |
| FR-151 | Numeric columns shall report `min`, `max`, `mean`, `std`, and `median`; categorical columns shall report their most frequent values with counts. | Must     |
| FR-152 | The system shall return dataset-level warnings for constant columns, high missing rates, all-missing columns, high cardinality, possible identifier columns, mixed types, text columns, and single-value label candidates. | Must     |
| FR-153 | The system shall return a preview of the first rows of the dataset, bounded by a caller-supplied limit and offset. | Must     |
| FR-154 | Insights shall be advisory: no warning shall block job creation.                                                     | Must     |
| FR-155 | Insights shall be computed once per dataset content hash and remain stable for that dataset.                        | Must     |
| FR-156 | An unknown dataset id shall return `404 DATASET_NOT_FOUND`.                                                          | Must     |

**Acceptance criteria:**

- `GET /api/v1/datasets/{datasetId}/insights?previewLimit=&previewOffset=` returns `200`.
- `previewLimit` outside 1–100 returns `400 VALIDATION_ERROR` with `details.field = "previewLimit"`.

**See also:** [`FEATURE-16`](../02-features/FEATURE-16-Dataset-Insights.md).

---

## Reliable Job Execution

| ID     | Requirement                                                                                                   | Priority |
| ------ | ------------------------------------------------------------------------------------------------------------- | -------- |
| FR-160 | The worker shall execute at most `mlrs_MAX_CONCURRENT_JOBS` jobs concurrently; the default is 1.                | Must     |
| FR-161 | Jobs beyond the concurrency limit shall remain `queued` and the API shall report their queue position.        | Must     |
| FR-162 | Each job attempt shall be bounded by `mlrs_JOB_TIMEOUT_SECONDS` (default 1800).                                 | Must     |
| FR-163 | A job that exceeds its budget shall be recorded `failed` with the error code `JOB_TIMEOUT` and shall leave no artefact behind. | Must     |
| FR-164 | A transient failure shall be retried up to `mlrs_JOB_MAX_ATTEMPTS` times (default 2).                           | Must     |
| FR-165 | A deterministic failure shall not be retried.                                                                  | Must     |
| FR-166 | The delay between attempts shall be exponential with full jitter and shall never be negative.                   | Must     |
| FR-167 | A job whose attempts are exhausted shall be recorded `failed` with the error code `JOB_RETRY_EXHAUSTED`.        | Must     |
| FR-168 | The job payload shall expose `attempts`, `maxAttempts`, and `nextAttemptAt`.                                    | Must     |
| FR-169 | The job status `retrying` shall be a non-terminal status that keeps the client polling.                        | Must     |

**Acceptance criteria:**

- `GET /api/v1/jobs/{jobId}` exposes `attempts`, `maxAttempts`, and `nextAttemptAt`.
- A job waiting for a retry reports `status = "retrying"`.

**See also:** [`FEATURE-17`](../02-features/FEATURE-17-Reliable-Job-Execution.md),
[`FEATURE-04`](../02-features/FEATURE-04-Training-Job.md).

---

## Client Resilience & Actionable Guidance

| ID     | Requirement                                                                                                     | Priority |
| ------ | --------------------------------------------------------------------------------------------------------------- | -------- |
| FR-170 | The API client shall abort a request that exceeds its timeout and surface a distinguishable timeout error.       | Must     |
| FR-171 | The API client shall retry idempotent requests on transient failures with randomised exponential backoff.        | Must     |
| FR-172 | The API client shall not retry a request whose failure is deterministic, and shall retry a `POST` only when it carries an `Idempotency-Key`. | Must     |
| FR-173 | The API client shall cancel in-flight requests when the calling component unmounts.                              | Must     |
| FR-174 | Every documented backend error code shall have a user-facing title, a plain-language message, and a retryable flag. | Must     |
| FR-175 | Error alerts shall render the guidance and a primary action, and shall never render a raw stack trace.            | Must     |
| FR-176 | The job wizard shall validate each step live and list every blocking problem with a control that focuses it.     | Must     |
| FR-177 | The wizard's primary action shall remain enabled on an invalid step and shall explain itself when used.          | Must     |
| FR-178 | The wizard shall restore its in-progress state after a page reload and shall clear it when a job is accepted.     | Must     |

**Acceptance criteria:**

- A `GET` that fails with a network error is retried; a `GET` that returns `400` is not.
- Clicking the wizard's primary action on an invalid step moves focus to the first invalid field.
- Every error code in the catalogue renders a distinct, actionable message.

**See also:** [`FEATURE-18`](../02-features/FEATURE-18-Client-Resilience-and-Guidance.md),
[`FEATURE-07`](../02-features/FEATURE-07-Error-Handling.md).

---

## API Documentation

| ID     | Requirement                                                                                     | Priority |
| ------ | ----------------------------------------------------------------------------------------------- | -------- |
| FR-190 | The backend shall serve an interactive API reference at `/docs` and `/redoc`, and the raw schema at `/openapi.json`. | Must     |
| FR-191 | The committed `docs/06-api-specification/backend/openapi.yaml` shall be regenerated from the code and checked for drift in CI. | Must     |
| FR-192 | The interactive reference and the committed schema shall be linked from the product documentation and from the application. | Must     |
| FR-193 | Interactive documentation shall be disableable via `mlrs_ENABLE_INTERACTIVE_DOCS`.                | Should   |

**Acceptance criteria:**

- `GET /openapi.json` returns the generated OpenAPI document.
- CI fails when `docs/06-api-specification/backend/openapi.yaml` differs from the generated document.

**See also:** [`06-api-specification.md`](../06-api-specification.md).

---

## Health

| ID     | Requirement                                                                | Priority |
| ------ | -------------------------------------------------------------------------- | -------- |
| FR-070 | The backend shall serve `GET /healthz` (liveness) at the process root.     | Must     |
| FR-071 | The backend shall serve `GET /readyz` that checks SQLite connectivity. | Must     |
| FR-072 | Compose healthchecks shall gate frontend, backend, and sqlite readiness. | Must     |

**See also:** [`US-012`](../03-user-stories/US-012.md).

---

## Requirements Traceability

| Requirement | User Story             | Feature | Architecture           | Test                   |
| ----------- | ---------------------- | ------- | ---------------------- | ---------------------- |
| FR-001..006 | US-001, US-010         | F-001   | Backend upload, volume | test_dataset_upload    |
| FR-010..015 | US-002                 | F-002   | Job create validation  | test_column_map        |
| FR-020..027 | US-003                 | F-003   | Algorithm catalogue    | test_algorithms        |
| FR-030..040 | US-004, US-005, US-011 | F-004   | Worker                 | test_train_job         |
| FR-080..086 | US-013                 | F-009   | Self-training pipeline | test_semi_supervised   |
| FR-090..096 | US-014                 | F-010   | Clustering pipeline    | test_clustering        |
| FR-100..105 | US-015                 | F-010   | Anomaly pipeline       | test_anomaly_detection |
| FR-110..115 | US-016                 | F-011   | RL pipeline            | test_rl                |
| FR-050..057 | US-006, US-007         | F-005   | ONNX export            | test_onnx_export       |
| FR-060..063 | US-008, US-009         | F-006   | SQLite jobs        | test_job_history       |
| FR-070..072 | US-012                 | —       | Compose                | test_health            |
| FR-007..009 | US-001, US-010         | F-001   | Upload validation      | test_upload_validation |
| FR-120..127 | US-009                 | F-013   | Job query + `tags`    | test_job_filters       |
| FR-130..134 | US-009                 | F-014   | Job query              | test_job_compare       |
| FR-140..144 | US-007                 | F-015   | Volume + zip           | test_bulk_download     |
| FR-150..156 | US-001, US-010         | F-016   | Dataset profile        | test_dataset_insights  |
| FR-160..169 | US-004, US-005, US-011 | F-017   | Worker                 | test_worker_resilience |
| FR-170..178 | US-001..US-009         | F-018   | Frontend client        | test_client_resilience |
| FR-190..193 | US-012                 | —       | OpenAPI generation     | test_openapi_drift     |

**Cross-references:**

- Features: [`02-features.md`](../02-features.md)
- User stories: [`03-user-stories.md`](../03-user-stories.md)
- Architecture: [`04-architecture.md`](../04-architecture.md)
- API: [`06-api-specification.md`](../06-api-specification.md)
