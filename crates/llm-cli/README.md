# llm-cli

Command-line interface for [llm.rs](../../README.md). Train ML models from CSV files and export them as ONNX artefacts.

## Installation

From the `crates/` directory:

```sh
cargo build --release -p llm-cli
```

The binary will be at `target/release/llm`.

## Usage

```sh
llm --help
```

### Commands

- `llm algorithms [--task <task>]` - List available algorithms
- `llm dataset <path>` - Upload/register a CSV dataset
- `llm train ...` - Train a model from a dataset
- `llm job <id>` - Show job status and metrics
- `llm export --job-id <id> --output <path> [--format onnx|pkl|joblib]` - Export artefact
- `llm artefacts` - List artefacts
- `llm jobs` - List jobs

### Example

```sh
# Register dataset
llm dataset data/iris.csv

# Train classifier
llm train \
  --dataset-id <id> \
  --task classification \
  --algorithm logistic_regression \
  --features sepal_length,sepal_width,petal_length,petal_width \
  --label species

# Check status
llm job <job_id>

# Export ONNX
llm export --job-id <job_id> --output model.onnx
```

## Dependencies

Uses `llm-core` for training logic and `llm-db` for persistence.

## Testing

```sh
cargo test -p llm-cli
```
