#!/usr/bin/env bash
set -e

echo "Running coverage for llm-core (threshold: 80%)"
cargo llvm-cov -p llm-core --fail-under-lines 80 --fail-under-functions 80 --summary-only

echo -e "\nRunning coverage for llm-db (threshold: 80%)"
cargo llvm-cov -p llm-db --fail-under-lines 80 --fail-under-functions 80 --summary-only

echo -e "\nRunning coverage for llm-cli (threshold: 80%)"
cargo llvm-cov -p llm-cli --fail-under-lines 80 --fail-under-functions 80 --summary-only
