# llm-client (llm) — Data Model

## 1. Purpose

This document defines the SQLite 18 data model: entities, relationships, constraints, and job state transitions.

The model must support:

- Dataset metadata and content hashes.
- Per-dataset column profiles (insights) computed once at upload time.
- Training job lifecycle across six task types.
- Column mapping (X and conditional Y), algorithm, hyperparameters, and **experience columns** for reinforcement learning (`rl_columns`).
- Job metadata (`name`, `tags`) and retry bookkeeping (`attempts`, `max_attempts`, `next_attempt_at`).
- Task-specific metrics and library versions.
- Artefact records (ONNX required).
- Optional idempotency keys for job create.

Bytes are **not** stored in SQLite. Paths on disk are stored as relative keys, never as client-controlled absolute paths.

## 2. Service Data Ownership

| Store      | File                                                           | Tables / objects                                                   |
| ---------- | -------------------------------------------------------------- | ------------------------------------------------------------------ |
| SQLite | [`sqlite/README.md`](./05-data-model/sqlite/README.md) | `datasets`, `dataset_insights`, `jobs`, `job_metrics`, `artefacts`, `idempotency_keys` |
| Volume     | —                                                              | `datasets/*.csv`, `artefacts/{jobId}/*`                            |

## 3. Design Principles

1. **UUID primary keys** (UUID v7 preferred when the driver supports it; otherwise UUIDv4 is acceptable if documented).
2. **Parameterised SQL only.**
3. **JSONB** for hyperparameters, metrics, column arrays, library versions.
4. **Status CHECK constraints** for job lifecycle.
5. **Artefact format CHECK** (`onnx`, `pkl`, `joblib`).
6. **Relative storage keys** (`datasets/{id}.csv`).
7. **ONNX uniqueness:** at most one `onnx` artefact per job (`UNIQUE (job_id, format)`).
8. **Conditional label:** `label_column` is required for `classification`, `regression`, and `semi_supervised`, and must be `NULL` for `clustering`, `anomaly_detection`, and `reinforcement_learning`.

## 4. Relationships

- Dataset 1—* Jobs
- Dataset 1—0..1 Dataset insights
- Job 1—0..1 Job metrics (one metrics JSON row)
- Job 1—* Artefacts (1..n on success; ≥1 onnx)
- Idempotency key 0..1—* Job (job create)

## 5. State transitions

```
queued → validating → training → evaluating → exporting → completed
   ↑          ↘        ↘          ↘            ↘         ↘ failed
   └──── retrying (transient failure; attempts < max_attempts)
```

Any non-terminal state may transition to `failed`. Terminal states do not leave `completed` or `failed`.

`retrying` is non-terminal: the job waits until `next_attempt_at`, returns to `queued`, and is
claimed again. `attempts` is incremented when an attempt starts. A job that exhausts
`max_attempts` transitions to `failed` with the error code `JOB_RETRY_EXHAUSTED`.

## 6. Key workflows

### 6.1 Upload

`INSERT INTO datasets (id, original_filename, sha256, byte_size, storage_key, columns, row_count, created_at)`.

### 6.2 Job create

`INSERT INTO jobs (..., status='queued')`. Optional `idempotency_keys` row with cached response.

`label_column` is `NULL` for `clustering`, `anomaly_detection`, and `reinforcement_learning`; the table-level constraint rejects any other combination. For `reinforcement_learning`, `rl_columns` (JSONB) must be present and valid (action/reward and either next-state columns or episode column).

### 6.3 Complete

Insert `job_metrics` (with optional `charts` for result visualisation), insert `artefacts` (onnx required), `UPDATE jobs SET status='completed'`.

### 6.4 Task-specific payloads

`jobs.task_type` determines the mapping, split, and metrics contract:

| Task                  | `label_column` | Split                                                          | `job_metrics.metrics` keys                                                 |
| --------------------- | -------------- | -------------------------------------------------------------- | -------------------------------------------------------------------------- |
| `classification`      | Required       | Hold-out, stratified                                           | `accuracy`, `precisionMacro`, `recallMacro`, `f1Macro`                     |
| `regression`          | Required       | Hold-out                                                       | `rmse`, `mae`, `r2`                                                        |
| `semi_supervised`     | Required       | Hold-out on labelled rows; unlabelled rows in train            | classification keys + `labelledRows`, `unlabelledRows`, `iterations`       |
| `clustering`          | `NULL`         | None (in-sample)                                               | `silhouette`, `daviesBouldin`, `calinskiHarabasz`, plus `inertia` or `bic` |
| `anomaly_detection`   | `NULL`         | None (in-sample)                                               | `anomalyCount`, `anomalyRate`, `scoreMean`, `scoreStd`                     |
| `reinforcement_learning` | `NULL`         | Hold-out of episodes (or transitions when no episode column)    | `meanReward`, `meanQ`, `tdErrorMean`, `tdErrorStd`, `actions`, plus `episodeCount`, `meanEpisodeReturn` when episode column exists |

`job_metrics.metrics` stays JSONB so task-specific keys do not require a schema change. `split_ratio` is recorded for every job but is ignored by unsupervised tasks.

### 6.6 Job metadata and discovery columns

`jobs.name` (nullable, ≤ 120 characters) and `jobs.tags` (`TEXT[]`, ≤ 10 lowercase slugs) are
edited through `PATCH /api/v1/jobs/{jobId}`. They are metadata only: the training configuration
of a job (`feature_columns`, `label_column`, `rl_columns`, `hyperparameters`, `split_ratio`,
`random_seed`, export flags) is immutable after creation.

The tag filter uses GIN containment (`tags @> ARRAY[...]`), so **all** requested tags must be
present. The free-text search matches `lower(name)`, the tag slugs, and `algorithm_id`.

### 6.7 Dataset insights

`INSERT INTO dataset_insights (...)` runs once, inside the same transaction as the dataset row.
The `columns` JSON holds one entry per dataset column; `warnings` JSON holds the
dataset-level warning list. Preview rows are read from the CSV on demand and are not stored.

Insights are advisory — no insight blocks job creation.

### 6.8 Migration note

Existing rows are compatible: every pre-Phase-9 job has a non-null label. The local schema is
defined in `crates/llm-db/src/schema.rs` as a `SCHEMA` constant, applied idempotently at startup.
There are no separate migration files — schema changes are made by editing the `SCHEMA` constant
and redeploying.

## 7. Identifier and time rules

- Timestamps: RFC3339 `TEXT` UTC.
- Hashes: lowercase hex SHA-256.
- `library_versions` example: `{ "rust": "1.x", "llm": "...", "onnx": "...", "onnx-protobuf": "..." }`.

## Cross-references

- Architecture: [`04-architecture.md`](04-architecture.md)
- Schema: [`05-data-model/sqlite/schema.sql`](./05-data-model/sqlite/schema.sql)
