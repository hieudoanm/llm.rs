---
title: Workflow
description: Upload, map, train, and export with llm.rs
order: 3
author: llm.rs
tags: [workflow, tasks, metrics, onnx]
---

# Training workflow

Every task type follows the same four-stage path:

1. **Upload** a CSV with a header row.
2. **Map** feature columns to **X** and, when the task requires it, a label to **Y**.
3. **Choose** a task and an algorithm from the catalogue.
4. **Train** and review task-appropriate metrics, then download the ONNX artefact.

## Task types

| Task                     | Mapping                                   | Example metrics                                   |
| ------------------------ | ----------------------------------------- | ------------------------------------------------- |
| `classification`         | Features **X** + label **Y**              | Accuracy, precision, recall, F1, confusion matrix |
| `regression`             | Features **X** + numeric label **Y**      | RMSE, MAE, R²                                     |
| `semi_supervised`        | Features **X** + partially labelled **Y** | Classification metrics plus label/iteration stats |
| `clustering`             | Features **X** only                       | Silhouette, Davies–Bouldin, Calinski–Harabasz     |
| `anomaly_detection`      | Features **X** only                       | Anomaly count and rate, score distribution        |
| `reinforcement_learning` | State features **X** + experience columns | Mean reward, mean Q, TD-error statistics          |

Empty label cells are treated as unlabelled for semi-supervised jobs. For offline
reinforcement learning, declare the action, reward, and next-state or episode columns
in the experience mapping.

## Evaluation

Supervised, semi-supervised, and offline reinforcement-learning jobs are evaluated on a
hold-out split controlled by the split ratio and seed. Unsupervised jobs report
in-sample cluster or anomaly scores.

## Export

Every successful job writes an `.onnx` file. The exported graph is self-contained: the
preprocessing learned during training is part of the model. Optional pickle and joblib
artefacts are convenience formats, never a replacement for ONNX.

## Reproducibility

Each job records the dataset fingerprint, column mapping, task, algorithm,
hyperparameters, split ratio, seed, library versions, and metrics so the run can be
inspected later.
