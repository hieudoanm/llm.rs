# llm-core

Core ML logic for [llm.rs](../../README.md). Provides configuration, CSV parsing, dataframes, algorithm catalogue, training pipelines across all task types, preprocessing, ONNX export, and storage utilities.

## Features

- **Configuration**: Environment/settings via `config.rs`
- **Data ingestion**: CSV parsing and dataframe utilities
- **Catalogue**: Curated algorithms for classification, regression, semi-supervised learning, clustering, anomaly detection, and offline reinforcement learning
- **Training**: Task-specific training pipelines with metrics
- **Preprocessing**: Feature scaling/encoding
- **ONNX export**: Export trained models to ONNX format via `onnx-protobuf`
- **Storage**: Dataset/artefact paths and utilities
- **Domain types**: Shared types for datasets, jobs, tasks, etc.

## Supported Tasks & Algorithms

- **Classification**: Logistic Regression, Random Forest, Gradient Boosting, Decision Tree, Extra Trees, k-NN, Gaussian Naive Bayes, SVM, MLP
- **Regression**: Linear/Ridge/Lasso/ElasticNet, k-NN, Decision Tree, Random Forest, Extra Trees, SVR, MLP
- **Semi-supervised**: Self-training classifier
- **Clustering**: KMeans, MiniBatchKMeans, Gaussian Mixture
- **Anomaly detection**: Isolation Forest
- **Offline RL**: Fitted Q-Iteration (FQI)

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
llm-core = { path = "../llm-core" }
```

## Testing

```sh
cargo test -p llm-core
```
