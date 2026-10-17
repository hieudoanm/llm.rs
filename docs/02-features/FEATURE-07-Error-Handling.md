# FEATURE-07: Error Handling

## Priority

P0

## Description

All API errors use a single envelope. The UI maps codes to alerts. Stack traces never appear in JSON or the browser.

## Envelope

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

## Codes

| Code                      | HTTP                             | When                                         |
| ------------------------- | -------------------------------- | -------------------------------------------- |
| `VALIDATION_ERROR`        | 400                              | Schema / CSV parse                           |
| `INVALID_COLUMN_MAP`      | 400                              | X/Y mapping                                  |
| `INVALID_LABEL_TYPE`      | 400                              | Y vs taskType                                |
| `INSUFFICIENT_LABELS`     | 400                              | Semi-supervised labelled-row minimum not met |
| `INVALID_HYPERPARAMETERS` | 400                              | Hyperparameter out of range                  |
| `UNKNOWN_ALGORITHM`       | 400                              | Allow-list miss                              |
| `TASK_ALGORITHM_MISMATCH` | 400                              | Catalogue mismatch                           |
| `IDEMPOTENCY_KEY_EXISTS`  | 409                              | `Idempotency-Key` reused with a different payload |
| `UPLOAD_TOO_LARGE`        | 413                              | Size cap                                     |
| `UNSUPPORTED_FILE_TYPE`   | 400                              | Upload is not a CSV by extension, content type, or sniffed bytes |
| `MALFORMED_CSV`           | 400                              | Bytes are not decodable UTF-8 or not parseable as delimited text |
| `DATASET_NOT_FOUND`       | 404                              | Unknown datasetId                            |
| `JOB_NOT_FOUND`           | 404                              | Unknown jobId                                |
| `ARTEFACT_NOT_FOUND`      | 404                              | Unknown artefact                             |
| `TRAINING_FAILED`         | 200 on GET job (`status=failed`) | Fit/validation error recorded on job          |
| `EXPORT_FAILED`           | 200 on GET job (`status=failed`) | ONNX/pkl/joblib export error recorded on job  |
| `JOB_TIMEOUT`             | 200 on GET job (`status=failed`) | The attempt exceeded `llm_JOB_TIMEOUT_SECONDS` |
| `JOB_RETRY_EXHAUSTED`     | 200 on GET job (`status=failed`) | Every attempt failed with a transient error  |
| `WORKER_INTERRUPTED`      | 200 on GET job (`status=failed`) | A crashed process left the job non-terminal   |
| `INTERNAL_ERROR`          | 500                              | Unexpected server error                       |
| `SERVICE_UNAVAILABLE`     | 503                              | Postgres down                                |

Job-level failures are **not** 5xx on the poll endpoint; the GET succeeds and `status` is `failed`.

Worker failures record a job-level code: validation and fit errors set `TRAINING_FAILED`, export errors set `EXPORT_FAILED`, and any other unexpected error sets `INTERNAL_ERROR`. The code appears on the poll payload as `error.code` with `status=failed`.

The client adds two transport-level codes that never come from the server:
`NETWORK_ERROR` (the request never reached the API) and `REQUEST_TIMEOUT` (the request exceeded
its client-side deadline). Both are retryable. The full catalogue, with the user-facing guidance
for every code, is in
[`FEATURE-18`](./FEATURE-18-Client-Resilience-and-Guidance.md).
