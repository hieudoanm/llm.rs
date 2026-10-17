---
title: CLI
description: llm.rs command-line reference
order: 2
author: llm.rs
tags: [cli, commands, datasets, training, export]
---

# CLI reference

The `llm` binary trains models from a CSV and exports portable artefacts without the
desktop UI. It uses the same `llm-core` training code and `llm-db` persistence layer as
the app.

## Command summary

```text
llm algorithms [--task <task>]
llm dataset <path.csv>
llm train --dataset-id <id> --task <task> --algorithm <id> --features <columns> [--label <column>]
llm job <job-id>
llm jobs
llm datasets
llm export <job-id> <output> [--format onnx|pkl|joblib]
```

## `llm algorithms`

List every allow-listed algorithm, optionally filtered by task:

```sh
llm algorithms
llm algorithms --task classification
llm algorithms --task regression
```

Each entry prints its id, task type, description, and default hyperparameters.

## `llm dataset`

Register a CSV file with a header row. The command validates the CSV, stores a copy,
and prints the dataset id, content hash, and size:

```sh
llm dataset data/demo.csv
```

## `llm train`

Start a training job. Training runs to completion and exports ONNX as part of the job:

```sh
llm train \
  --dataset-id <dataset-id> \
  --task classification \
  --algorithm random_forest_classifier \
  --features x1,x2,x3 \
  --label target \
  --split-ratio 0.2 \
  --random-seed 42
```

| Flag                | Purpose                                                                                                            |
| ------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `--dataset-id`      | Dataset id printed by `llm dataset` (required).                                                                    |
| `--task`            | `classification`, `regression`, `semi_supervised`, `clustering`, `anomaly_detection`, or `reinforcement_learning`. |
| `--algorithm`       | Algorithm id from `llm algorithms` (required).                                                                     |
| `--features`        | Comma-separated feature columns mapped to **X** (required).                                                        |
| `--label`           | Label column mapped to **Y**; required for supervised and semi-supervised tasks.                                   |
| `--hyperparameters` | JSON object of overrides (default `{}`).                                                                           |
| `--split-ratio`     | Hold-out ratio (default `0.2`).                                                                                    |
| `--random-seed`     | Random seed (default `42`).                                                                                        |

## `llm job` and `llm jobs`

Inspect a single job with its metrics and artefacts, or list recent jobs:

```sh
llm job <job-id>
llm jobs
```

## `llm export`

Copy a stored artefact to a path of your choice:

```sh
llm export <job-id> ./model.onnx
llm export <job-id> ./model.pkl --format pkl
```

ONNX is always produced. Pickle and joblib are optional extras where the algorithm
supports them.
