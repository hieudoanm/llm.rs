# CLI — llm

Part of [`11-deployment.md`](../11-deployment.md). Documents the `llm`
command-line interface: offline train-and-export from a CSV, binary
installation, and usage.

## Overview

`llm` is the llm-client CLI. `llm run` trains a llm model
from a CSV and exports a portable artefact (ONNX / PKL / JobLib) entirely
offline — no backend, no database, no network required. Every input (file
location, features, label, algorithm, output location) can be passed as flags
or supplied interactively.

The CLI is distributed as a single-file PyInstaller binary attached to the
`latest` GitHub Release, and it can also be installed from source with `uv` or
`pip`.

## Installation

### Option 1 — Prebuilt binary (recommended)

Download the `llm` binary from the `latest` release and place it on your PATH:

```bash
curl -fsSL https://github.com/hieudoanm/llm-client/releases/download/latest/llm \
  -o "$HOME/.local/bin/llm"
chmod +x "$HOME/.local/bin/llm"
```

Ensure `$HOME/.local/bin` is on your `PATH`, then:

```bash
llm --help
```

There is also a maintained installer script in the repository:

```bash
bash crates/llm-server/scripts/install.sh
```

It downloads the `latest` release binary to `$HOME/.local/bin/llm`.

### Option 2 — From source

```bash
# From crates/llm-server
cargo build --release
# produces target/release/llm
```

### Build a standalone binary

```bash
# From crates/llm-server
cargo build --release   # produces target/release/llm (single-file executable)
./target/release/llm --help
```

## Usage

```bash
# Show help
llm --help
llm run --help

# Train from a CSV and export ONNX (all flags, non-interactive)
llm run data/train.csv \
  --features x1,x2 \
  --label y \
  --task-type regression \
  --algorithm linear_regression \
  --format onnx \
  --output ./out/model.onnx \
  --no-interactive

# Fully interactive (prompts for every missing input)
llm run data/train.csv

# Clustering / anomaly detection (no label required)
llm run data/train.csv \
  --features x1,x2 \
  --task-type clustering \
  --algorithm kmeans \
  --format onnx \
  --no-interactive
```

### `llm run` flags

| Flag               | Purpose                                                          |
|--------------------|------------------------------------------------------------------|
| `csv_path` (arg)   | Path to the input CSV with a header row (required)               |
| `--output`, `-o`   | Output artefact path (default: `<csv stem>.<ext>`)               |
| `--task-type`, `-t`| Task type: `classification`, `regression`, `semi_supervised`, `clustering`, `anomaly_detection` |
| `--features`, `-f` | Comma-separated feature columns                                  |
| `--label`, `-l`    | Label column (required for supervised / semi-supervised tasks)   |
| `--algorithm`, `-a`| Algorithm id (prompted if omitted)                               |
| `--format`         | Export format: `onnx`, `pkl`, `joblib` (default: `onnx`)         |
| `--no-interactive` | Fail instead of prompting for missing input                      |

When a required input is omitted and stdin is a TTY, `llm` prompts
interactively (`inquirer`). With `--no-interactive` or a non-TTY stdin, a
missing required input causes a non-zero exit.

## Environment

| Variable       | Default                 | Description                      |
|----------------|-------------------------|----------------------------------|

`llm` reads no environment variables. It is fully offline: the `run` command
never contacts a backend.

## Development

From `crates/llm-server`:

```bash
make lint    # ruff check src tests
make format  # ruff format + ruff check --fix
make test    # pytest tests -v --cov=src/llm --cov-report=term-missing
make build   # pyinstaller --onefile --name llm src/llm/pyinstaller_entry.py
```

### Layout

```
crates/llm-server/
├── Cargo.toml   # packaging, dev deps (clippy, fmt)
├── llm.spec         # PyInstaller spec (kept for reference; CI/build use the entry point)
├── Makefile         # lint / test / format / build
├── scripts/
│   └── install.sh   # downloads the latest release binary
├── src/llm/
│   ├── __init__.py   # package entry, exposes main()
│   ├── cli.py        # click group: run (offline)
│   ├── helpers.py    # inquirer prompts, tabulate display, formatting
│   ├── local.py      # offline train+export pipeline (no backend)
│   ├── pipeline.py   # llm training pipelines per task type
│   ├── estimator.py  # estimator construction
│   ├── export.py     # ONNX / pickle / joblib export
│   ├── preprocessor.py  # llm ColumnTransformer preprocessor
│   ├── pyinstaller_entry.py  # frozen entry point (package context preserved)
│   └── catalogue/    # algorithm catalogue (mirrors the backend)
├── tests/            # pytest suite
└── examples/         # demo CSV and run script
```

**See also:** [`local-development.md`](./local-development.md),
[`environment-variables.md`](./environment-variables.md).
