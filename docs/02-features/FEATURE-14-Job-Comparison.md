# FEATURE-14: Job Comparison

## Priority

P1 (FR-130..FR-134)

## Description

Training is iterative: the user changes one hyperparameter, runs again, and wants to know whether
the new run is actually better. Today that means opening each job tab and reading the numbers
from memory. This feature adds a **comparison view** that puts up to five jobs side by side, with
the metric differences highlighted.

Comparison is read-only. It reads job rows and metrics; it never re-runs training and never
mutates a job.

## User-visible behaviour

### Selecting jobs to compare

- The jobs table row menu has **Compare**; selecting it adds the job to the current selection.
- A persistent selection bar appears at the bottom of the jobs page showing the chosen jobs with
  their status badges and a remove action per chip.
- From the job detail page, **Compare with…** opens a picker listing recent jobs of the same task
  type first, then all other jobs.
- Selection is capped at **5** jobs; the sixth attempt shows a notice explaining the cap.
- The selection is stored in the URL (`/jobs?compare=<id>,<id>`) so a comparison can be
  bookmarked.

### The comparison view

Route: `/jobs/compare?ids=<id>,<id>[,<id>…]`.

- A column per job, a row per attribute.
- **Header row** — job name (or `Job <short id>` when unnamed), status badge, algorithm, task
  type, and a link that opens the job detail page.
- **Configuration rows** — dataset name, row count, feature column count, label column, split
  ratio, random seed, and the algorithm's hyperparameters (only keys that differ across the
  compared jobs are highlighted).
- **Metrics rows** — every metric key returned by the selected jobs, one row per metric. Metrics
  present on some jobs but not others are shown with an em dash for the missing job.
- **Artefact row** — the exported formats per job.
- **Highlight** — the best value per metric is marked. "Best" means: **higher is better** for
  accuracy, precision, recall, F1, R², silhouette (negated), and negative Davies–Bouldin;
  **lower is better** for RMSE, MAE, Davies–Bouldin, inertia, BIC, and anomaly rate. Metrics with
  an unknown direction are never marked.
- **Difference column** — for each metric row, the spread between the best and worst job, shown as
  a signed delta when at least two jobs share the metric.

### Warnings

The comparison view must not imply that unlike jobs are comparable. It surfaces a warning banner
when:

- The compared jobs use **different task types** — metrics are not comparable at all.
- The compared jobs use **different datasets** — metric differences may come from the data, not
  the configuration.
- The compared jobs use **different feature columns** — the models are not solving the same
  problem.
- One or more jobs are **not completed** — its metrics are empty and the row is marked
  "not finished"; the job is still shown so the user can see the failure reason.

## API

| Method | Path                        | Success | Purpose                                     |
| ------ | --------------------------- | ------- | ------------------------------------------- |
| GET    | `/api/v1/jobs/compare`      | 200     | Compare 2–5 jobs by id                     |

Query: `ids` — comma-separated UUIDs, **2 to 5** entries. Duplicate ids are collapsed.

Response:

```json
{
  "jobs": [
    {
      "jobId": "uuid",
      "name": "baseline",
      "tags": ["baseline"],
      "status": "completed",
      "taskType": "classification",
      "algorithmId": "logistic_regression",
      "createdAt": "2026-10-21T10:00:00Z",
      "dataset": { "datasetId": "uuid", "filename": "iris.csv", "rowCount": 150, "columnCount": 5 },
      "featureColumns": ["a", "b"],
      "labelColumn": "target",
      "splitRatio": 0.2,
      "randomSeed": 42,
      "hyperparameters": { "C": 1.0 },
      "metrics": { "accuracy": 0.97 },
      "artefactFormats": ["onnx"],
      "error": null
    }
  ],
  "warnings": [
    { "code": "DIFFERENT_DATASETS", "message": "Compared jobs use different datasets." }
  ]
}
```

The `warnings` array is always present and may be empty. Warning codes:

| Code                    | Meaning                                                       |
| ----------------------- | ------------------------------------------------------------- |
| `DIFFERENT_TASKS`       | Jobs use different task types — metrics are not comparable.   |
| `DIFFERENT_DATASETS`    | Jobs use different datasets.                                  |
| `DIFFERENT_FEATURES`    | Jobs use different feature column sets.                       |
| `INCOMPLETE_JOBS`       | At least one job is not `completed` and has no metrics.        |

## Edge cases

| Case                                | Behaviour                                                        |
| ----------------------------------- | ----------------------------------------------------------------- |
| Fewer than 2 ids                    | `400 VALIDATION_ERROR` (`details.field = "ids"`, `details.min = 2`) |
| More than 5 ids                     | `400 VALIDATION_ERROR` (`details.field = "ids"`, `details.max = 5`) |
| Malformed UUID in `ids`             | `400 VALIDATION_ERROR` (`details.field = "ids"`)                   |
| One id does not exist               | `404 JOB_NOT_FOUND` with `details.missingJobIds = ["<id>"]`       |
| Compared job is `failed`            | Returned with `status = "failed"`, `metrics = null`, and its `error` |
| Compared job is still running       | Returned with `status` and `metrics = null`; a warning is raised |
| Jobs use different task types       | `warnings` contains `DIFFERENT_TASKS`; metric rows are only shown for metrics present on ≥ 2 jobs |
| A metric appears on only one job    | The row is hidden by default and revealed by "Show all metrics"  |
| Metric direction unknown            | The row is never marked as best/worst                              |
| A job is deleted mid-comparison     | The next request returns `404 JOB_NOT_FOUND` for that id          |

## Acceptance conditions

- `GET /api/v1/jobs/compare?ids=<a>,<b>` returns `200` with one entry per job, in the requested
  order.
- Comparing jobs from two different datasets yields `warnings` containing `DIFFERENT_DATASETS`.
- Comparing a completed job with a failed job returns `200`; the failed job carries its `error`
  payload and `metrics` is `null`.
- The comparison view renders metric rows with a "best value" marker using the documented
  direction rules, and never marks a row whose direction is unknown.
- The selection survives a page reload via the URL.
- Comparison does not change any job's status, metrics, or artefacts.

**See also:** [`FEATURE-06`](./FEATURE-06-Job-History.md),
[`FEATURE-13`](./FEATURE-13-Job-Discovery.md),
[`06-api-specification.md`](../06-api-specification.md).
