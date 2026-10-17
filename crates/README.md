# llm.rs Crates

This workspace contains the Rust crates that power [llm.rs](../README.md): a local-first machine learning training studio that trains models from CSV files and exports them as ONNX.

## Crates

| Crate                    | Description                                                                                                                                                                                                    |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`llm-core`](./llm-core) | Core ML logic: configuration, CSV parsing, dataframes, catalogue of algorithms, training pipelines (supervised, semi-supervised, unsupervised, offline RL), preprocessing, ONNX export, and storage utilities. |
| [`llm-db`](./llm-db)     | SQLite persistence layer (via `sqlx`). Manages datasets, training jobs, artefacts, idempotency keys, and schema migrations.                                                                                    |
| [`llm-cli`](./llm-cli)   | Command-line interface for training models from CSV and exporting ONNX artefacts.                                                                                                                              |

## Building

From the `crates/` directory:

```sh
cargo build --workspace
cargo test --workspace
```

Or build a specific crate:

```sh
cargo build -p llm-cli --release
```

## Workspace Layout

- Uses Cargo workspace with shared dependencies defined in `[workspace.dependencies]`
- All crates share workspace version/edition/license
- `llm-cli` depends on both `llm-core` and `llm-db`
- `llm-db` depends on `llm-core`
