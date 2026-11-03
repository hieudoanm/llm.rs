---
title: Architecture
description: Crates, packages, and local storage in llm.rs
order: 5
author: llm.rs
tags: [architecture, rust, tauri, sqlite, onnx]
---

# Architecture

llm.rs separates training from its user interfaces so the same logic backs the desktop
app and the CLI.

## Repository layout

| Path               | What it is                                                                                         |
| ------------------ | -------------------------------------------------------------------------------------------------- |
| `crates/llm-core`  | Pure-Rust core: CSV parsing, catalogue, preprocessing, training, metrics, charts, and ONNX export. |
| `crates/llm-db`    | SQLite persistence for datasets, jobs, metrics, and artefacts.                                     |
| `crates/llm-cli`   | The `llm` command-line binary.                                                                     |
| `packages/desktop` | Next.js UI wrapped in a Tauri desktop shell.                                                       |
| `packages/landing` | This static documentation and marketing site.                                                      |
| `docs/`            | The product and implementation specification.                                                      |

## Request flow

1. A CSV is registered and hashed.
2. The user maps **X** and, when required, **Y**, then chooses a task and algorithm.
3. Training runs outside the UI thread and records its lifecycle.
4. Metrics and charts are stored with the job.
5. The model is exported to ONNX and recorded as an artefact.

## Storage

State is local. By default the app writes to `.llm-data/`, with SQLite metadata and
artefacts under the same root. Configure it with `LLM_DATA_DIR`; cap uploads with
`LLM_MAX_UPLOAD_MB`.

## Testing

```sh
# Rust workspace
cd crates && cargo test --workspace

# Desktop frontend
cd packages/desktop && pnpm test
```

The Rust core is tested independently of the UI, which keeps training and export
behaviour stable across both surfaces.
