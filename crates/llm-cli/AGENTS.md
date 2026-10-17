# AGENTS.md — crates/llm-cli

Guidance for AI agents working on `llm-cli`, the command-line interface for llm.rs.

## Purpose

`llm-cli` is the standalone `llm` binary that trains ML models from CSV files and exports ONNX artefacts, all from the command line. It uses `llm-core` for training logic and `llm-db` for persistence.

## Entry point

- `src/main.rs` — `#[tokio::main] async fn main()` with a `clap::Parser`-derived CLI

## Subcommands

| Subcommand | Purpose |
|---|---|
| `llm algorithms [--task <type>]` | List available algorithms, optionally filtered by task type |
| `llm dataset <path>` | Upload/register a CSV dataset (reads file, parses CSV, stores bytes + metadata) |
| `llm train --dataset-id <id> --task <type> --algorithm <id> --features <cols> [--label <col>] [--hyperparameters <json>] [--split-ratio <f>] [--random-seed <n>]` | Train a model and export ONNX |
| `llm job <id>` | Show job status, metrics, and artefacts |
| `llm jobs` | List all jobs |
| `llm datasets` | List all stored datasets |

## Layout

```
src/
  main.rs — CLI definition (clap subcommands) + all command handlers (cmd_*)
```

## Key flow (`cmd_train`)

1. Parse dataset UUID, load dataset from DB, read CSV from storage
2. Validate algorithm exists and matches the task type
3. Create `Job` record, insert into DB, set status to `Training`
4. Call `llm_core::training::train(&job, &df)` → `TrainingOutput`
5. Set status to `Exporting`, export model to ONNX via `llm_core::export::onnx::export_onnx`
6. Write artefact to filesystem, insert artefact record, record metrics
7. Set status to `Completed`

## Conventions

- **Error handling** — all commands return `anyhow::Result<()>`; failures print an error and exit non-zero
- **Logging** — `tracing_subscriber` with level from `Settings.log_level`
- **Settings** — resolved once via `llm_core::config::get_settings()`
- **Database** — `llm_db::Database::connect(settings)` opens the SQLite pool and runs migrations

## Dependencies

- `llm-core` — training, catalogue, domain, storage, config, error
- `llm-db` — SQLite persistence (Database, repositories)
- `clap` (derive) — CLI argument parsing
- `anyhow` — error handling
- `tokio` (full) — async runtime
- `tracing`, `tracing-subscriber` — logging
- `serde`, `serde_json` — JSON (hyperparameters, metrics)
- `uuid` — job/dataset IDs
- `polars` — dataframe (re-exported via llm-core)

## Commands (run from `crates/`)

```sh
cargo build --release -p llm-cli   # build release binary
./target/release/llm --help        # show CLI help
cargo check -p llm-cli             # typecheck
cargo test -p llm-cli              # unit tests
cargo clippy -p llm-cli -- -D warnings  # lint
```

## Example session

```sh
# List algorithms
./target/release/llm algorithms --task classification

# Upload a dataset
./target/release/llm dataset data.csv

# Train a model
./target/release/llm train \
  --dataset-id <uuid> \
  --task classification \
  --algorithm logistic_regression \
  --features "sepal_length,sepal_width,petal_length,petal_width" \
  --label species

# Check job status
./target/release/llm job <job-id>

# Export artefact
./target/release/llm export --job-id <job-id> --output model.onnx
```
