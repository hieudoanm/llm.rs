#!/usr/bin/env bash
#
# Convert OpenAPI YAML specs to JSON and Postman collections.
#
# This script installs dependencies (if needed) and runs the Python converter
# for every `<service>/openapi.yaml` under this directory.
#
# Usage:
#   ./docs/06-api-specification/openapi.sh

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "${script_dir}"

echo "Installing dependencies..."
uv sync

echo "Format scripts..."
uv run ruff format

echo "Converting OpenAPI YAML specs..."
uv run openapi.py
