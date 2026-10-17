# SQLite schema (llm)

Authoritative DDL: [`schema.sql`](./schema.sql).

Alembic migrations in the backend package must produce the same objects. This file is the reviewable contract; migrations are the runtime source.

## Tables

| Table              | Purpose                       |
| ------------------ | ----------------------------- |
| `datasets`         | Uploaded CSV metadata         |
| `dataset_insights` | Per-dataset column profile    |
| `jobs`             | Training lifecycle            |
| `job_metrics`      | Task-specific metrics JSON    |
| `artefacts`        | Exported model files          |
| `idempotency_keys` | Cached `POST /jobs` responses |

`jobs.label_column` is nullable. A table-level `CHECK` requires it for `classification`, `regression`, and `semi_supervised`, and requires `NULL` for `clustering`, `anomaly_detection`, and `reinforcement_learning`. `job_metrics.metrics` remains JSONB because the key set depends on `task_type`. `jobs.rl_columns` is JSONB and required when `task_type = 'reinforcement_learning'` (validated by the application).

## Job metadata, retries, and the `retrying` status

`jobs.name` is an optional human-readable label (≤ 120 characters, `NULL` when blank).
`jobs.tags` is a `TEXT[]` of lowercase tag slugs, at most 10 per job, empty by default. The
application normalises tags (trim, lowercase, case-insensitive de-duplication) and rejects
values outside 1–32 characters or containing characters other than letters, digits, `-`, `_`,
`.`, and spaces.

`jobs.attempts` counts the attempts that have started (`0` before the worker claims the job).
`jobs.max_attempts` is the per-job attempt budget captured at creation time (default 2), so a
later configuration change does not alter an in-flight job's budget. `jobs.next_attempt_at` is
set while the job waits for a retry and is `NULL` otherwise.

`jobs.status` accepts `retrying` in addition to the existing statuses. `retrying` is
non-terminal: the worker moves the job back to `queued` after the backoff delay and starts the
next attempt.

## Dataset insights

`dataset_insights` stores the profile computed once per dataset, keyed by `dataset_id`. The
`columns` JSONB holds one entry per dataset column (inferred type, missing counts, distinct
count, numeric summary, most frequent values, per-column warning codes). `warnings` JSONB holds
the dataset-level warning list. Preview rows are **not** stored — they are read from the CSV on
demand and bounded by the caller-supplied limit and offset.

Insights are advisory. Nothing in this table gates job creation.

## Indexes

- `jobs (created_at DESC, id DESC)` for history.
- `jobs (status)` for the worker claim query.
- `jobs` GIN on `tags` for the tag filter (`@>` containment).
- `jobs (algorithm_id)` for the algorithm filter.
- `jobs (lower(name))` for the case-insensitive name search.
- `artefacts (job_id)`.
- `idempotency_keys (key)` primary key.

## Idempotency keys

`idempotency_keys` caches `POST /api/v1/jobs` responses keyed by the request's
`Idempotency-Key` header. `request_hash` is the SHA-256 of the canonicalised
request body: a replay with the same key and hash returns the cached
`response_status`/`response_body`; a different hash returns `409
IDEMPOTENCY_KEY_EXISTS`. Rows older than `expires_at` (24 hours after
`created_at`) are ignored and may be reused. `job_id` is `SET NULL` on job
delete, so a deleted job's key becomes a miss rather than an error.

## Worker claim (sketch)

```sql
UPDATE jobs
SET status = 'validating', updated_at = now()
WHERE id = (
  SELECT id FROM jobs
  WHERE status = 'queued'
  ORDER BY created_at
  FOR UPDATE SKIP LOCKED
  LIMIT 1
)
RETURNING *;
```
