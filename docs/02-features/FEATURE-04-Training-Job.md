# FEATURE-04: Training Job

## Priority

P0 (FR-030..FR-040)

## Description

Creating a job enqueues training. A worker validates the CSV against the mapping, fits the estimator with the task's data strategy, evaluates with task-appropriate metrics, and advances status until `completed` or `failed`. HTTP never blocks on `fit`.

## User-visible behaviour

- Primary action: "Start training".
- Redirect or navigate to job status.
- Status stepper: queued → validating → training → evaluating → exporting → completed.
- Metrics card on success, with labels and direction (higher/lower is better) for the selected task.
- Error alert with retry guidance on failure.
- Polling until terminal status.

## Inputs

See [`06-api-specification.md`](../06-api-specification.md) `POST /api/v1/jobs`.

Optional `Idempotency-Key` header (1–255 `[a-zA-Z0-9_-]`) prevents double-submit from the UI.

Idempotency contract (`POST /api/v1/jobs` only):

- With no `Idempotency-Key`, every request creates a new job.
- The first request with a given key creates the job and caches the response; a later request with the **same key and an identical payload** replays the cached `202` body (same `jobId`) without creating another job.
- The same key with a **different payload** returns `409 IDEMPOTENCY_KEY_EXISTS`.
- Keys expire after 24 hours, after which the key may be reused.
- The key may only be supplied for job creation; the frontend generates one per wizard submission so a double-click cannot start two jobs.

## Outputs

`202 Accepted`: `{ jobId, status, datasetId, taskType, algorithmId, createdAt }`.

Poll payload adds `status`, `metrics`, `artefacts`, `error`, `updatedAt`, `libraryVersions`.

## Behaviour

1. Validate payload, catalogue, mapping, task-specific label rules, and optional idempotency key.
2. Insert job `queued`; `label_column` is `NULL` for unsupervised tasks.
3. Enqueue worker (in-process asyncio task queue or a dedicated worker container sharing the DB — see architecture; MVP is a Axum background worker with a Postgres `FOR UPDATE SKIP LOCKED` claim).
4. Worker: `validating` → load CSV → resolve the task strategy → encode features → apply the task split → `training` → fit → `evaluating` → metrics → `exporting` → ONNX (+ optional) → `completed`.
5. On exception: `failed` with `TRAINING_FAILED` or a more specific code (`INVALID_COLUMN_MAP`, `INSUFFICIENT_LABELS`, `INVALID_HYPERPARAMETERS`, `EXPORT_FAILED`).

### Task split and fit strategies

| Task                | Training rows                                     | Evaluation rows             | Preprocessor fit  |
| ------------------- | ------------------------------------------------- | --------------------------- | ----------------- |
| `classification`    | Random train split, stratified by Y               | Held-out labelled rows      | Train split only  |
| `regression`        | Random train split                                | Held-out labelled rows      | Train split only  |
| `semi_supervised`   | Labelled train split **plus all unlabelled rows** | Held-out labelled rows only | All training rows |
| `clustering`        | All rows                                          | All rows (in-sample)        | All rows          |
| `anomaly_detection` | All rows                                          | All rows (in-sample)        | All rows          |

Unsupervised jobs ignore `splitRatio`; the value is accepted for API compatibility and recorded but has no effect.

## Metrics

**Classification and semi-supervised:** `accuracy`, `precisionMacro`, `recallMacro`, `f1Macro`. Semi-supervised adds `labelledRows`, `unlabelledRows`, and `iterations`.

**Regression:** `rmse`, `mae`, `r2`.

**Clustering:** `silhouette`, `daviesBouldin`, `calinskiHarabasz`, plus `inertia` (KMeans/MiniBatchKMeans) or `bic` (GaussianMixture).

**Anomaly detection:** `anomalyCount`, `anomalyRate`, `scoreMean`, `scoreStd` (scores are `decision_function` values; lower means more anomalous).

The job status screen shows every score with this plain-language meaning and its direction.

| Metric             | Task                            | Meaning                                                                                                                                                | Better |
| ------------------ | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ | ------ |
| `accuracy`         | classification, semi-supervised | Share of test rows where the predicted class matches the true class.                                                                                   | Higher |
| `precisionMacro`   | classification, semi-supervised | Of the rows predicted as each class, the share that are correct — averaged equally over classes.                                                       | Higher |
| `recallMacro`      | classification, semi-supervised | Of the rows truly in each class, the share the model finds — averaged equally over classes.                                                            | Higher |
| `f1Macro`          | classification, semi-supervised | Harmonic mean of macro precision and recall: one balanced score for both.                                                                              | Higher |
| `labelledRows`     | semi-supervised                 | Rows with a non-empty label available for training and evaluation.                                                                                     | —      |
| `unlabelledRows`   | semi-supervised                 | Rows with an empty label that participate in training only.                                                                                            | —      |
| `iterations`       | semi-supervised                 | Self-training iterations performed before stopping.                                                                                                    | —      |
| `rmse`             | regression                      | Root mean squared error: typical prediction error in the units of the label column, weighting large errors more.                                       | Lower  |
| `mae`              | regression                      | Mean absolute error: average size of the prediction error, in the units of the label column.                                                           | Lower  |
| `r2`               | regression                      | Coefficient of determination: share of the variance of the label explained by the model — 1 is a perfect fit, 0 is no better than predicting the mean. | Higher |
| `silhouette`       | clustering                      | How much closer each row is to its own cluster than to the nearest other cluster, from −1 to 1.                                                        | Higher |
| `daviesBouldin`    | clustering                      | Average similarity between each cluster and its most similar cluster; lower is more separated.                                                         | Lower  |
| `calinskiHarabasz` | clustering                      | Ratio of between-cluster to within-cluster dispersion.                                                                                                 | Higher |
| `inertia`          | clustering                      | Sum of squared distances from each row to its cluster centre.                                                                                          | Lower  |
| `bic`              | clustering                      | Bayesian information criterion for the fitted mixture; penalises unnecessary components.                                                               | Lower  |
| `anomalyCount`     | anomaly detection               | Rows flagged as outliers (`predict == -1`).                                                                                                            | —      |
| `anomalyRate`      | anomaly detection               | Share of rows flagged as outliers.                                                                                                                     | —      |
| `scoreMean`        | anomaly detection               | Mean `decision_function` score across rows.                                                                                                            | —      |
| `scoreStd`         | anomaly detection               | Standard deviation of `decision_function` scores.                                                                                                      | —      |

## Charts

On `completed`, the job status screen also renders a task-specific **Model report** (charts) beneath
the metrics list. The selection, the bounded `charts` data contract, and the rendering rules are
normative in [`07-ui-ux-design/system/charts.md`](../07-ui-ux-design/system/charts.md).

| Task              | Charts                                                             |
| ----------------- | ------------------------------------------------------------------ |
| classification    | confusion matrix, per-class precision/recall/F1                    |
| semi_supervised   | classification charts + label coverage + self-training progress    |
| regression        | predicted vs actual, residuals vs predicted, residual distribution |
| clustering        | cluster scatter (PCA-2), cluster sizes, silhouette plot            |
| anomaly_detection | score distribution, anomaly vs normal split, score by row order    |

Charts augment the metrics; they never replace them. A chart whose data is absent is hidden, and
charts are not rendered for non-terminal or `failed` jobs. This is planned, not yet implemented.

## Edge cases

- Worker crash mid-fit: job remains non-terminal; operator restarts backend; a recovery pass re-queues jobs stuck in `training` longer than a timeout **or** marks them `failed` with `WORKER_INTERRUPTED` (choose fail-fast for MVP).
- All-NaN feature column → `VALIDATION_ERROR` during validating.
- Semi-supervised with no unlabelled rows → valid; behaves as supervised self-training.
- Semi-supervised with all labels empty or only one class → `INSUFFICIENT_LABELS`.
- Clustering where silhouette is undefined (all rows identical or fewer than two clusters) → omit `silhouette`; the job still completes with the remaining metrics.
- Unsupervised task with a label column → `INVALID_COLUMN_MAP`.

## Execution limits

The job lifecycle above assumes a healthy, single-attempt execution. The bounded, retryable
contract is normative in
[`FEATURE-17`](./FEATURE-17-Reliable-Job-Execution.md):

- At most `llm_MAX_CONCURRENT_JOBS` jobs execute at once (default 1); the rest stay `queued` and
  report their queue position.
- Each attempt is bounded by `llm_JOB_TIMEOUT_SECONDS` (default 1800); on expiry the job is
  `failed` with `JOB_TIMEOUT` and no artefact is written.
- A transient failure is retried up to `llm_JOB_MAX_ATTEMPTS` times (default 2) with
  exponential backoff and full jitter; the job reports the non-terminal status `retrying` between
  attempts. A deterministic failure is never retried.
- The poll payload exposes `attempts`, `maxAttempts`, and `nextAttemptAt`.

## Acceptance conditions

- Sample classification CSV completes with metrics and ONNX.
- Sample regression CSV completes with regression metrics.
- Sample semi-supervised CSV completes with labelled-only evaluation and ONNX.
- Sample clustering and anomaly CSVs complete with unsupervised metrics and ONNX.
- Forced exporter failure yields `failed` and no ONNX artefact row.
