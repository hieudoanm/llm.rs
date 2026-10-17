# llm-client (llm) — API Specification

## 1. Purpose

This document defines the REST API contract: endpoints, models, status codes, and errors.

The API must:

- Use RESTful conventions.
- Return the shared error envelope.
- Use UUIDs for resource identifiers.
- Accept jobs asynchronously (`202`).
- Always expose ONNX on successful jobs.
- Never return filesystem paths.

Normative OpenAPI: [`06-api-specification/backend/openapi.yaml`](./06-api-specification/backend/openapi.yaml).

## 2. Service Boundaries

| Service | OpenAPI                | Endpoints                                                                       |
| ------- | ---------------------- | ------------------------------------------------------------------------------- |
| Backend | `backend/openapi.yaml` | `/healthz`, `/readyz`, `/api/v1/datasets`, `/api/v1/algorithms`, `/api/v1/jobs` |

One deployable API. The Next.js app is not an BFF for training.

## 3. Base URL

- Local Compose: `http://127.0.0.1:8000`
- Health: `http://127.0.0.1:8000/healthz`
- Versioned API: `http://127.0.0.1:8000/api/v1`

Frontend: `http://127.0.0.1:3000`

## 4. Authentication

MVP is local single-user. No JWT. Bind to `127.0.0.1`. A future `Authorization` header is reserved and must not be required until a feature document says so.

## 5. Task types

`taskType` is one of:

| Value               | Label column                         | Evaluation                |
| ------------------- | ------------------------------------ | ------------------------- |
| `classification`    | Required                             | Hold-out, stratified      |
| `regression`        | Required                             | Hold-out                  |
| `semi_supervised`   | Required; empty cells are unlabelled | Hold-out on labelled rows |
| `clustering`        | Must be absent                       | In-sample                 |
| `anomaly_detection` | Must be absent                       | In-sample                 |
| `reinforcement_learning` | Must be absent; `rlColumns` required | Hold-out (episodes/transitions) |

`GET /api/v1/algorithms?taskType=` accepts any of these six values. `POST /api/v1/jobs` enforces the matching label rule (`INVALID_COLUMN_MAP`), semi-supervised minimums (`INSUFFICIENT_LABELS`), and hyperparameter ranges (`INVALID_HYPERPARAMETERS`). For `reinforcement_learning`, experience-column validation (`rlColumns`) is enforced and reward-type mismatches return `INVALID_LABEL_TYPE`.

## 6. Error Response Format

Every error response — application errors and Axum request-validation errors alike — uses the same flat envelope. The `error` object is at the **top level**; it is never nested under `detail`.

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {},
    "requestId": "uuid"
  }
}
```

- `code` — machine-readable code (see [`FEATURE-07`](./02-features/FEATURE-07-Error-Handling.md)).
- `message` — human-readable summary.
- `details` — optional structured context (omitted when absent).
- `requestId` — the request correlation id; the value of the `X-Request-Id` request header when supplied, otherwise a server-generated UUID.

Request-body validation failures (serde) return `400` with `code: "VALIDATION_ERROR"` and the per-field errors under `details.fields`. Unknown request fields (for example a misspelled `exportPkl`) are rejected with the same envelope: request models forbid extra fields.

Codes: see [`FEATURE-07`](./02-features/FEATURE-07-Error-Handling.md). `IDEMPOTENCY_KEY_EXISTS` (409) is returned when a job-create `Idempotency-Key` is reused with a different payload.

Additional codes introduced by the Phase 12 documents:

| Code                   | HTTP | When                                                        |
| ---------------------- | ---- | ----------------------------------------------------------- |
| `UNSUPPORTED_FILE_TYPE`| 400  | The upload is not a CSV by extension, content type, or bytes. |
| `MALFORMED_CSV`        | 400  | The bytes cannot be decoded as UTF-8 or parsed as delimited text. |
| `JOB_TIMEOUT`          | job `status=failed` | The attempt exceeded `llm_JOB_TIMEOUT_SECONDS`.       |
| `JOB_RETRY_EXHAUSTED`  | job `status=failed` | Every attempt failed with a transient error.           |
| `WORKER_INTERRUPTED`   | job `status=failed` | A crashed process left the job in a non-terminal status. |
| `DATABASE_ERROR`       | 500  | An unhandled database failure; the driver text is never returned. |
| `CONFLICT`             | 500  | An unhandled integrity (constraint) failure.                |

The `DATABASE_ERROR` and `CONFLICT` codes are produced by the
unhandled-exception handler (`src/api/errors.py`) so a driver string, DSN, or SQL
fragment never reaches the client; the raw text is logged server-side only.

## 7. Endpoints (summary)

| Method | Path                                          | Success | Purpose            |
| ------ | --------------------------------------------- | ------- | ------------------ |
| GET    | `/healthz`                                    | 200     | Liveness           |
| GET    | `/readyz`                                     | 200/503 | Postgres ping      |
| GET    | `/health/worker`                              | 200     | Worker status (running/queued/retrying vs limits) |
| POST   | `/api/v1/datasets`                            | 201     | Upload CSV         |
| GET    | `/api/v1/datasets/{datasetId}`                | 200     | Metadata + columns |
| GET    | `/api/v1/datasets/{datasetId}/insights`       | 200     | Column profile + preview |
| GET    | `/api/v1/algorithms`                          | 200     | Catalogue          |
| POST   | `/api/v1/jobs`                                | 202     | Enqueue training   |
| GET    | `/api/v1/jobs`                                | 200     | History (filter/sort/search) |
| GET    | `/api/v1/jobs/compare`                        | 200     | Compare 2–5 jobs   |
| GET    | `/api/v1/jobs/artefacts.zip`                  | 200     | Bundle for 1–10 jobs |
| GET    | `/api/v1/jobs/{jobId}`                        | 200     | Poll               |
| PATCH  | `/api/v1/jobs/{jobId}`                        | 200     | Update name/tags   |
| DELETE | `/api/v1/jobs/{jobId}`                        | 204     | P1 cleanup         |
| GET    | `/api/v1/jobs/{jobId}/artefacts/{artefactId}` | 200     | Download           |
| GET    | `/api/v1/jobs/{jobId}/artefacts.zip`          | 200     | Bundle for one job |

Route-ordering rule: the static paths `/api/v1/jobs/compare` and `/api/v1/jobs/artefacts.zip`
are registered **before** `/api/v1/jobs/{jobId}` so they are never captured by the UUID path
parameter.

### 7.1 Query parameters — `GET /api/v1/jobs`

| Parameter      | Type    | Default     | Notes                                                                  |
| -------------- | ------- | ----------- | ---------------------------------------------------------------------- |
| `limit`        | integer | `20`        | 1–100; otherwise `400 VALIDATION_ERROR`.                               |
| `cursor`       | string  | —           | Opaque, URL-safe token returned as `nextCursor`. It encodes the sort key, the last row's id, and a fingerprint of the active filter and sort set. A cursor issued for a different filter or sort is rejected with `400 VALIDATION_ERROR` (`details.field = "cursor"`). |
| `status`       | string  | —           | Comma-separated job statuses.                                          |
| `taskType`     | string  | —           | Comma-separated task types.                                            |
| `algorithmId`  | string  | —           | Comma-separated algorithm ids.                                         |
| `tag`          | string  | —           | Comma-separated tag slugs; **all** must match (AND).                   |
| `q`            | string  | —           | Free text over name, tags, and algorithm id. Max 200 characters.        |
| `createdFrom`  | ISO-8601| —           | Inclusive lower bound on `createdAt`.                                  |
| `createdTo`    | ISO-8601| —           | Exclusive upper bound on `createdAt`.                                  |
| `sort`         | string  | `createdAt:desc` | `createdAt`, `updatedAt`, `status`, `algorithmId`, each optionally suffixed `:asc` / `:desc`. |

Filters combine with AND. An unknown enum value, a malformed timestamp, a malformed cursor, an
unknown sort field, or `createdTo` earlier than `createdFrom` returns `400 VALIDATION_ERROR`
with `details.field` set to the offending parameter name.

Ordering is stable: rows with an equal sort key are ordered by `id DESC`. Sorting by `status`
uses the **lifecycle order** (`queued` → `validating` → `training` → `evaluating` →
`exporting` → `retrying` → `completed` → `failed`), not alphabetical order.

### 7.2 `PATCH /api/v1/jobs/{jobId}`

```json
{ "name": "baseline", "tags": ["baseline", "release"] }
```

Only `name` and `tags` are accepted; any other field returns `400 VALIDATION_ERROR`.
`name` is trimmed and stored as `null` when blank, and is limited to 120 characters. `tags`
accepts at most 10 entries of 1–32 characters each, normalised to lowercase and de-duplicated
case-insensitively. The response is the full job payload.

### 7.3 Job metadata and execution fields

Both the job list and the job detail carry the discovery and reliability fields:

| Field          | Type              | Meaning                                                                 |
| -------------- | ----------------- | ----------------------------------------------------------------------- |
| `name`         | string \| null    | User-set label; `null` when the job was never renamed.                  |
| `tags`         | array of string   | Lowercase slugs, at most 10.                                            |
| `attempts`     | integer           | Attempts already made, including the running one.                       |
| `maxAttempts`  | integer           | Total attempts allowed (`llm_JOB_MAX_ATTEMPTS`).                         |
| `nextAttemptAt`| datetime \| null  | When the next retry starts; `null` unless the job is `retrying`.        |
| `queuePosition`| integer \| null   | Queued jobs ahead of this one; `0` at the front of the queue, `null` for a job that is not `queued`. |

### 7.4 `GET /health/worker`

Reports the worker queue against the configured concurrency limit, so a stuck
or queued job is explainable without reading the process logs:

```json
{
  "maxConcurrentJobs": 2,
  "runningJobs": 1,
  "queuedJobs": 3,
  "retryingJobs": 0,
  "availableSlots": 1
}
```

- `runningJobs` counts the non-terminal statuses (`validating`, `training`,
  `evaluating`, `exporting`) — a slot is only busy while an attempt is in
  flight.
- `queuedJobs` counts `queued`, `retryingJobs` counts `retrying`.
- `availableSlots` is `max(0, maxConcurrentJobs - runningJobs)` and comes from
  `llm_MAX_CONCURRENT_JOBS`.

The endpoint is tagged `health` and lives outside `/api/v1`. It is a
diagnostic aid, not part of the training contract.

## 8. Job create body

Supervised or semi-supervised:

```json
{
  "datasetId": "uuid",
  "featureColumns": ["col_a", "col_b"],
  "labelColumn": "target",
  "taskType": "classification",
  "algorithmId": "logistic_regression",
  "hyperparameters": {},
  "splitRatio": 0.2,
  "randomSeed": 42,
  "exportPkl": false,
  "exportJoblib": false
}
```

Clustering or anomaly detection: omit `labelColumn` (or send `null`) and supply the task's hyperparameters. For reinforcement learning, omit `labelColumn` (or send `null`) and supply `rlColumns` plus FQI hyperparameters.

Extra fields are rejected: the body accepts only the fields above — there is no `exportFormats` array. Requested export formats are the individual boolean flags `exportPkl` and `exportJoblib`; ONNX is always produced.

```json
{
  "datasetId": "uuid",
  "featureColumns": ["col_a", "col_b"],
  "taskType": "clustering",
  "algorithmId": "kmeans",
  "hyperparameters": { "n_clusters": 4 }
}
```

Headers: optional `Idempotency-Key`, `X-Request-Id`.

`Idempotency-Key` (optional, 1–255 chars of `[a-zA-Z0-9_-]`) makes job creation safe to retry. The first request with a key creates the job and caches its `202` body; a repeat with the same key and an identical payload replays that body (same `jobId`). The same key with a different payload returns `409 IDEMPOTENCY_KEY_EXISTS`. Keys expire after 24 hours. When the header is absent (or blank), each request creates a new job.

## 9. Job poll body (completed)

```json
{
  "jobId": "uuid",
  "datasetId": "uuid",
  "status": "completed",
  "taskType": "classification",
  "algorithmId": "logistic_regression",
  "featureColumns": ["col_a"],
  "labelColumn": "target",
  "hyperparameters": {},
  "metrics": {
    "accuracy": 0.91,
    "precisionMacro": 0.9,
    "recallMacro": 0.9,
    "f1Macro": 0.9
  },
  "charts": {
    "classes": ["0", "1"],
    "confusionMatrix": [
      [45, 5],
      [3, 47]
    ],
    "perClass": [
      {
        "class": "0",
        "precision": 0.94,
        "recall": 0.9,
        "f1": 0.92,
        "support": 50
      },
      {
        "class": "1",
        "precision": 0.9,
        "recall": 0.94,
        "f1": 0.92,
        "support": 50
      }
    ],
    "featureImportance": {
      "feature": ["col_a", "col_b"],
      "importance": [0.62, 0.38],
      "method": "coef_"
    }
  },
  "artefacts": [
    {
      "artefactId": "uuid",
      "format": "onnx",
      "filename": "{jobId}.onnx",
      "byteSize": 12345
    }
  ],
  "libraryVersions": {},
  "error": null,
  "createdAt": "2026-10-17T00:00:00Z",
  "updatedAt": "2026-10-17T00:00:10Z"
}
```

Clustering poll body (unsupervised tasks return `labelColumn: null`):

```json
{
  "jobId": "uuid",
  "datasetId": "uuid",
  "status": "completed",
  "taskType": "clustering",
  "algorithmId": "kmeans",
  "featureColumns": ["col_a", "col_b"],
  "labelColumn": null,
  "hyperparameters": { "n_clusters": 4 },
  "metrics": {
    "silhouette": 0.62,
    "daviesBouldin": 0.71,
    "calinskiHarabasz": 183.4,
    "inertia": 214.9
  },
  "charts": {
    "clusterSizes": [52, 48, 50, 50],
    "projection": {
      "x": [1.2, -0.5, 0.8, ...],
      "y": [0.3, 1.1, -0.7, ...],
      "cluster": [0, 1, 2, ...]
    },
    "silhouetteSamples": {
      "value": [0.72, 0.65, 0.81, ...],
      "cluster": [0, 1, 2, ...]
    }
  },
  "artefacts": [
    {
      "artefactId": "uuid",
      "format": "onnx",
      "filename": "{jobId}.onnx",
      "byteSize": 12345
    }
  ],
  "libraryVersions": {},
  "error": null,
  "createdAt": "2026-10-17T00:00:00Z",
  "updatedAt": "2026-10-17T00:00:10Z"
}
```

Semi-supervised metrics use the classification keys plus `labelledRows`, `unlabelledRows`, and `iterations`. Anomaly detection returns `anomalyCount`, `anomalyRate`, `scoreMean`, and `scoreStd`. Reinforcement learning returns `meanReward`, `meanQ`, `tdErrorMean`, `tdErrorStd`, and `actions` (plus `episodeCount` and `meanEpisodeReturn` when an episode column was declared). See [`FEATURE-04`](./02-features/FEATURE-04-Training-Job.md#metrics).

### 10. Charts payload

The `charts` field contains bounded evaluation data for the Model report visualisation. The contract is defined in [`07-ui-ux-design/system/charts.md`](./07-ui-ux-design/system/charts.md#41-data-contract-table). The field is present in all completed job responses; it is `null` for jobs completed before the charts feature was added or if chart collection failed.

## 11. serde Models (Request/Response Schemas)

The following models define the type-safe contracts used by the Axum backend. They are implemented in `crates/llm-api/src/api/schemas/` (grouped by domain — `shared.py`, `health.py`, `datasets.py`, `algorithms.py`, `jobs.py`, and re-exported from `schemas/__init__.py`) and must be kept in sync with this specification and the OpenAPI YAML.

### 11.1 Health & Readiness

```rust
# GET /healthz
class HealthResponse(BaseModel):
    status: str = "ok"

# GET /readyz
class ReadyResponse(BaseModel):
    status: str = "ok"

# GET /health/worker
class WorkerStatusResponse(BaseModel):
    maxConcurrentJobs: int
    runningJobs: int
    queuedJobs: int
    retryingJobs: int
    availableSlots: int
```

### 11.2 Dataset Models

```rust
# POST /api/v1/datasets -> 201, GET /api/v1/datasets/{datasetId} -> 200
class ColumnPreview(BaseModel):
    name: str
    inferredDtype: str
    nullCount: int
    sampleValues: list[Any]

class DatasetResponse(BaseModel):
    datasetId: UUID
    sha256: str
    filename: str
    byteSize: int
    columns: list[ColumnPreview]
    rowCount: int
    createdAt: datetime
```

### 11.3 Algorithm Models

```rust
# GET /api/v1/algorithms?taskType=... -> 200
class AlgorithmResponse(BaseModel):
    algorithmId: str
    name: str
    taskType: TaskType
    family: Literal["llm"]
    description: str
    defaultHyperparameters: dict[str, Any]
    exports: list[ArtefactFormat]

class AlgorithmsListResponse(BaseModel):
    algorithms: list[AlgorithmResponse]
```

### 11.4 Job Models

```rust
# Shared enums
class TaskType(StrEnum):
    CLASSIFICATION = "classification"
    REGRESSION = "regression"
    SEMI_SUPERVISED = "semi_supervised"
    CLUSTERING = "clustering"
    ANOMALY_DETECTION = "anomaly_detection"
    REINFORCEMENT_LEARNING = "reinforcement_learning"

class JobStatus(StrEnum):
    QUEUED = "queued"
    VALIDATING = "validating"
    TRAINING = "training"
    EVALUATING = "evaluating"
    EXPORTING = "exporting"
    RETRYING = "retrying"
    COMPLETED = "completed"
    FAILED = "failed"

class ArtefactFormat(StrEnum):
    ONNX = "onnx"
    PKL = "pkl"
    JOBLIB = "joblib"

# RL columns specification
class RLColumns(BaseModel):
    actionColumn: str
    rewardColumn: str
    nextStateColumns: list[str] = Field(default_factory=list)
    doneColumn: str | None = None
    episodeColumn: str | None = None

    model_config = ConfigDict(extra="forbid")

# POST /api/v1/jobs request
class CreateJobRequest(BaseModel):
    datasetId: UUID
    featureColumns: list[str] = Field(min_length=1)
    taskType: TaskType
    algorithmId: str
    labelColumn: str | None = None
    rlColumns: RLColumns | None = None
    hyperparameters: dict[str, Any] = Field(default_factory=dict)
    splitRatio: float = Field(default=0.2, gt=0, lt=1)
    randomSeed: int = Field(default=42, ge=0)
    exportPkl: bool = False
    exportJoblib: bool = False

    model_config = ConfigDict(extra="forbid")

    @model_validator(mode="after")
    def validate_label_rules(self) -> "CreateJobRequest":
        # Enforced in validate_job_request(); documented here for OpenAPI
        if self.taskType in {TaskType.CLASSIFICATION, TaskType.REGRESSION, TaskType.SEMI_SUPERVISED}:
            if not self.labelColumn:
                raise ValueError(f"labelColumn required for {self.taskType}")
        if self.taskType in {TaskType.CLUSTERING, TaskType.ANOMALY_DETECTION, TaskType.REINFORCEMENT_LEARNING}:
            if self.labelColumn:
                raise ValueError(f"{self.taskType} must not set labelColumn")
        if self.taskType == TaskType.REINFORCEMENT_LEARNING:
            if not self.rlColumns:
                raise ValueError("rlColumns required for reinforcement_learning")
        else:
            if self.rlColumns:
                raise ValueError("rlColumns only allowed for reinforcement_learning")
        return self

# POST /api/v1/jobs -> 202 response
class JobAcceptedResponse(BaseModel):
    jobId: UUID
    status: JobStatus
    datasetId: UUID
    taskType: TaskType
    algorithmId: str
    createdAt: datetime
    name: str | None = None
    tags: list[str] = Field(default_factory=list)
    attempts: int = 0
    maxAttempts: int = 2
    nextAttemptAt: datetime | None = None
    queuePosition: int | None = None

# GET /api/v1/jobs -> 200 response
class JobListResponse(BaseModel):
    jobs: list[JobAcceptedResponse]  # Uses JobAccepted fields
    nextCursor: str | None = None
    hasMore: bool

# PATCH /api/v1/jobs/{jobId} request
class UpdateJobMetadataRequest(BaseModel):
    name: str | None = Field(default=None, max_length=120)
    tags: list[str] = Field(default_factory=list, max_length=10)

    model_config = ConfigDict(extra="forbid")

# GET /api/v1/jobs/compare -> 200 response
class ComparisonDataset(BaseModel):
    datasetId: UUID
    filename: str
    rowCount: int
    columnCount: int

class ComparisonJob(BaseModel):
    jobId: UUID
    name: str | None = None
    tags: list[str] = Field(default_factory=list)
    status: JobStatus
    taskType: TaskType
    algorithmId: str
    createdAt: datetime
    dataset: ComparisonDataset
    featureColumns: list[str]
    labelColumn: str | None = None
    splitRatio: float
    randomSeed: int
    hyperparameters: dict[str, Any] = Field(default_factory=dict)
    metrics: dict[str, float | int] | None = None
    artefactFormats: list[ArtefactFormat] = Field(default_factory=list)
    error: JobErrorDetail | None = None

class ComparisonWarning(BaseModel):
    code: str
    message: str

class JobComparisonResponse(BaseModel):
    jobs: list[ComparisonJob]
    warnings: list[ComparisonWarning] = Field(default_factory=list)

# GET /api/v1/datasets/{datasetId}/insights -> 200 response
class NumericSummary(BaseModel):
    min: float | None = None
    max: float | None = None
    mean: float | None = None
    std: float | None = None
    median: float | None = None

class ValueCount(BaseModel):
    value: str
    count: int

class ColumnInsight(BaseModel):
    name: str
    inferredType: str  # numeric | boolean | categorical | text | datetime
    missingCount: int
    missingRate: float
    uniqueCount: int
    numeric: NumericSummary | None = None
    topValues: list[ValueCount] = Field(default_factory=list)
    warnings: list[str] = Field(default_factory=list)

class DatasetWarning(BaseModel):
    code: str
    column: str | None = None
    message: str

class DatasetPreview(BaseModel):
    offset: int
    limit: int
    totalRows: int
    columns: list[str]
    rows: list[list[str | None]]

class DatasetInsightsResponse(BaseModel):
    datasetId: UUID
    sha256: str
    filename: str
    rowCount: int
    columnCount: int
    totalMissingCells: int
    missingRate: float
    columns: list[ColumnInsight]
    warnings: list[DatasetWarning] = Field(default_factory=list)
    preview: DatasetPreview

# GET /api/v1/jobs/{jobId} -> 200 response (full job detail)
class JobErrorDetail(BaseModel):
    code: str
    message: str

class ArtefactResponse(BaseModel):
    artefactId: UUID
    format: ArtefactFormat
    filename: str
    byteSize: int

class JobResponse(BaseModel):
    jobId: UUID
    status: JobStatus
    datasetId: UUID
    taskType: TaskType
    algorithmId: str
    createdAt: datetime
    updatedAt: datetime
    featureColumns: list[str]
    labelColumn: str | None = None
    rlColumns: RLColumns | None = None
    hyperparameters: dict[str, Any] = Field(default_factory=dict)
    metrics: dict[str, float | int] | None = None
    charts: dict[str, Any] | None = None
    artefacts: list[ArtefactResponse] = Field(default_factory=list)
    libraryVersions: dict[str, str] | None = None
    error: JobErrorDetail | None = None
    name: str | None = None
    tags: list[str] = Field(default_factory=list)
    attempts: int = 0
    maxAttempts: int = 2
    nextAttemptAt: datetime | None = None
    queuePosition: int | None = None
```

### 11.5 Error Envelope

```rust
class ErrorDetail(BaseModel):
    code: str
    message: str
    details: dict[str, Any] | None = None
    requestId: UUID

class ErrorEnvelope(BaseModel):
    error: ErrorDetail
```

### 11.6 Validation Rules Summary

| Field | Rule |
|-------|------|
| `featureColumns` | Min 1, all must exist in dataset header |
| `labelColumn` | Required for classification, regression, semi_supervised; forbidden for clustering, anomaly_detection, reinforcement_learning |
| `rlColumns` | Required for reinforcement_learning; forbidden otherwise |
| `splitRatio` | 0 < splitRatio < 1 |
| `randomSeed` | >= 0 |
| `hyperparameters` | Validated per-algorithm in `validate_hyperparameters()` |
| `exportPkl`/`exportJoblib` | Optional extras; ONNX is always produced |
| _any other field_ | Rejected (`VALIDATION_ERROR`); request models forbid extra fields |
| `name` (PATCH) | Optional, max 120 characters; blank is stored as `null` |
| `tags` (PATCH) | At most 10 tags, 1–32 characters each, lowercase-normalised, de-duplicated |

---

## 12. Artefact bundle responses

Both bundle endpoints respond with a binary ZIP, not with the JSON envelope:

| Property                | Value                                                     |
| ----------------------- | --------------------------------------------------------- |
| `Content-Type`          | `application/zip`                                          |
| `Content-Disposition`   | `attachment; filename="<name>.zip"`                       |
| `Content-Length`        | The archive size (the archive is buffered before streaming) |
| `X-llm-Missing-Files`   | Comma-separated paths omitted because the file was absent  |

`GET /api/v1/jobs/{jobId}/artefacts.zip` → `<jobId>.zip`, containing `<jobId>/` with the job's
artefacts, `metrics.json` (when metrics exist), and `manifest.json`.

`GET /api/v1/jobs/artefacts.zip?ids=` → `llm-jobs.zip`, containing one `<jobId>/` directory per
job that has artefacts, plus a top-level `skipped.json` listing the omitted jobs and the reason
(`NO_ARTEFACTS` or `NOT_COMPLETED`).

---

## 13. Interactive API reference

The backend serves its own generated documentation:

| Path            | Content                                                     |
| --------------- | ----------------------------------------------------------- |
| `/docs`         | Swagger UI — the interactive, "try it out" reference.        |
| `/redoc`        | ReDoc — the reference-oriented rendering.                    |
| `/openapi.json` | The raw OpenAPI 3.1 document.                               |

Rules:

- These paths are **outside** the `/api/v1` base path and are not part of the training contract.
- They are enabled by default and can be disabled with `llm_ENABLE_INTERACTIVE_DOCS=false`.
- The committed copy of the schema is
  [`06-api-specification/backend/openapi.yaml`](./06-api-specification/backend/openapi.yaml). It is
  regenerated from the Axum application with `utoipa`. CI fails when the
  committed file differs from the generated one (NFR-073).
- Every route is tagged (`health`, `datasets`, `algorithms`, `jobs`) so the reference groups them
  by resource.
- The application links to the reference from its footer, using `NEXT_PUBLIC_API_DOCS_URL`
  (default `${NEXT_PUBLIC_API_URL}/docs`).

### 13.1 Regeneration workflow

```bash
# Regenerate the backend OpenAPI YAML from the Axum app (utoipa)
cargo run --bin export-openapi --manifest-path crates/llm-api/Cargo.toml

# Verify no drift (CI uses this)
cargo run --bin export-openapi --manifest-path crates/llm-api/Cargo.toml -- --check
```

### 13.2 Troubleshooting

| Symptom | Fix |
| ------- | --- |
| CI fails at "Check the committed OpenAPI document" (`--check` reports drift) | Regenerate the YAML from the app (`crates/llm-api/.../export_openapi.py`), regenerate the derivatives (`docs/06-api-specification/openapi.py` if needed), and commit all updated OpenAPI files. |
| `openapi.yaml` is out of sync after changing routes/schemas | Update `src/api/schemas.py` / routers / `main.py`, regenerate the schema, and update this document and the frontend client in the same change (per AGENTS.md). |
| `GET /openapi.json` 404s locally | Ensure `llm_ENABLE_INTERACTIVE_DOCS` is not `false`. The setting comes from `src/config.py` and is controlled via the environment. |

---

## Cross-references

- Requirements: [`01-requirements.md`](01-requirements.md)
- Architecture: [`04-architecture.md`](04-architecture.md)
