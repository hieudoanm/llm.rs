# FEATURE-17: Reliable Job Execution

## Priority

P1 (FR-160..FR-169)

## Description

Training runs in an in-process `asyncio.Task` spawned by `POST /api/v1/jobs`. Three properties are
missing today, and all three bite the user:

1. **No timeout.** A fit that never returns (a pathological hyperparameter, a huge CSV, a hung
   thread) occupies the worker forever. Every later job stays `queued` and the UI looks broken.
2. **No bounded concurrency.** Every accepted job spawns its own task, so ten accepted jobs run
   ten fits at once on a laptop with a 512 MiB memory limit. The container is OOM-killed and every
   job dies.
3. **No retry.** A transient failure — a dropped Postgres connection, a momentarily full disk, a
   container hiccup — permanently fails a job that would have succeeded on a second attempt.

This feature makes the worker bounded, interruptible, and retry-aware.

## User-visible behaviour

### Bounded concurrency

- At most **N** jobs execute at the same time. `N` is `mlrs_MAX_CONCURRENT_JOBS` (default **1**,
   preserving the current serial behaviour).
- When the limit is reached, additional jobs stay in `queued` and the jobs page shows the queue
  position: **"Queued — 2 jobs ahead"**.
- The queue position is computed from the number of `queued` jobs with an older `created_at`.

### Per-job timeout

- Every job has a wall-clock budget: `mlrs_JOB_TIMEOUT_SECONDS` (default **1800** = 30 minutes).
- When the budget is exceeded, the worker cancels the job and records `failed` with the error code
  `JOB_TIMEOUT`.
- The timeout message names the limit and the elapsed time:
  `"Training exceeded the 1800 s time limit and was stopped."`
- A timed-out job leaves no partial artefacts. The worker removes any artefact file it had already
  written before recording the failure.

### Retry with backoff

- A job is attempted up to `mlrs_JOB_MAX_ATTEMPTS` times (default **2** — the initial attempt plus
  one retry).
- **Only transient failures are retried.** Transient means the failure was caused by the
  environment rather than by the data: a database connection error, a disk write error, an OS-level
  error, or an unhandled worker interruption. Deterministic failures — invalid column map, invalid
  label type, insufficient labels, invalid hyperparameters, a fit that raises a
  `ValueError`/`TypeError` from the estimator — are **never** retried, because they would fail
  identically.
- Between attempts the job returns to `queued` with the status `retrying`, and the worker waits
  before the next attempt.
- The wait uses **exponential backoff with full jitter**: attempt *n* waits a random duration in
  `[0, base × 2^(n-1))` seconds, where `base` is `mlrs_JOB_RETRY_BASE_SECONDS` (default **2**).
  With the default base the first retry waits 0–2 s, the second 0–4 s, the fourth 0–16 s, capped
  at 30 s.
- The job records `attempts` (how many attempts have started) and `nextAttemptAt` (when the next
  attempt is scheduled).
- When the last attempt fails, the job is recorded `failed` with the code `JOB_RETRY_EXHAUSTED`
  and the message from the underlying failure.

### Job status `retrying`

A new job status is added:

| Status    | Badge   | User copy                                          |
| --------- | ------- | -------------------------------------------------- |
| `retrying` | warning | "Retrying — attempt 2 of 2 after a temporary error" |

`retrying` is a non-terminal status: the job detail page keeps polling, and the stepper shows the
job at the step where it failed, with a "retrying" marker.

### Observable state

The job payload gains:

| Field          | Type              | Meaning                                                        |
| -------------- | ----------------- | -------------------------------------------------------------- |
| `attempts`     | integer           | Number of attempts started so far (1 while running, 0 before the worker claims it). |
| `nextAttemptAt`| timestamp \| null | When the next attempt is scheduled; `null` when not waiting.    |
| `maxAttempts`  | integer           | The configured attempt budget for this job.                    |

## Configuration

| Variable                    | Default | Meaning                                                  |
| --------------------------- | ------- | -------------------------------------------------------- |
| `mlrs_MAX_CONCURRENT_JOBS`   | `1`     | Maximum jobs executing concurrently.                    |
| `mlrs_JOB_TIMEOUT_SECONDS`   | `1800`  | Wall-clock budget for a single job attempt.            |
| `mlrs_JOB_MAX_ATTEMPTS`      | `2`     | Total attempts per job, including the first.            |
| `mlrs_JOB_RETRY_BASE_SECONDS`| `2`     | Base delay of the exponential backoff.                  |

Invalid values (non-integer, zero, negative, or above the documented maximum) fall back to the
default and are logged once at startup as a warning.

## API

No new endpoints. The existing `GET /api/v1/jobs` and `GET /api/v1/jobs/{jobId}` carry the new
fields and the new `retrying` status.

## Edge cases

| Case                                          | Behaviour                                                          |
| --------------------------------------------- | ------------------------------------------------------------------ |
| Job deleted while queued                      | The job is removed; the worker skips it and never starts an attempt |
| Job deleted while running                     | The task is cancelled at its next await point; no artefacts are written |
| Timeout fires while the fit is inside `asyncio.to_thread` | The attempt is abandoned; the OS thread may finish but its result is discarded |
| A retry is scheduled while the backend restarts | On startup, jobs in `retrying` are reset to `queued` and re-attempted |
| The container is OOM-killed                    | On startup, jobs left in `validating`/`training`/`evaluating`/`exporting` older than `mlrs_WORKER_STALE_SECONDS` (default 3600) are recorded `failed` with `WORKER_INTERRUPTED` |
| Retry budget exhausted                        | `failed` with `JOB_RETRY_EXHAUSTED`; the underlying message is preserved in `error.message` |
| A deterministic failure on the first attempt  | No retry; `failed` with the original code (`TRAINING_FAILED`)      |
| `mlrs_MAX_CONCURRENT_JOBS` = 1 (default)       | Behaviour matches today's serial worker, including "next job starts after the previous one terminates" |

## Acceptance conditions

- With `mlrs_MAX_CONCURRENT_JOBS=1`, a second accepted job remains `queued` until the first reaches
  a terminal status.
- With `mlrs_MAX_CONCURRENT_JOBS=2`, two jobs run simultaneously and a third waits.
- A job whose fit exceeds `mlrs_JOB_TIMEOUT_SECONDS` is recorded `failed` with `error.code =
  "JOB_TIMEOUT"`, and no artefact row is created for it.
- A job whose first attempt fails with a transient error and whose second attempt succeeds ends
  `completed` with `attempts = 2` and a valid ONNX artefact.
- A job whose first attempt fails with `INVALID_COLUMN_MAP` is recorded `failed` after exactly
  one attempt (`attempts = 1`).
- A job whose attempts are exhausted is recorded `failed` with `error.code =
  "JOB_RETRY_EXHAUSTED"`.
- The wait between two attempts is randomised within `[0, base × 2^(n-1))` seconds and is never
  negative.
- The jobs page shows the queue position for `queued` jobs.

**See also:** [`FEATURE-04`](./FEATURE-04-Training-Job.md),
[`FEATURE-07`](./FEATURE-07-Error-Handling.md),
[`04-architecture.md`](../04-architecture.md),
[`11-deployment/environment-variables.md`](../11-deployment/environment-variables.md).
