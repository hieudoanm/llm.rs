#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXAMPLES_DIR="$REPO_ROOT/examples"
CLI_BIN="$REPO_ROOT/crates/target/release/llm"
CSV_FILE="$EXAMPLES_DIR/examples.csv"
OUTPUT_DIR="$EXAMPLES_DIR/out"
MODEL_OUT="$OUTPUT_DIR/model.onnx"

# Colors
BOLD="\033[1m"
DIM="\033[2m"
GREEN="\033[32m"
CYAN="\033[36m"
RESET="\033[0m"

step() { echo; echo -e "${BOLD}${CYAN}==> $1${RESET}"; }
ok()   { echo -e "  ${GREEN}✓${RESET} $1"; }

echo -e "${BOLD}llm.rs — end-to-end example: CSV to ONNX${RESET}"
echo -e "${DIM}Repository root: $REPO_ROOT${RESET}"
echo -e "${DIM}CSV file:        $CSV_FILE${RESET}"

# Ensure the CLI binary is built
if [[ ! -x "$CLI_BIN" ]]; then
  step "Building llm CLI (cargo build --release -p llm-cli)"
  cd "$REPO_ROOT/crates"
  cargo build --release -p llm-cli
fi

mkdir -p "$OUTPUT_DIR"

step "Step 1: Upload dataset"
DATASET_ID=$("$CLI_BIN" dataset "$CSV_FILE" | grep 'id:' | awk '{print $2}')
ok "Dataset ID: $DATASET_ID"

step "Step 2: Train a regression model (x -> y)"
JOB_ID=$("$CLI_BIN" train \
  --dataset-id "$DATASET_ID" \
  --task regression \
  --algorithm linear_regression \
  --features "x" \
  --label "y" \
  | grep '^Training job' | awk '{print $3}' | tr -d '...')
ok "Job ID: $JOB_ID"

step "Step 3: Show job status"
"$CLI_BIN" job "$JOB_ID"

step "Step 4: Export ONNX artefact"
"$CLI_BIN" export "$JOB_ID" "$MODEL_OUT" --format onnx
ok "ONNX artefact: $MODEL_OUT"

step "Step 5: Test prediction with exported ONNX model"
python3 - "$MODEL_OUT" <<'PY'
import sys, onnx, numpy as np
path = sys.argv[1]
m = onnx.load(path)
W = B = None
for t in m.graph.initializer:
    arr = onnx.numpy_helper.to_array(t)
    if t.name == "W": W = arr
    elif t.name == "B": B = arr
xs = np.array([[1.0],[2.0],[5.0]], dtype=np.float32)
pred = xs @ W.T + B
for x, y in zip(xs.flatten(), pred.flatten()):
    print(f"  x={x:g} -> y={y:.4f}")
PY

echo
echo -e "${BOLD}${GREEN}Done${RESET}"
echo -e "  ONNX artefact: $MODEL_OUT"
echo -e "  Job ID:        $JOB_ID"
echo -e "  Dataset ID:    $DATASET_ID"