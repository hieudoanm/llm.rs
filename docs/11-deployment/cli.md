# CLI — mlrs

Part of [`11-deployment.md`](../11-deployment.md). Documents the `mlrs`
command-line interface: offline train-and-export from a CSV, binary
installation, and usage.

## Overview

`mlrs` is the scikit-learn-client CLI. `mlrs run` trains a scikit-learn model
from a CSV and exports a portable artefact (ONNX / PKL / JobLib) entirely
offline — no backend, no database, no network required. Every input (file
location, features, label, algorithm, output location) can be passed as flags
or supplied interactively.

The CLI is distributed as a single-file PyInstaller binary attached to the
`latest` GitHub Release, and it can also be installed from source with `uv` or
`pip`.

## Installation

### Option 1 — Prebuilt binary (recommended)

Download the `mlrs` binary from the `latest` release and place it on your PATH:

```bash
curl -fsSL https://github.com/hieudoanm/scikit-learn-client/releases/download/latest/mlrs \
  -o "$HOME/.local/bin/mlrs"
chmod +x "$HOME/.local/bin/mlrs"
```

Ensure `$HOME/.local/bin` is on your `PATH`, then:

```bash
mlrs --help
```

There is also a maintained installer script in the repository:

```bash
bash crates/mlrs-server/scripts/install.sh
```

It downloads the `latest` release binary to `$HOME/.local/bin/mlrs`.

### Option 2 — From source

```bash
# From crates/mlrs-server
cargo build --release
# produces target/release/mlrs
```

### Build a standalone binary

```bash
# From crates/mlrs-server
cargo build --release   # produces target/release/mlrs (single-file executable)
./target/release/mlrs --help
```

## Usage

```bash
# Show help
mlrs --help
mlrs run --help

# Train from a CSV and export ONNX (all flags, non-interactive)
mlrs run data/train.csv \
  --features x1,x2 \
  --label y \
  --task-type regression \
  --algorithm linear_regression \
  --format onnx \
  --output ./out/model.onnx \
  --no-interactive

# Fully interactive (prompts for every missing input)
mlrs run data/train.csv

# Clustering / anomaly detection (no label required)
mlrs run data/train.csv \
  --features x1,x2 \
  --task-type clustering \
  --algorithm kmeans \
  --format onnx \
  --no-interactive
```

### `mlrs run` flags

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

When a required input is omitted and stdin is a TTY, `mlrs` prompts
interactively (`inquirer`). With `--no-interactive` or a non-TTY stdin, a
missing required input causes a non-zero exit.

## Environment

| Variable       | Default                 | Description                      |
|----------------|-------------------------|----------------------------------|

`mlrs` reads no environment variables. It is fully offline: the `run` command
never contacts a backend.

## Development

From `crates/mlrs-server`:

```bash
make lint    # ruff check src tests
make format  # ruff format + ruff check --fix
make test    # pytest tests -v --cov=src/mlrs --cov-report=term-missing
make build   # pyinstaller --onefile --name mlrs src/mlrs/pyinstaller_entry.py
```

### Layout

```
crates/mlrs-server/
├── Cargo.toml   # packaging, dev deps (clippy, fmt)
├── mlrs.spec         # PyInstaller spec (kept for reference; CI/build use the entry point)
├── Makefile         # lint / test / format / build
├── scripts/
│   └── install.sh   # downloads the latest release binary
├── src/mlrs/
│   ├── __init__.py   # package entry, exposes main()
│   ├── cli.py        # click group: run (offline)
│   ├── helpers.py    # inquirer prompts, tabulate display, formatting
│   ├── local.py      # offline train+export pipeline (no backend)
│   ├── pipeline.py   # mlrs training pipelines per task type
│   ├── estimator.py  # estimator construction
│   ├── export.py     # ONNX / pickle / joblib export
│   ├── preprocessor.py  # mlrs ColumnTransformer preprocessor
│   ├── pyinstaller_entry.py  # frozen entry point (package context preserved)
│   └── catalogue/    # algorithm catalogue (mirrors the backend)
├── tests/            # pytest suite
└── examples/         # demo CSV and run script
```

**See also:** [`local-development.md`](./local-development.md),
[`environment-variables.md`](./environment-variables.md).
