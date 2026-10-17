---
title: Algorithms
description: The llm.rs algorithm catalogue
order: 4
author: llm.rs
tags: [algorithms, classification, regression, clustering, reinforcement-learning]
---

# Algorithms

llm.rs ships a curated, allow-listed catalogue rather than an open-ended estimator
registry. Run `llm algorithms` for the exact list and defaults in your build.

## Classification

Logistic regression, random forest, gradient boosting, decision tree, extra trees,
k-nearest neighbours, Gaussian naive Bayes, support vector machines, and a configurable
multi-layer perceptron.

Metrics include a hold-out confusion matrix, per-class precision/recall/F1, and feature
importances where the model exposes them.

## Regression

Linear regression, ridge, lasso, elastic net, k-nearest neighbours, decision tree,
random forest, extra trees, support vector regression, and a configurable multi-layer
perceptron.

Models are scored with RMSE, MAE, and R².

## Semi-supervised learning

Self-training classification where only some rows carry a label. Empty label cells are
treated as unlabelled, and the job reports labelled/unlabelled counts and iteration
statistics alongside the standard classification metrics.

## Clustering

KMeans, MiniBatchKMeans, and Gaussian Mixture models, scored with silhouette,
Davies–Bouldin, and Calinski–Harabasz scores plus inertia or BIC.

## Anomaly detection

An IsolationForest with configurable contamination. Jobs report anomaly count and rate
alongside the score distribution.

## Offline reinforcement learning

Fitted Q-Iteration (`fitted_q_iteration`) turns a logged experience CSV into a
Q-function. The exported ONNX graph takes `(state, action)` and returns a scalar
Q-value. Metrics include mean reward, mean Q, and TD-error statistics.

## Neural networks

`mlp_classifier` and `mlp_regressor` expose depth and width (`hidden_layer_sizes`),
`activation`, `solver`, L2 `alpha`, `learning_rate_init`, `max_iter`, `batch_size`, and
`early_stopping`. They use the same preprocessor and ONNX export path as every other
catalogue entry.

## Export support

Every algorithm in the default catalogue exports ONNX. Use `llm algorithms` to confirm
the supported formats in your build.
