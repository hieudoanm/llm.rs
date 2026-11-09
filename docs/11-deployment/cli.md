# CLI — llm

Part of [`11-deployment.md`](../11-deployment.md). Documents the `llm`
command-line interface: offline train-and-export from a CSV, installation,
and usage.

## Overview

`llm` is the llm.rs command-line client, implemented in Rust in
[`crates/llm-cli`](../../crates/llm-cli). It registers CSV datasets, trains
models, and exports portable ONNX artefacts. Dataset bytes and job records
are persisted to the local SQLite database (`DATABASE_URL`, default
`.llm-data/llm.db`) and the filesystem data directory (`LLM_DATA_DIR`).

## Installation

### Option 1 — Prebuilt binary

Download the `llm` binary from the `latest` GitHub Release and place it on
your `PATH`:

```bash
curl -fsSL https://github.com/hieudoanm/llm.rs/releases/download/cli-latest/llm \
  -o "$HOME/.local/bin/llm"
chmod +x "$HOME/.local/bin/llm"
```

Ensure `$HOME/.local/bin` is on your `PATH`, then:

```bash
llm --help
```

### Option 2 — From source

```bash
# From crates/
cargo build --release -p llm-cli
# produces target/release/llm
```

The binary is a single-file executable with no runtime dependencies.

## Usage

```bash
# Show help
llm --help

# List algorithms (optionally filtered by task type)
llm algorithms --task classification

# Register a dataset
llm dataset data/iris.csv

# Train a model
llm train \
  --dataset-id <uuid> \
  --task classification \
  --algorithm logistic_regression \
  --features sepal_length,sepal_width,petal_length,petal_width \
  --label species

# Check job status and metrics
llm job <job-id>

# List jobs and datasets
llm jobs
llm datasets

# Export the ONNX artefact
llm export <job-id> model.onnx --format onnx
```

### Commands

| Command | Purpose |
|---------|---------|
| `llm algorithms [--task <type>]` | List available algorithms |
| `llm dataset <path>` | Upload/register a CSV dataset |
| `llm train` | Train a model and export ONNX |
| `llm job <id>` | Show job status, metrics, and artefacts |
| `llm export <job-id> <output> [--format onnx\|pkl\|joblib]` | Export a job's artefact |
| `llm jobs` | List all jobs |
| `llm datasets` | List all stored datasets |

### `llm train` flags

| Flag | Purpose |
|------|---------|
| `--dataset-id` | Dataset UUID from `llm dataset` (required) |
| `--task` | Task type: `classification`, `regression`, `semi_supervised`, `clustering`, `anomaly_detection`, `reinforcement_learning` (required) |
| `--algorithm` | Algorithm id; see `llm algorithms` (required) |
| `--features` | Comma-separated feature column names (required) |
| `--label` | Label column; required for supervised / semi-supervised tasks |
| `--hyperparameters` | JSON string of algorithm hyperparameters (default `{}`) |
| `--split-ratio` | Train/test split ratio, 0.0–1.0 (default `0.2`) |
| `--random-seed` | Random seed (default `42`) |

## Configuration

`llm` honours the shared environment configuration documented in
[`environment-variables.md`](./environment-variables.md): `DATABASE_URL`,
`LLM_DATA_DIR`, `LLM_LOG_LEVEL`, `RUST_LOG`, and the `LLM_*` job/worker
tunables. Settings are resolved once at startup.

## Development

From `crates/`:

```bash
cargo build --release -p llm-cli   # build the binary
cargo check -p llm-cli             # typecheck
cargo test -p llm-cli              # unit tests
cargo clippy -p llm-cli -- -D warnings
```

### Layout

```
crates/llm-cli/
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs       # entry point, logging, command dispatch
    ├── cli.rs        # clap subcommand definitions
    └── commands/     # per-command handlers (dataset, train, job, export, ...)
```
