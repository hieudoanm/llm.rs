# Phase 10: Unsupervised learning

Week 8. Add label-free clustering and anomaly detection on top of the Phase 9
nullable-label plumbing. No new endpoints.

## Scope

1. **Catalogue** — add `kmeans`, `mini_batch_kmeans`, `gaussian_mixture`, and
   `isolation_forest`; expose them under `taskType=clustering` and
   `taskType=anomaly_detection`.
2. **Validation** — unsupervised tasks must omit `labelColumn`
   (`INVALID_COLUMN_MAP`); validate `2 <= n_clusters/n_components <= rows - 1`
   and `contamination` (`auto` or `(0, 0.5]`) with `INVALID_HYPERPARAMETERS`.
3. **Training** — fit the preprocessor and estimator on all rows; no hold-out.
   Compute silhouette, Davies–Bouldin, Calinski–Harabasz, plus inertia/BIC for
   clustering; compute anomaly count/rate and `decision_function` moments for
   IsolationForest.
4. **Export** — convert the fitted preprocessor + estimator with
   `target_opset={"": ONNX_OPSET, "ai.onnx.ml": 3}`; compare ONNX outputs with
   mlrs.
5. **API** — accept the two new `taskType` values; allow `labelColumn: null`;
   document the metrics and errors; update OpenAPI.
6. **UI** — add Clustering and Anomaly detection to the task select; hide the
   label control; add the cluster-count and contamination fields; render the
   new metrics; add job-history badges; extend the algorithms page to five
   groups.
7. **Prototypes and screenshots** — update `02-new-job` and `05-algorithms`, then
   regenerate light/dark images.
8. **Tests** — clustering and anomaly fixtures; API validation tests; metrics
   tests; ONNX parity tests for all four estimators; Playwright paths for both
   tasks.

## Acceptance

- A label-free CSV trains with every Phase 10 estimator and produces ONNX.
- Cluster labels and anomaly labels/scores match mlrs.
- `labelColumn` supplied to an unsupervised task returns
  `400 INVALID_COLUMN_MAP`.
- Invalid cluster counts and contamination return
  `400 INVALID_HYPERPARAMETERS`.
- The UI explains each metric and its direction.
- Existing supervised and semi-supervised flows remain green.
