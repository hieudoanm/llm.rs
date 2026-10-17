# llm — Critical Scenarios

1. Valid classification CSV → ONNX loads in ORT; predict length equals test rows.
2. Valid regression CSV → metrics keys rmse/mae/r2.
3. Y in X → 400 INVALID_COLUMN_MAP; no job row.
4. Unknown algorithm → 400.
5. Export monkeypatched to fail → status failed, no onnx artefact.
6. Idempotency key replay → same jobId.
7. Worker serial: two jobs queued, second starts after first terminal.
8. Delete job (P1) removes files.
9. `/readyz` 503 if SQLite locked.
10. Playwright happy path.
11. Semi-supervised CSV with blank labels → classification metrics on labelled hold-out, `labelledRows`/`unlabelledRows`/`iterations`, and ONNX predictions equal to the fitted wrapper.
12. Semi-supervised CSV with fewer than 10 labelled rows → 400 INSUFFICIENT_LABELS; no job row.
13. Clustering CSV → silhouette/Davies–Bouldin/Calinski–Harabasz/inertia metrics; ONNX cluster labels equal llm.
14. GaussianMixture → BIC metric; ONNX labels equal llm.
15. Anomaly CSV → anomalyCount/anomalyRate/scoreMean/scoreStd; ONNX labels and scores match llm.
16. Clustering or anomaly job with `labelColumn` → 400 INVALID_COLUMN_MAP; classification/semi-supervised without `labelColumn` → 400 INVALID_COLUMN_MAP.
17. Clustering `n_clusters` ≥ row count → 400 INVALID_HYPERPARAMETERS.
18. Catalogue integrity across all six task types: every entry fits, exports ONNX, and matches llm; the two id sets stay in sync with the spec tables.
19. Playwright semi-supervised happy path (blank-label fixture) and label-less clustering path. Reinforcement-learning Playwright path: experience CSV → map state + rlColumns → train with `fitted_q_iteration` → ONNX download.
20. Completed classification job with chart data → `charts` field populated in poll response, charts rendered in UI (confusion matrix, precision/recall/F1).
21. Completed job without chart data (e.g., before feature implementation) → `charts` is null, Model report section hidden in UI.
22. Chart data caps enforced: classification with 60 classes → `classes` and `confusionMatrix` truncated to first 50, note added; regression with 1000 test rows → `regressionPoints` sampled to 500 points using `randomSeed`.
23. Charts respect `randomSeed` for deterministic sampling: same job with same seed produces identical chart data across runs.
24. Accessibility: screen reader reads chart values via visually-hidden table or `aria-label`; keyboard navigation reaches chart interactive elements; `prefers-reduced-motion: reduce` suppresses animations.

25. Reinforcement learning: experience CSV with label supplied → 400 INVALID_COLUMN_MAP; missing `rlColumns` or missing `nextStateColumns`/`episodeColumn` → 400 INVALID_COLUMN_MAP; non-numeric reward → 400 INVALID_LABEL_TYPE; FQI out-of-range hyperparameters → 400 INVALID_HYPERPARAMETERS; completed RL job returns `meanReward`, `meanQ`, `tdErrorMean`, `tdErrorStd`, `actions` and RL charts; ONNX Q-values match llm for state+one-hot action.

## Phase 12 scenarios

26. **Upload hardening**: a ZIP/gzip/PNG/PDF byte stream renamed to `.csv` → 400 `UNSUPPORTED_FILE_TYPE`; a non-UTF-8 byte stream → 400 `MALFORMED_CSV`; a header with a duplicate column name → 400 `VALIDATION_ERROR` with `details.field = "columns"`; a valid CSV still returns `201`.
27. **CORS configuration**: with `llm_CORS_ORIGINS` set to a single custom origin, a request with a different `Origin` header receives no `Access-Control-Allow-Origin`; with the default value, `http://localhost:3000` is allowed and `http://evil.example` is not; `llm_CORS_ORIGINS=*` is rejected while credentials are enabled.
28. **Worker concurrency**: with `llm_MAX_CONCURRENT_JOBS=1`, a second job stays `queued` until the first is terminal; with `=2`, two jobs execute at the same time.
29. **Job timeout**: with `llm_JOB_TIMEOUT_SECONDS=1`, a job whose fit is slow is recorded `failed` with `error.code = "JOB_TIMEOUT"` and creates no artefact row and no artefact file.
30. **Worker retry**: a transient failure on the first attempt followed by a success ends `completed` with `attempts = 2`; a deterministic failure ends `failed` after exactly one attempt; an exhausted budget ends `failed` with `error.code = "JOB_RETRY_EXHAUSTED"`; the delay between attempts is inside `[0, base × 2^(n-1))`.
31. **Job filters**: `GET /api/v1/jobs?status=failed` returns only failed jobs; `?tag=a,b` returns only jobs carrying both tags; `?q=<algorithm fragment>` matches the algorithm id; `?sort=algorithmId:asc` orders by algorithm id with a stable tie-break; an unknown status, an unknown sort field, a repeated query parameter, and `createdTo < createdFrom` each return `400 VALIDATION_ERROR`.
32. **Job metadata**: `PATCH /api/v1/jobs/{jobId}` with `{"tags": ["Baseline", "baseline", "release"]}` stores `["baseline", "release"]`; an empty `name` stores `null`; an unknown field returns `400 VALIDATION_ERROR`; the training configuration is unchanged after the patch.
33. **Job comparison**: `GET /api/v1/jobs/compare?ids=<a>,<b>` returns one entry per job; fewer than 2 ids and more than 5 ids return `400 VALIDATION_ERROR`; an unknown id returns `404 JOB_NOT_FOUND` with `details.missingJobIds`; comparing jobs from different datasets adds the `DIFFERENT_DATASETS` warning; comparing a failed job returns `200` with `metrics = null` and the job's error.
34. **Bulk download**: `GET /api/v1/jobs/{jobId}/artefacts.zip` returns `application/zip` containing the ONNX, `metrics.json`, and `manifest.json`; the extracted ONNX is byte-identical to the individually served artefact; `GET /api/v1/jobs/artefacts.zip?ids=<a>,<b>` returns one directory per job and a `skipped.json` naming omitted jobs; a job with no artefacts returns `404 ARTEFACT_NOT_FOUND`; the volume gains no new file.
35. **Dataset insights**: `GET /api/v1/datasets/{datasetId}/insights` returns a profile for every column; a constant column raises `CONSTANT_COLUMN`; a column whose distinct count equals the row count raises `POSSIBLE_ID_COLUMN`; `previewLimit` outside 1–100 returns `400 VALIDATION_ERROR`; no warning prevents job creation.
36. **Client retries**: a `GET` that fails with a network error is retried and eventually succeeds; a `GET` that returns `400` is not retried; a `POST /api/v1/jobs` without an `Idempotency-Key` is not retried; a request that exceeds `timeoutMs` fails with `REQUEST_TIMEOUT`.
37. **Client cancellation**: unmounting the job-detail view aborts the in-flight poll request.
38. **Actionable errors**: every code in the error catalogue renders a distinct title, guidance, and retryable flag; an unknown code falls back to the generic guidance and shows the raw code in the details disclosure; no alert renders a stack trace.
39. **Wizard validation**: clicking the primary action on an invalid step keeps the button enabled, expands the summary, and moves focus to the first invalid field; the summary lists every blocking problem; reloading the wizard restores the in-progress setup and shows the restore notice.
40. **OpenAPI drift**: the committed `docs/06-api-specification/backend/openapi.yaml` matches the schema generated from the running application; `GET /openapi.json` returns `200`; the CI drift check fails when the committed file is edited by hand.
