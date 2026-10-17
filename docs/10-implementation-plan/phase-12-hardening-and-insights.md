# Phase 12: Hardening & Insights

Hardens the platform against the failure modes a real user hits on a laptop, and adds the
discovery, comparison, and insight features on top of the existing workflow. No new learning
task type, no new algorithm, and no new dependency.

The phase is specified **before** it is implemented. The governing documents are
[`FEATURE-13`](../02-features/FEATURE-13-Job-Discovery.md) through
[`FEATURE-18`](../02-features/FEATURE-18-Client-Resilience-and-Guidance.md), plus the new
environment-variable reference
([`11-deployment/environment-variables.md`](../11-deployment/environment-variables.md)).

## Scope

1. **Central configuration** — add `crates/mlrs-api/src/config.py`: a single settings module
   that reads every environment variable once, validates it, and falls back to a documented
   default with one startup warning. Move the CORS allow-list out of `main.py` into
   `mlrs_CORS_ORIGINS`; add `mlrs_CORS_ALLOW_CREDENTIALS`, `mlrs_ENABLE_INTERACTIVE_DOCS`, and
   `mlrs_LOG_LEVEL`. Wire the new variables through `docker-compose.yaml` and `.env.example`.
2. **Upload hardening** — validate the declared content type, the extension, and the sniffed
   bytes; reject binary payloads renamed to `.csv` with `UNSUPPORTED_FILE_TYPE`; reject
   undecodable bytes with `MALFORMED_CSV`; reject empty or duplicate header names with
   `VALIDATION_ERROR`.
3. **Worker reliability** — bound concurrency with `mlrs_MAX_CONCURRENT_JOBS`; bound each attempt
   with `mlrs_JOB_TIMEOUT_SECONDS`; retry transient failures up to `mlrs_JOB_MAX_ATTEMPTS` times
   with exponential backoff and full jitter based on `mlrs_JOB_RETRY_BASE_SECONDS`; add the
   non-terminal status `retrying`; expose `attempts`, `maxAttempts`, and `nextAttemptAt`; sweep
   stale jobs with `mlrs_WORKER_STALE_SECONDS`.
4. **Job discovery** — add `name` and `tags` to `jobs`; add `PATCH /api/v1/jobs/{jobId}`;
   extend `GET /api/v1/jobs` with `status`, `taskType`, `algorithmId`, `tag`, `q`,
   `createdFrom`, `createdTo`, and `sort`.
5. **Job comparison** — add `GET /api/v1/jobs/compare?ids=`; add the `/jobs/compare` page.
6. **Bulk download** — add `GET /api/v1/jobs/{jobId}/artefacts.zip` and
   `GET /api/v1/jobs/artefacts.zip?ids=`; build the archive in a temporary file and stream it.
7. **Dataset insights** — add the `dataset_insights` table, the profile computation at upload
   time, `GET /api/v1/datasets/{datasetId}/insights`, and the insights panel.
8. **Client resilience** — give the fetch wrapper a timeout, retries with jitter for safe
   methods, `AbortSignal` support, and a discriminated `ApiError` taxonomy.
9. **Actionable errors** — add the central error catalogue and extend `ApiAlert` with
   `guidance`, `action`, and `details`; apply the catalogue on every screen.
10. **Wizard feedback** — live per-step validation, a focusable summary panel, an always-enabled
    primary action, and `sessionStorage` state persistence.
11. **OpenAPI surfacing** — tag the routers, document `/docs`, `/redoc`, and `/openapi.json`,
    link the reference from the app footer and the docs, and add a CI drift check against the
    committed `docs/06-api-specification/backend/openapi.yaml`.
12. **Migration** — Alembic revision `0005`: widen the `jobs.status` CHECK to include `retrying`,
    add `name`, `tags`, `attempts`, `max_attempts`, `next_attempt_at`, add the discovery
    indexes, and create `dataset_insights`.

## Order of work

1. Configuration module and env-var wiring (1) — everything else reads settings from it.
2. Upload hardening (2) and dataset insights storage (7) — both touch the upload path.
3. Migration `0005` (12) — the schema must exist before the query code that uses it.
4. Worker reliability (3).
5. Job discovery (4) and comparison (5).
6. Bulk download (6).
7. Client resilience, actionable errors, wizard feedback (8, 9, 10).
8. OpenAPI surfacing and the CI drift check (11).
9. Regenerate the OpenAPI document and the Postman collection.

## Acceptance

- The backend starts with no environment variables set and serves every documented route.
- `mlrs_CORS_ORIGINS` controls the allow-list; the hard-coded list is gone from `main.py`.
- A binary payload renamed to `.csv` is rejected with `UNSUPPORTED_FILE_TYPE`; a valid CSV is
  still accepted with `201`.
- With `mlrs_MAX_CONCURRENT_JOBS=1` a second job waits for the first; a job that exceeds
  `mlrs_JOB_TIMEOUT_SECONDS` fails with `JOB_TIMEOUT` and leaves no artefact; a transient failure
  is retried and can succeed; a deterministic failure is not retried.
- `GET /api/v1/jobs` filters and sorts as documented; `PATCH /api/v1/jobs/{jobId}` updates only
  `name` and `tags`.
- `GET /api/v1/jobs/compare?ids=` returns 2–5 jobs with the documented warnings.
- The ZIP bundle contains byte-identical artefacts and writes nothing to the volume.
- The insights endpoint returns a profile for every column and never blocks a job.
- The client retries safe requests, times out long ones, aborts on unmount, and renders an
  actionable message for every documented error code.
- The wizard explains every blocking problem and survives a reload.
- `GET /openapi.json` works, the committed `openapi.yaml` matches it, and CI fails on drift.
- The backend suite keeps ≥ 80 % coverage; the frontend suite keeps ≥ 80 % statements, lines,
  and functions.
- All existing scenarios in
  [`05-critical-test-scenarios.md`](../09-testing-strategy/05-critical-test-scenarios.md) stay
  green.

## See also

- [`02-features.md`](../02-features.md) — feature index
- [`04-architecture.md`](../04-architecture.md) — configuration, concurrency, failure recovery
- [`05-data-model.md`](../05-data-model.md) — schema changes
- [`06-api-specification.md`](../06-api-specification.md) — REST contract
- [`11-deployment/environment-variables.md`](../11-deployment/environment-variables.md) — env vars
