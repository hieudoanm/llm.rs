#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXAMPLES_DIR="$REPO_ROOT/examples"
CLI_BIN="$REPO_ROOT/crates/target/release/llm"
OUTPUT_DIR="$EXAMPLES_DIR/out"

BOLD="\033[1m"; DIM="\033[2m"; GREEN="\033[32m"; CYAN="\033[36m"; RESET="\033[0m"
step() { echo; echo -e "${BOLD}${CYAN}==> $1${RESET}"; }
ok()   { echo -e "  ${GREEN}✓${RESET} $1"; }

ensure_binary() {
  if [[ ! -x "$CLI_BIN" ]]; then
    step "Building llm CLI"
    cd "$REPO_ROOT/crates"
    cargo build --release -p llm-cli
  fi
}

run_example() {
  local name="$1" csv="$2" task="$3" algo="$4" features="$5" label="$6"
  local out="$OUTPUT_DIR/${name}.onnx"
  step "[$name] Upload dataset"
  local dataset_id
  dataset_id=$("$CLI_BIN" dataset "$csv" | grep 'id:' | awk '{print $2}')
  ok "Dataset ID: $dataset_id"

  step "[$name] Train ($task / $algo)"
  local job_id
  job_id=$("$CLI_BIN" train \
    --dataset-id "$dataset_id" \
    --task "$task" \
    --algorithm "$algo" \
    --features "$features" \
    --label "$label" \
    | grep '^Training job' | awk '{print $3}' | tr -d '...')
  ok "Job ID: $job_id"

  step "[$name] Export ONNX"
  "$CLI_BIN" export "$job_id" "$out" --format onnx >/dev/null
  ok "ONNX artefact: $out"
}

echo -e "${BOLD}llm.rs — end-to-end examples${RESET}"
ensure_binary
mkdir -p "$OUTPUT_DIR"

run_example \
  "linear_regression" \
  "$EXAMPLES_DIR/csv/regression/linear_regression.csv" \
  "regression" "linear_regression" "x" "y"

echo
echo -e "${BOLD}${GREEN}Done${RESET}"