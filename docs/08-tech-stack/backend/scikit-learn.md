# mlrs — Rust ML

Build with linfa/ndarray/polars. Fit on train split only for supervised tasks. Classification metrics from `mlrs-core` with `average="macro"` and `zero_division=0`. Regression: RMSE, MAE, R². Do not leak test labels into the encoder (`handle_unknown="ignore"`). Pin linfa in `crates/Cargo.toml`.

## Task strategies

- **Classification / regression:** stratified (classification) or random hold-out using `splitRatio` and `randomSeed`; fit the preprocessor and estimator on the train split only.
- **Semi-supervised:** use self-training with a logistic-regression base. Empty, whitespace-only, `NaN`, and `null` label cells become the `-1` sentinel. Split only the labelled rows; append all unlabelled rows to the training features; fit the preprocessor on all training rows; evaluate on the labelled hold-out. Require at least two classes, 10 labelled rows, and 2 labelled rows per class.
- **Clustering / anomaly detection:** fit the preprocessor and estimator on all rows. Do not create a hold-out split. Compute silhouette, Davies–Bouldin, and Calinski–Harabasz for clustering, plus inertia (KMeans/MiniBatchKMeans) or BIC (GaussianMixture). For IsolationForest, report `predict` counts and `decision_function` moments.

## ONNX notes

- Convert the full pipeline with `onnx-protobuf`.
- Pin `target_opset` (default 15) via env `ONNX_OPSET`.
- Verify with `onnx.checker.check_model` before insert.
- CI: ONNX Runtime InferenceSession, one `run` with dummy floats matching `n_features` after transform.
