# FEATURE-18: Client Resilience & Actionable Guidance

## Priority

P1 (FR-170..FR-180)

## Description

Three frontend weaknesses compound each other:

1. The `fetch` wrapper has **no timeout, no retry, and no cancellation**. A backend that accepts a
   connection but never answers leaves the browser spinner forever; the job-status poll keeps
   firing after the user navigated away.
2. **Error codes are mapped inconsistently.** Some screens show a raw message, some show a generic
   "Something went wrong", and a few show the code itself. `INSUFFICIENT_LABELS` — which has a
   precise, fixable meaning — is indistinguishable from `INTERNAL_ERROR`.
3. The **three-step wizard** gives no live feedback about what is missing. Advancing is disabled
   with no explanation of *why*.

This feature fixes all three. It is entirely client-side and adds no backend endpoint.

## 1. Resilient API client

### Request contract

Every call goes through one wrapper (`src/lib/api.ts`). It gains:

| Option          | Default                                | Behaviour                                                                  |
| --------------- | -------------------------------------- | -------------------------------------------------------------------------- |
| `timeoutMs`     | `15000` (`60000` for uploads)          | The request is aborted with `AbortSignal.timeout()` after this long.        |
| `retries`       | `2` for idempotent GETs, `0` otherwise | Number of additional attempts after the first.                              |
| `retryBaseMs`   | `300`                                  | Base of the exponential backoff between attempts.                            |
| `signal`        | —                                      | An external `AbortSignal` that cancels the request and all its retries.     |

### What is retried

Only **safe** requests are retried automatically:

- `GET`, `HEAD` — always retryable.
- `POST` — retryable **only** when an `Idempotency-Key` header is present. The backend already
  replays the original `202` body for a repeated key, so the retry is safe.
- `DELETE` — not retried (the first attempt may already have succeeded).

A request is retried when it fails with:

- a network error (DNS, connection refused, connection reset),
- HTTP `502`, `503`, `504`,
- HTTP `429`.

It is **not** retried for `400`, `404`, `409`, or `422` — those are deterministic answers.

### Backoff

Attempt *n* waits a random duration in `[0, retryBaseMs × 2^(n-1))` ms (full jitter), capped at
`5000` ms. The wait is skipped when the request was aborted by the caller.

### Long polls and cancellation

- The job-status poll uses `refetchInterval` of **1000 ms** and stops on a terminal status
  (`completed` / `failed`).
- `refetchOnWindowFocus` is `false` for the poll query so returning to the tab does not trigger a
  burst of requests.
- When the job-detail component unmounts, the in-flight poll request is aborted through the
  `AbortSignal` held in a ref, so no request survives the page.
- TanStack Query's own `signal` is threaded into the wrapper so React Query's cancellation is
  honoured.

### Error taxonomy

The wrapper raises a single `ApiError` type with a discriminated `code`:

| `ApiError.code`    | Raised when                                                          | Retryable |
| ------------------ | -------------------------------------------------------------------- | --------- |
| `NETWORK_ERROR`    | The request never reached the server.                                | Yes       |
| `REQUEST_TIMEOUT`  | The request exceeded `timeoutMs`.                                    | Yes       |
| `REQUEST_ABORTED`  | The caller aborted via `signal`.                                     | No        |
| `HTTP_ERROR`       | The server answered with a non-2xx status.                            | Depends on the status |
| `INVALID_RESPONSE` | The server answered 2xx with a body that is not valid JSON.           | No        |

`ApiError` keeps the server's error envelope when present (`code`, `message`, `details`,
`requestId`), so the backend's own codes survive the transport layer.

## 2. Actionable error guidance

### Central error catalogue

A single module (`src/lib/errors.ts`) maps every backend error code to a user-facing description:

```ts
interface ErrorGuidance {
  title: string;        // short heading
  message: string;      // one or two sentences, plain language
  action?: string;      // label of the primary action
  actionHref?: string;  // where the action navigates
  retryable: boolean;   // may the user simply retry?
  details?: string[];   // extra bullet points
}
```

Catalogue (abridged — the module is the source of truth):

| Backend code             | Title                | Guidance                                                                                   | Action          |
| ------------------------ | -------------------- | ------------------------------------------------------------------------------------------ | --------------- |
| `VALIDATION_ERROR`       | Check the form       | Some values are not valid. The highlighted fields list what to correct.                     | —               |
| `INVALID_COLUMN_MAP`     | Fix the column map   | The label column is also a feature, or a column in the map is not in the dataset.            | Back to mapping |
| `INVALID_LABEL_TYPE`     | Wrong label type     | The label column must be numeric for regression and categorical for classification.         | Back to mapping |
| `INSUFFICIENT_LABELS`    | Not enough labels    | Semi-supervised needs at least 10 labelled rows, at least 2 classes, and at least 2 rows per class. | Back to mapping |
| `INVALID_HYPERPARAMETERS`| Invalid setting      | The hyperparameter value is outside the range this algorithm accepts.                        | Back to model   |
| `UNKNOWN_ALGORITHM`      | Algorithm unavailable| The algorithm is not in the catalogue any more. Pick another one.                            | Back to model   |
| `TASK_ALGORITHM_MISMATCH`| Wrong algorithm      | That algorithm does not support the selected task.                                           | Back to model   |
| `DATASET_NOT_FOUND`      | Dataset not found    | The dataset was deleted. Upload the CSV again.                                                | Upload again    |
| `JOB_NOT_FOUND`          | Job not found        | The job was deleted.                                                                         | Back to jobs    |
| `ARTEFACT_NOT_FOUND`     | File not found       | The artefact is no longer on disk. Re-run the job.                                            | Re-run job      |
| `UPLOAD_TOO_LARGE`       | File too large       | The file exceeds the {limit} MiB upload limit. Split it or sample it first.                  | Choose another  |
| `UNSUPPORTED_FILE_TYPE`  | Not a CSV file       | The upload must be a plain-text CSV with a header row.                                       | Choose another  |
| `MALFORMED_CSV`          | Cannot read the CSV  | The file could not be parsed as CSV. Check the delimiter and the header row.                 | Choose another  |
| `IDEMPOTENCY_KEY_EXISTS` | Request already sent | A different payload was already submitted with the same idempotency key.                    | —               |
| `TRAINING_FAILED`        | Training failed      | The model could not be fitted with this mapping and these settings.                          | Review settings |
| `EXPORT_FAILED`          | Export failed        | The model trained, but the ONNX export failed.                                               | Re-run job      |
| `JOB_TIMEOUT`            | Training timed out   | Training exceeded the {limit} s limit and was stopped. Reduce the rows or features, or raise the limit. | Review settings |
| `JOB_RETRY_EXHAUSTED`    | Temporary failure    | The job failed {attempts} times because of a temporary environment error.                      | Retry           |
| `INTERNAL_ERROR`         | Server error         | Something failed on the server. The request id is shown below when reporting the problem.    | Retry           |
| `SERVICE_UNAVAILABLE`    | Service unavailable  | The database is not reachable. Start the stack with `docker compose up -d`.                   | Retry           |
| `NETWORK_ERROR`          | Cannot reach the API | The request did not reach the backend at {apiUrl}. Check that the stack is running.            | Retry           |
| `REQUEST_TIMEOUT`        | Request timed out    | The server did not answer within {timeout} ms. It may be busy — try again.                    | Retry           |

`{placeholders}` are interpolated from the error `details` object, from the client's own
configuration, or from a sensible default.

### Alert component

`ApiAlert` gains three optional props:

| Prop         | Type                    | Effect                                                            |
| ------------ | ----------------------- | ----------------------------------------------------------------- |
| `guidance`   | `string`                | Rendered under the message as the actionable explanation.          |
| `action`     | `{ label, href?, onClick? }` | A button below the guidance.                                   |
| `details`    | `string[]`              | A collapsible list of extra bullets (`<details>`/`<summary>`).     |

Behaviour:

- `guidance` is always rendered — it is the difference between "something went wrong" and
  "fix this".
- The `action` button is `btn-primary` when the guidance is retryable, `btn-ghost` otherwise.
- The `requestId` is shown in a `text-xs` footer of every error alert so a user can quote it.
- The alert never renders a raw stack trace, a Rust exception repr, or an HTML error page.

### Where guidance is applied

| Screen                | Error sources                                    | Guidance applies                          |
| --------------------- | ------------------------------------------------ | ----------------------------------------- |
| Job wizard (upload)   | `UPLOAD_TOO_LARGE`, `UNSUPPORTED_FILE_TYPE`, `MALFORMED_CSV`, `VALIDATION_ERROR`, `NETWORK_ERROR` | Yes — inline under the drop zone |
| Job wizard (mapping)  | `INVALID_COLUMN_MAP`, `INVALID_LABEL_TYPE`, `INSUFFICIENT_LABELS` | Yes — inline on the offending control |
| Job wizard (model)    | `UNKNOWN_ALGORITHM`, `TASK_ALGORITHM_MISMATCH`, `INVALID_HYPERPARAMETERS` | Yes — inline on the control |
| Job detail            | `TRAINING_FAILED`, `EXPORT_FAILED`, `JOB_TIMEOUT`, `JOB_RETRY_EXHAUSTED`, `INTERNAL_ERROR` | Yes — with the job's error code and request id |
| Jobs list             | `NETWORK_ERROR`, `REQUEST_TIMEOUT`, `SERVICE_UNAVAILABLE` | Yes — with a retry button |
| Dataset insights      | `DATASET_NOT_FOUND`                               | Yes — with an "Upload again" action     |

## 3. Wizard validation feedback

### Live validation

Each wizard step validates continuously as the user edits it, and the result is exposed three
ways:

1. **Field-level message** — `text-error` under the offending control, linked to it with
   `aria-describedby`, and `aria-invalid="true"` on the control.
2. **Step indicator** — each step in the `steps` component shows a state: `complete` (check
   icon), `current`, or `blocked` (warning icon) with the reason as its accessible name.
3. **Summary panel** — a collapsible **"Fix {n} issues to continue"** panel listing every
   problem as a button that focuses the offending field.

### The primary action stays enabled

The **Next** / **Start training** button is **not** disabled while the step is invalid. Clicking
it:

1. expands the summary panel,
2. moves keyboard focus to the first invalid field,
3. announces the count in the `aria-live` region.

A disabled button with no explanation is a worse experience than an enabled button that explains
itself, and it is invisible to keyboard and screen-reader users.

### Validation rules surfaced live

| Step      | Rule                                                                 | Message shown                                        |
| --------- | -------------------------------------------------------------------- | ---------------------------------------------------- |
| Dataset   | A file is selected and uploaded.                                     | "Upload a CSV to continue."                          |
| Dataset   | The file is a `.csv` and below the size limit.                       | "Only .csv files are accepted." / "The file is larger than the {limit} MiB limit." |
| Mapping   | At least one feature column is selected.                             | "Select at least one feature column."                |
| Mapping   | The label column is not also a feature column.                       | "{column} is used as both the label and a feature."  |
| Mapping   | The label column is required for the selected task.                  | "Choose a label column."                             |
| Mapping   | The label column is absent for unsupervised and RL tasks.            | "{task} does not use a label column."                |
| Mapping   | The label column is numeric for regression.                          | "Regression needs a numeric label column."           |
| Model     | An algorithm compatible with the task is selected.                   | "Choose an algorithm."                               |
| Model     | Every hyperparameter is within the catalogue range.                  | "{field} must be at least {min}." / "{field} must be at most {max}." |
| Model     | A whole-number hyperparameter is an integer.                         | "{field} must be a whole number."                    |
| Model     | A non-numeric value is rejected.                                     | "{field} must be a number."                          |
| Model     | The number of clusters is ≥ 2 and ≤ rows − 1.                        | "Number of clusters must be at least 2." / "Number of clusters must be at most {max}." |
| Model     | Contamination is `auto` or in (0, 0.5].                              | "Contamination must be `auto` or between 0 and 0.5."  |

### Schema-driven hyperparameters

The hyperparameter rules are described once, as a Zod schema built from the catalogue metadata
(`app/jobs/new/_lib/hyperparameters.ts`): each field's `min`, `max`, and `step` become a `.min()`
/ `.max()` / `.int()` rule and its message. The same schema feeds both the hyperparameter form
(`react-hook-form` + `zodResolver`) and the wizard's live validation, so an inline message and the
summary panel can never disagree. Adding a knob to the catalogue is enough to get its input, its
bounds, and its message.

### Wizard keyboard navigation

The wizard is one page with three steps, so the keyboard walks it directly:

- **Alt+→** advances to the next step (subject to the same validation as **Next**).
- **Alt+←** returns to the previous step (never blocked).
- **Alt+1** / **Alt+2** / **Alt+3** jump to the dataset, mapping, and model steps.

The browser's native tab order still moves between the controls inside a step.

### Accidental-navigation guard

While a dataset is loaded and no job has been created, refreshing or closing the tab raises the
browser's unsaved-changes confirmation (a `beforeunload` handler). In-app navigation is **not**
guarded: the state is mirrored into `sessionStorage`, so returning to the jobs list loses nothing.
The guard is dropped once the job is accepted.

### Wizard state persistence

- The wizard's in-progress state (uploaded dataset id, selected columns, task, algorithm,
  hyperparameters, export flags) is written to `sessionStorage` under the key
  `llm:wizard:new-job`.
- The state is restored on mount, with a dismissible notice: **"Restored your unfinished job
  setup."**
- The stored state is cleared when the job is accepted, when the user clicks **Start over**, or
  when the stored `datasetId` no longer resolves (`404 DATASET_NOT_FOUND` → the notice becomes
  "The dataset for your restored setup no longer exists" and the wizard restarts at the upload
  step).
- Only identifiers and settings are stored — never file contents, never metric values.

## Edge cases

| Case                                                        | Behaviour                                                              |
| ----------------------------------------------------------- | ---------------------------------------------------------------------- |
| The backend is down while polling                            | The poll shows the network-error state with a retry button; the interval is not increased |
| The backend restarts mid-poll                                | The next poll succeeds; the transient failure is not shown as a fatal error once a poll succeeds |
| The user leaves the job page while a poll is in flight        | The request is aborted; no state update is applied to an unmounted component |
| The user clicks "Next" three times quickly                    | The validation summary is shown once, focus moves to the first invalid field |
| A validation message is longer than one line                 | It wraps; the control's layout is not clipped                              |
| The API returns an unknown error code                        | The generic "Server error" guidance is used, and the raw code and message are shown in the details disclosure |
| The API returns a 200 with a malformed body                   | `INVALID_RESPONSE` guidance is shown and the screen enters the unexpected-error state |

## Acceptance conditions

- A `GET` that fails with a network error is retried up to the configured number of attempts, with
  randomised backoff between attempts.
- A `GET` that returns `400` is **not** retried.
- A `POST /api/v1/jobs` without an `Idempotency-Key` is **not** retried; with one it is.
- A request that exceeds `timeoutMs` fails with `ApiError.code === 'REQUEST_TIMEOUT'` and shows
  the timeout guidance.
- Unmounting the job-detail page aborts the in-flight poll request.
- Every backend error code in the catalogue renders a distinct, actionable message — no screen
  shows a bare error code to the user.
- The wizard's primary action remains enabled on an invalid step; clicking it focuses the first
  invalid field and lists every problem.
- **Alt+→** advances to the next step only when the current step is valid; **Alt+←** always returns
  to the previous step; **Alt+1**/*2*/*3* jump to a step by number.
- Refreshing the wizard with a dataset loaded raises the unsaved-changes confirmation; accepting a
  job dismisses it.
- Every hyperparameter rule is declared once in the Zod schema and reused by both the form and the
  wizard's validation summary.
- Refreshing the wizard restores the in-progress setup and shows the restore notice.

**See also:** [`FEATURE-07`](./FEATURE-07-Error-Handling.md),
[`FEATURE-08`](./FEATURE-08-UI-States-Global.md),
[`07-ui-ux-design/system/states.md`](../07-ui-ux-design/system/states.md),
[`09-testing-strategy.md`](../09-testing-strategy.md).
