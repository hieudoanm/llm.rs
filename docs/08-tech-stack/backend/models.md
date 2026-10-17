# llm — Model Catalogue

The allow-listed estimators the platform can train: 25 estimators across the six
task types (9 classifiers, 10 regressors, 1 self-training classifier, 3
clustering models, 1 anomaly detector, and the `fitted_q_iteration`
Q-function). Served by `GET /api/v1/algorithms`, selected in the UI model step,
and fitted by the worker — never any other model.

- **Source of truth:** `crates/llm-core/src/training/catalogue/` (`ALGORITHMS`,
  assembled in `catalogue/algorithms.rs`).
- **Spec table:** [`01-requirements`](../../01-requirements/functional-requirements.md)
  (FR-020..027, FR-080..105). `crates/llm-core/src/training/` tests
  assert the code and the spec table stay in sync; tests assert every entry
  fits, exports to ONNX, and produces predictions identical to llm.
- **Shared pipeline for all entries:** ColumnTransformer-like preprocessing with
  StandardScaler (numeric columns) + OneHotEncoder(handle_unknown="ignore")
  (categorical), then the estimator. `random_state=42` wherever the estimator
  supports it. Split, seed, and metrics come from the job, not the algorithm.
- **Task strategies:** supervised fits the estimator on a labelled train split;
  semi-supervised appends unlabelled rows with a `-1` sentinel; unsupervised
  fits and scores all rows in-sample.

## Classification (9)

| algorithmId                    | Estimator                                                | defaultHyperparameters                           | Notes                                         |
| ------------------------------ | -------------------------------------------------------- | ------------------------------------------------ | --------------------------------------------- |
| `logistic_regression`          | `LogisticRegression(max_iter=1000)`                      | `{}`                                             | Linear baseline; probabilistic, very fast     |
| `random_forest_classifier`     | `RandomForestClassifier(n_estimators=100)`               | `{"n_estimators": 100}`                          | Bagged trees; robust default for tabular data |
| `gradient_boosting_classifier` | `GradientBoostingClassifier(n_estimators=100)`           | `{"n_estimators": 100}`                          | Sequential boosting; usually highest accuracy |
| `decision_tree_classifier`     | `DecisionTreeClassifier`                                 | `{}`                                             | Single tree; fully interpretable              |
| `extra_trees_classifier`       | `ExtraTreesClassifier(n_estimators=100)`                 | `{"n_estimators": 100}`                          | Randomized trees; trains faster than forest   |
| `k_neighbors_classifier`       | `KNeighborsClassifier(n_neighbors=5)`                    | `{"n_neighbors": 5}`                             | Instance-based; no explicit training phase    |
| `gaussian_naive_bayes`         | `GaussianNB`                                             | `{}`                                             | Generative baseline; cheapest fit             |
| `svm_classifier`               | `SVC(C=1.0, kernel="rbf")`                               | `{"C": 1.0, "kernel": "rbf"}`                    | Margin-based; strong on smaller datasets      |
| `mlp_classifier`               | `MLPClassifier(hidden_layer_sizes=(100,), max_iter=300)` | `hidden_layer_sizes`, `activation`, `solver`, `alpha`, `learning_rate_init`, `max_iter`, `early_stopping` | Configurable tabular MLP; fits are seeded |

## Regression (10)

| algorithmId               | Estimator                                               | defaultHyperparameters                           | Notes                                          |
| ------------------------- | ------------------------------------------------------- | ------------------------------------------------ | ---------------------------------------------- |
| `linear_regression`       | `LinearRegression`                                      | `{}`                                             | Ordinary least squares baseline                |
| `ridge_regression`        | `Ridge(alpha=1.0)`                                      | `{"alpha": 1.0}`                                 | L2 regularisation; stabilises correlated X     |
| `random_forest_regressor` | `RandomForestRegressor(n_estimators=100)`               | `{"n_estimators": 100}`                          | Bagged trees for continuous targets            |
| `decision_tree_regressor` | `DecisionTreeRegressor`                                 | `{}`                                             | Single tree; interpretable splits              |
| `extra_trees_regressor`   | `ExtraTreesRegressor(n_estimators=100)`                 | `{"n_estimators": 100}`                          | Randomized trees; fast, low variance           |
| `k_neighbors_regressor`   | `KNeighborsRegressor(n_neighbors=5)`                    | `{"n_neighbors": 5}`                             | Averages the k nearest training rows           |
| `lasso_regression`        | `Lasso(alpha=1.0)`                                      | `{"alpha": 1.0}`                                 | L1 regularisation; drives coefficients to zero |
| `elastic_net_regression`  | `ElasticNet(alpha=1.0, l1_ratio=0.5)`                   | `{"alpha": 1.0, "l1_ratio": 0.5}`                | Combined L1 + L2                               |
| `svm_regressor`           | `SVR(C=1.0, kernel="rbf")`                              | `{"C": 1.0, "kernel": "rbf"}`                    | Margin-based regression; epsilon-insensitive   |
| `mlp_regressor`           | `MLPRegressor(hidden_layer_sizes=(100,), max_iter=300)` | `hidden_layer_sizes`, `activation`, `solver`, `alpha`, `learning_rate_init`, `max_iter`, `early_stopping` | Configurable tabular MLP regressor             |

## Neural networks (P1)

`mlp_classifier` and `mlp_regressor` are llm `MLPClassifier` /
`MLPRegressor` models whose architecture is configurable through
hyperparameters — `hidden_layer_sizes` (depth and width), `activation`,
`solver`, `alpha` (L2), `learning_rate_init`, `max_iter`, `batch_size`, and
`early_stopping`. They are the platform's neural-net offering for **tabular**
data; the neural-net offering is configurable tabular llm MLPs (FR-023). Full
hyperparameter list, defaults, and validation ranges in
[`neural-networks.md`](./neural-networks.md).

## Semi-supervised (1)

| algorithmId                | Estimator                                                   | defaultHyperparameters                                          | Notes                                                  |
| -------------------------- | ----------------------------------------------------------- | --------------------------------------------------------------- | ------------------------------------------------------ |
| `self_training_classifier` | `SelfTrainingClassifier(estimator=LogisticRegression(...))` | `{"threshold": 0.75, "criterion": "threshold", "max_iter": 10}` | Learns from unlabelled rows; ONNX exports `estimator_` |

The base estimator is fixed to logistic regression (`max_iter=1000`) in the
P0 entry. A second base estimator, if added later, gets its own algorithmId
rather than a nested hyperparameter.

## Clustering (3)

| algorithmId         | Estimator         | defaultHyperparameters | Notes                               |
| ------------------- | ----------------- | ---------------------- | ----------------------------------- |
| `kmeans`            | `KMeans`          | `{"n_clusters": 3}`    | Centroid baseline; `n_init="auto"`  |
| `mini_batch_kmeans` | `MiniBatchKMeans` | `{"n_clusters": 3}`    | Mini-batch variant for larger CSVs  |
| `gaussian_mixture`  | `GaussianMixture` | `{"n_components": 3}`  | Probabilistic clusters; returns BIC |

`random_state=42` (or the job seed) is fixed. `n_clusters`/`n_components` must
satisfy `2 <= k <= rows - 1`; silhouette is omitted if undefined.

## Anomaly detection (1)

| algorithmId        | Estimator         | defaultHyperparameters                           | Notes                                               |
| ------------------ | ----------------- | ------------------------------------------------ | --------------------------------------------------- |
| `isolation_forest` | `IsolationForest` | `{"n_estimators": 100, "contamination": "auto"}` | `contamination` is `auto` or `(0, 0.5]`; ONNX ML v3 |

## Shared behaviour

- **Exports:** every supervised entry declares `exports: ["onnx", "pkl", "joblib"]`;
  the new semi-supervised and unsupervised entries declare `["onnx", "pkl", "joblib"]`
  as well. ONNX is always written on success, `pkl`/`joblib` only when requested
  on the job ([`onnx.md`](./onnx.md)).
- **Metrics:** classification and semi-supervised → accuracy, precision, recall,
  F1 (`average="macro"`, `zero_division=0`); regression → RMSE, MAE, R²;
  clustering → silhouette, Davies–Bouldin, Calinski–Harabasz, plus inertia/BIC;
  anomaly detection → anomaly count/rate and score mean/std.
- **Validation:** unknown id → `400 UNKNOWN_ALGORITHM` (FR-024);
  `taskType` mismatch → `400 TASK_ALGORITHM_MISMATCH` (FR-025); the catalogue
  endpoint filters with any of the six `taskType` values (FR-021); invalid
  hyperparameters → `400 INVALID_HYPERPARAMETERS` (FR-027). For
  `reinforcement_learning`, `rlColumns` are required and reward-type mismatches
  return `INVALID_LABEL_TYPE`.
- **Defaults:** baked into the estimator instance and displayed in the UI
  (`defaultHyperparameters`); applied when the client omits `hyperparameters`
  (FR-026).
- **ONNX guarantee:** `test_trains_and_exports_onnx` converts every algorithm
  through `llm2onnx` and compares onnxruntime predictions against llm
  (exact labels, `atol=1e-4` for regressors) on each change. The exporter pins
  `ai.onnx.ml` to 3. For reinforcement learning, Q-values are compared
  between llm and ORT on state+one-hot inputs.
- **Semi-supervised export:** train the wrapper, then export the fitted
  preprocessor + `SelfTrainingClassifier.estimator_`, because `llm2onnx` has no
  wrapper converter. Predictions match the wrapper.

## Not in the catalogue

- **DBSCAN, AgglomerativeClustering, LabelSpreading, LabelPropagation** — no
  verified `llm2onnx` converter with the pinned stack, so they cannot satisfy
  the ONNX contract.
- **XGBoost** — rejected as P0 (ONNX path extra), see
  [`08-tech-stack` §5.3](../../08-tech-stack.md#53-llm).

## Adding a model

1. Add the entry to the task-family catalogue module under
   `crates/llm-api/src/training/catalogue/` (for example
   `catalogue/classification.py`), then expose it through
   `catalogue/algorithms.py`.
2. Add its id to the task-specific id set in
   `crates/llm-api/tests/training/fixtures.py`.
3. Add the row to the catalogue table in
   [`01-requirements`](../../01-requirements/functional-requirements.md) and to
   the tables above.
4. Prove ONNX parity with the pinned stack (including `ai.onnx.ml` 3 for
   IsolationForest) before exposing it in the catalogue.
5. Run `cargo test --workspace` — the ONNX parity test
   (`tests/training/core/test_pipeline_export.py`) must pass.

## Reinforcement learning (Phase 11)

Phase 11 adds `reinforcement_learning` as a sixth task type with
`fitted_q_iteration` (Fitted Q-Iteration on a `GradientBoostingRegressor`).
Experience columns are passed via `rlColumns` (action, reward, and either
`nextStateColumns` mirroring `featureColumns` or `episodeColumn`, optional
`doneColumn`); no label column is used. The Q-function is exported to ONNX
(`ai.onnx.ml=3`) as a pipeline of the fitted preprocessor and the regressor,
taking state features followed by a one-hot action block; `π(s)=argmax_a Q(s,a)`
is a documented consumer-side rule. Metrics: `meanReward`, `meanQ`,
`tdErrorMean`, `tdErrorStd`, `actions`, and `episodeCount`/`meanEpisodeReturn`
when episodes exist. No new dependencies are added.
