# AGENTS.md — crates/llm-core

Guidance for AI agents working on `llm-core`, the pure-Rust core of the llm.rs training studio.

## Purpose

`llm-core` is the dependency-free (no HTTP, no database) core that backs both the Axum server and the Tauri desktop app. It owns:

- Configuration (`config::Settings`)
- CSV parsing (`csv`)
- Tabular data model backed by Polars (`dataframe::DataFrame`)
- Algorithm catalogue (`catalogue`)
- Domain model (`domain`: `TaskType`, `JobStatus`, `ArtefactFormat`, `Job`, `Dataset`, etc.)
- Error envelope (`error::LlmError`, `LlmResult<T>`)
- Training pipeline (`training::train`)
- Metrics, charts, insights (`insights`)
- Local filesystem storage (`storage`)
- ONNX export (`export::onnx`)

## Layout

```
src/
  lib.rs          — public modules + re-exports (Settings, domain types, LlmError)
  config.rs       — environment-driven Settings (DATABASE_URL, LLM_DATA_DIR, etc.)
  csv.rs          — CSV parsing/validation
  dataframe.rs    — DataFrame wrapper over polars::prelude::DataFrame
  domain.rs       — shared domain types (TaskType, JobStatus, ArtefactFormat, Job, Dataset, JobMetric)
  error.rs        — LlmError + LlmResult + error_body envelope
  export/         — ONNX export (onnx.rs)
  insights.rs     — task-appropriate metrics computation
  storage.rs      — local filesystem dataset/artefact storage
  training/       — training pipeline (train, TrainingOutput, model kinds)
  catalogue.rs    — algorithm catalogue (AlgorithmSpec, algorithms(), find())
```

## Key public API

- `llm_core::config::get_settings()` — resolve `Settings` from environment (cached via `OnceLock`)
- `llm_core::csv::parse_csv(text)` — parse & validate CSV text
- `llm_core::dataframe::DataFrame::from_csv_str(text)` / `from_csv_path(path)` — build a `DataFrame`
- `llm_core::training::train(&job, &df) -> LlmResult<TrainingOutput>` — run training
- `llm_core::export::onnx::export_onnx(&model) -> LlmResult<Vec<u8>>` — export model to ONNX bytes
- `llm_core::storage::{write_dataset, write_artefact, dataset_path, artefact_path, ensure_dirs}` — filesystem helpers
- `llm_core::catalogue::{algorithms(), find(id)}` — algorithm catalogue queries

## Conventions

- **No HTTP or database dependencies** — this crate must remain embeddable in both Axum and Tauri
- **Errors** use `LlmError` with status code + code + message; `LlmResult<T> = Result<T, LlmError>`
- **Polars** is the dataframe backend (`PolarsDataFrame = polars::prelude::DataFrame`)
- **ONNX** uses `onnx-protobuf = "=0.2.3"` which pins `protobuf = "=3.4.0"` — do not bump these independently
- All environment configuration goes through `config::Settings` — never `std::env::var` directly elsewhere

## Commands (run from `crates/`)

```sh
cargo check -p llm-core          # typecheck
cargo test -p llm-core           # unit tests
cargo build -p llm-core          # build
cargo clippy -p llm-core -- -D warnings  # lint
```

## Notes

- `protobuf` is pinned to `=3.4.0` because `onnx-protobuf = "=0.2.3"` hardcodes `VERSION_3_4_0` — changing it breaks compilation
- The `DataFrame` struct wraps a Polars `DataFrame` with `Utf8` columns so the same implementation serves uploaded text, stored datasets, and in-memory workers
