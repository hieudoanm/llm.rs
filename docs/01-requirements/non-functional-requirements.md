# Non-Functional Requirements

This document specifies the non-functional requirements (NFRs) for mlrs-client (mlrs). Each requirement carries a unique ID (`NFR-###`), a description, and a target.

## Performance

| ID      | Requirement                                                                    | Target  |
| ------- | ------------------------------------------------------------------------------ | ------- |
| NFR-001 | Dataset upload validation and persist shall complete within 2 s (p95) for files ≤ 10 MiB. | 2 s     |
| NFR-002 | `POST /api/v1/jobs` shall return within 300 ms (p95) after validation.         | 300 ms  |
| NFR-003 | `GET /api/v1/jobs/{jobId}` shall complete within 100 ms (p95).                 | 100 ms  |
| NFR-004 | mlrs P0 fits on the bundled sample CSV (≤ 5k rows, ≤ 20 features) shall complete within 60 s (p95) on a developer laptop. | 60 s    |
| NFR-005 | ONNX export for P0 mlrs models on the sample CSV shall complete within 15 s (p95). | 15 s    |
| NFR-006 | `GET /api/v1/jobs` shall complete within 200 ms (p95) with filters, sorting, and cursor pagination applied over 10 000 jobs. | 200 ms |
| NFR-007 | `GET /api/v1/datasets/{datasetId}/insights` shall complete within 1 s (p95) for a CSV within the upload size limit. | 1 s     |
| NFR-008 | A ZIP artefact bundle shall begin streaming within 2 s (p95) of the request, even when the archive is not yet complete. | 2 s     |

**Rationale:** Job *acceptance* is fast because training is asynchronous. Fit time is bounded only for the documented sample; larger user CSVs may take longer and must still update status.

**See also:** [`04-architecture.md`](../04-architecture.md), [`06-api-specification.md`](../06-api-specification.md).

---

## Availability

| ID      | Requirement                                                                 | Target |
| ------- | --------------------------------------------------------------------------- | ------ |
| NFR-010 | Local Compose shall start all three services to healthy within 90 s on a typical developer machine. | 90 s   |
| NFR-011 | Backend `/readyz` shall fail when SQLite is unreachable.                | Must   |
| NFR-012 | A single-node SQLite is sufficient; multi-AZ is out of scope.             | Local  |
| NFR-013 | A queued job shall never wait indefinitely: every job attempt shall terminate within `mlrs_JOB_TIMEOUT_SECONDS`. | 1800 s |
| NFR-014 | A transient backend failure shall not permanently lose an accepted job; the job shall be retried up to `mlrs_JOB_MAX_ATTEMPTS` times. | Must   |

**Environment scoping:** All availability NFRs describe the **local Docker Compose** profile. There is no production cloud profile in this release.

**See also:** [`11-deployment.md`](../11-deployment.md).

---

## Correctness and Durability

| ID      | Requirement                                                                 | Target |
| ------- | --------------------------------------------------------------------------- | ------ |
| NFR-020 | Every `completed` job shall have exactly one ONNX artefact row and file.    | Must   |
| NFR-021 | Metrics JSON shall match the job `taskType` (classification vs regression). | Must   |
| NFR-022 | Dataset bytes and artefact files shall survive container restart (named volumes). | Must   |
| NFR-023 | Job rows shall be written before the worker starts fit (crash leaves `queued`/`validating`, never a silent success). | Must   |
| NFR-024 | SQL migrations shall be versioned and replayable.                           | Must   |
| NFR-025 | A retried job attempt shall not leave duplicate artefact rows or duplicate metric writes. | Must   |
| NFR-026 | A timed-out or failed job shall leave no partial artefact file on the volume. | Must   |
| NFR-027 | A ZIP bundle shall contain byte-identical copies of the artefacts served individually. | Must   |
| NFR-028 | Job `name` and `tags` shall be readable on the list endpoint without a per-job extra query. | Must   |

**See also:** [`05-data-model.md`](../05-data-model.md).

---

## Security

| ID      | Requirement                                                                 | Target |
| ------- | --------------------------------------------------------------------------- | ------ |
| NFR-030 | Upload body size shall be capped (default 10 MiB, configurable).            | 10 MiB |
| NFR-031 | Stored filenames shall be generated UUIDs; user filenames are metadata only.| Must   |
| NFR-032 | Artefact download shall authorise by `jobId` + `artefactId` existence only (local single-user). | Must   |
| NFR-033 | Secrets shall not be committed; `.env.example` holds placeholders.          | Must   |
| NFR-034 | CORS shall be restricted to the known frontend origin.                      | Must   |
| NFR-035 | The API shall not deserialize untrusted binary model files.                       | Must   |
| NFR-036 | Algorithm ids shall be allow-listed; no dynamic loading of user code.      | Must   |
| NFR-037 | The CORS allow-list shall be configurable through `MLRS_CORS_ORIGINS` and shall default to the local frontend origins. | Must   |
| NFR-038 | The API shall reject `*` as a CORS origin while credentialed requests are enabled. | Must   |
| NFR-039 | Uploads shall be validated by content sniffing in addition to extension and size. | Must   |

**Rationale:** Local-only does not justify path traversal, unbounded uploads, or pickle of untrusted bytes.

**See also:** [`04-architecture.md`](../04-architecture.md#9-security-boundaries).

---

## Scalability

| ID      | Requirement                                                                 | Target |
| ------- | --------------------------------------------------------------------------- | ------ |
| NFR-040 | The local stack shall support one training worker (serial jobs) for MVP.    | 1      |
| NFR-041 | Concurrent job creates may queue; the worker processes one fit at a time.   | Must   |
| NFR-042 | The number of jobs executing concurrently shall be bounded by `mlrs_MAX_CONCURRENT_JOBS`. | Must   |
| NFR-043 | The stack shall hold at least 100 queued jobs without degrading the API.     | 100    |

**Rationale:** Serial training keeps CPU/RAM predictable on a laptop. Horizontal scale is a non-goal.

---

## Observability

| ID      | Requirement                                                                 | Target |
| ------- | --------------------------------------------------------------------------- | ------ |
| NFR-050 | Backend logs shall be structured JSON with `requestId` and `jobId`.         | Must   |
| NFR-051 | Dataset row values shall not be logged at info level.                       | Must   |
| NFR-052 | Job state transitions shall be logged (from → to).                          | Must   |
| NFR-053 | Worker retries and timeouts shall be logged with the job id and the attempt number. | Must   |
| NFR-054 | Configuration errors (an invalid environment variable) shall be logged once at startup, with the variable name and the value used. | Must   |

---

## Usability

| ID      | Requirement                                                                 | Target |
| ------- | --------------------------------------------------------------------------- | ------ |
| NFR-060 | Validation errors shall name the field and the rule that failed.            | Must   |
| NFR-061 | The UI shall poll job status at a bounded interval (default 1 s, stop on terminal). | Must   |
| NFR-062 | Screens shall meet WCAG 2.1 AA for contrast and keyboard access.            | AA     |
| NFR-063 | Every documented backend error code shall be rendered with a distinct, actionable message; a bare error code shall never be shown to the user. | Must   |
| NFR-064 | The job wizard shall explain every blocking validation problem before the user leaves the step. | Must   |
| NFR-065 | The browser UI shall never render a stack trace, an exception repr, or a raw HTML error page. | Must   |
| NFR-066 | The API client shall apply a request timeout to every call, and shall surface it as a distinct, retryable error. | Must   |
| NFR-067 | The API client shall cancel in-flight requests when the calling view is left, so that no request outlives its screen. | Must   |

---

## Portability

| ID      | Requirement                                                                 | Target |
| ------- | --------------------------------------------------------------------------- | ------ |
| NFR-070 | ONNX opset and exporter versions shall be pinned and documented.            | Must   |
| NFR-071 | A completed ONNX file shall load in the CI ONNX Runtime version.            | Must   |
| NFR-072 | Every environment variable shall have a documented default so the stack starts without a `.env`. | Must   |
| NFR-073 | The committed OpenAPI document shall be regenerated from the code in CI and CI shall fail on drift. | Must   |
| NFR-074 | The interactive API reference shall be reachable at `/docs` and `/redoc` in the default local deployment. | Must   |

---

## Requirements Traceability

| NFR         | Architecture                         | Test                    |
| ----------- | ------------------------------------ | ----------------------- |
| NFR-001..005 | API + worker                        | test_perf_bounds        |
| NFR-010..012 | Compose                             | test_compose_health     |
| NFR-020..024 | SQLite + volume                   | test_onnx_required      |
| NFR-030..039 | Upload + catalogue + CORS              | test_upload_limits, test_cors_config  |
| NFR-040..043 | Worker concurrency                  | test_serial_worker, test_worker_resilience |
| NFR-050..054 | Logging                             | test_log_redaction, test_worker_logging |
| NFR-060..067 | UI states and API client            | test_error_guidance, test_client_resilience |
| NFR-070..074 | Exporters and configuration         | test_onnx_runtime_load, test_openapi_drift |
