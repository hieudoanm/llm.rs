# FEATURE-10: Unsupervised Learning

## Priority

P0 (FR-090..FR-105). Planned in Phase 10.

## Description

Unsupervised learning adds two label-free tasks to the studio:

- **Clustering** — group rows into a chosen number of clusters with KMeans, MiniBatchKMeans, or GaussianMixture.
- **Anomaly detection** — flag unusual rows with IsolationForest.

Both tasks fit on every row, return in-sample metrics, and export a verified ONNX model.

## User-visible behaviour

- Task select adds **Clustering** and **Anomaly detection**.
- The label select is hidden for both tasks, with the helper "No label column is used."
- Clustering shows a **Number of clusters** field (2 to rows − 1, default 3).
- Anomaly detection shows a **Contamination** field (`auto` or 0 < value ≤ 0.5).
- The job status screen renders cluster or anomaly metrics with direction hints.
- Job history shows the task badge and no label column.
- The `/algorithms` page groups the catalogue into Classification, Regression, Semi-supervised, Clustering, and Anomaly detection sections.

## Inputs

| Field           | Type     | Required | Constraints                                        |
| --------------- | -------- | -------- | -------------------------------------------------- |
| datasetId       | UUID     | Yes      | Must exist                                         |
| featureColumns  | string[] | Yes      | Subset of header, ≥ 1                              |
| labelColumn     | —        | No       | Must be omitted or `null`                          |
| taskType        | enum     | Yes      | `clustering` \| `anomaly_detection`                |
| algorithmId     | string   | Yes      | Catalogue allow-list for the task                  |
| hyperparameters | object   | No       | Cluster count or contamination as documented below |

### Clustering

| algorithmId         | Estimator         | Hyperparameters            | Default |
| ------------------- | ----------------- | -------------------------- | ------- |
| `kmeans`            | `KMeans`          | `n_clusters` (2..rows−1)   | 3       |
| `mini_batch_kmeans` | `MiniBatchKMeans` | `n_clusters` (2..rows−1)   | 3       |
| `gaussian_mixture`  | `GaussianMixture` | `n_components` (2..rows−1) | 3       |

`random_state` is fixed to 42 (or the job seed) so metrics and exports are reproducible.

### Anomaly detection

| algorithmId        | Estimator         | Hyperparameters                         | Default |
| ------------------ | ----------------- | --------------------------------------- | ------- |
| `isolation_forest` | `IsolationForest` | `contamination` (`auto` or 0 < c ≤ 0.5) | `auto`  |

## Behaviour

1. Validate that `labelColumn` is absent and that the hyperparameters are in range.
2. Fit the shared preprocessor on all rows.
3. Fit the estimator on all rows.
4. Compute in-sample metrics:
   - **Clustering:** silhouette, Davies–Bouldin, Calinski–Harabasz, plus inertia (KMeans/MiniBatchKMeans) or BIC (GaussianMixture).
   - **Anomaly detection:** anomaly count and rate, plus mean and standard deviation of `decision_function`.
5. Export the fitted preprocessor + estimator to ONNX with `ai.onnx.ml` set to 3.
6. Record the metrics and complete the job.

No hold-out split is created: unsupervised evaluation scores the same rows used for fitting. `splitRatio` is accepted for API compatibility but ignored.

## Metrics

| Metric             | Task              | Meaning                                                            |
| ------------------ | ----------------- | ------------------------------------------------------------------ |
| `silhouette`       | clustering        | Separation and cohesion, from −1 to 1 (higher is better).          |
| `daviesBouldin`    | clustering        | Average worst-case similarity between clusters (lower is better).  |
| `calinskiHarabasz` | clustering        | Between-cluster vs within-cluster dispersion (higher is better).   |
| `inertia`          | KMeans, MiniBatch | Sum of squared distances to the nearest centre (lower is better).  |
| `bic`              | GaussianMixture   | Information criterion with a complexity penalty (lower is better). |
| `anomalyCount`     | anomaly detection | Number of rows predicted as outliers.                              |
| `anomalyRate`      | anomaly detection | Share of rows predicted as outliers.                               |
| `scoreMean`        | anomaly detection | Mean `decision_function` score; lower means more anomalous.        |
| `scoreStd`         | anomaly detection | Standard deviation of `decision_function` scores.                  |

## Edge cases

- A supplied label column → `400 INVALID_COLUMN_MAP`.
- `n_clusters`/`n_components` below 2 or greater than or equal to the row count → `400 INVALID_HYPERPARAMETERS`.
- `contamination` outside `(0, 0.5]` → `400 INVALID_HYPERPARAMETERS`.
- Silhouette undefined (all rows identical) → omit `silhouette`; return the remaining metrics.
- Fewer than three rows → `400 INVALID_HYPERPARAMETERS` because no valid cluster count exists.
- ONNX export must set `ai.onnx.ml` to 3; the default v4 operators are not supported by the pinned ONNX Runtime.

## Acceptance conditions

- Clustering CSV completes for KMeans, MiniBatchKMeans, and GaussianMixture with documented metrics and ONNX.
- Anomaly CSV completes for IsolationForest with anomaly metrics and ONNX.
- ONNX cluster labels and anomaly labels/scores match llm.
- Unsupervised jobs expose `labelColumn = null`.
- The UI hides the label control and validates the cluster/contamination fields before submission.
