---
title: Overview
description: llm.rs documentation overview
order: 0
author: llm.rs
tags: [overview, machine-learning, csv, onnx]
---

# llm.rs documentation

Machine Learning in Rust (llm.rs) is a local, full-stack training studio for tabular
data. Upload a CSV with a header row, map feature columns to **X** and, when the task
requires it, a label to **Y**, choose a task and an algorithm, train, and download a
portable **ONNX** model.

Everything runs on your own machine. Datasets, jobs, metrics, and artefacts are kept
in local SQLite storage — no cloud account or infrastructure setup.

## Start here

- [Getting started](/docs/01-getting-started/) — build the CLI or run the desktop app.
- [CLI reference](/docs/02-cli/) — register datasets, train, inspect jobs, and export.
- [Training workflow](/docs/03-workflow/) — map columns and choose a task.
- [Algorithms](/docs/04-algorithms/) — the curated catalogue for every task type.
- [Architecture](/docs/05-architecture/) — crates, packages, and local storage.

## The workflow

1. Register a CSV with a header row.
2. Map feature columns to **X** and, for supervised tasks, the target to **Y**.
3. Choose a task — classification, regression, semi-supervised, clustering, anomaly
   detection, or offline reinforcement learning — and an algorithm.
4. Train and review task-appropriate metrics.
5. Download the `.onnx` artefact.

Every successful job exports ONNX. It is the single export contract across all six task
types.
