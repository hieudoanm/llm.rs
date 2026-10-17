#!/usr/bin/env bash
set -euo pipefail

export NODE_NO_WARNINGS=1

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRATES_DIR="$REPO_ROOT/crates"
DESKTOP_DIR="$REPO_ROOT/packages/desktop"
LOCAL_BIN="$HOME/.local/bin"
CARGO_BIN="$HOME/.cargo/bin"

INSTALL_CLI=0
INSTALL_DESKTOP=0

usage() {
  cat <<'USAGE'
Usage: ./scripts/install.sh [options]

Options:
  --all        Install CLI and desktop app (default)
  --cli        Install only the llm CLI
  --desktop    Install only the desktop app
  -h, --help   Show this help

Environment:
  LLM_CARGO_FEATURES  Cargo features to pass to CLI and desktop builds

Note: the desktop app embeds the kernel and does not need the CLI separately,
but both can be installed together with `./scripts/install.sh --cli --desktop`.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --all)
      INSTALL_CLI=1
      INSTALL_DESKTOP=1
      shift
      ;;
    --cli)
      INSTALL_CLI=1
      INSTALL_DESKTOP=0
      shift
      ;;
    --desktop)
      INSTALL_CLI=0
      INSTALL_DESKTOP=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      usage
      exit 1
      ;;
  esac
done

# Default to all if nothing was specified
if [[ "$INSTALL_CLI" -eq 0 && "$INSTALL_DESKTOP" -eq 0 ]]; then
  INSTALL_CLI=1
  INSTALL_DESKTOP=1
fi

echo "Installing llm.rs"
echo "Repository root: $REPO_ROOT"
echo "Targets: CLI=${INSTALL_CLI} Desktop=${INSTALL_DESKTOP}"

step() {
  echo
  echo "==> $1"
}

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Error: '$1' not found"
    echo "$2"
    exit 1
  fi
}

if [[ "$INSTALL_CLI" -eq 1 || "$INSTALL_DESKTOP" -eq 1 ]]; then
  require_command cargo "Please install Rust from https://rustup.rs/"
fi
if [[ "$INSTALL_DESKTOP" -eq 1 ]]; then
  require_command node "Please install Node.js 26 or newer."
  require_command pnpm "Please install pnpm."
fi

CARGO_ARGS=(build --release --locked --bin llm)
TAURI_ARGS=(build)
if [ -n "${LLM_CARGO_FEATURES:-}" ]; then
  CARGO_ARGS+=(--features "$LLM_CARGO_FEATURES")
fi

step "Installing JavaScript dependencies (desktop)"
if command -v pnpm >/dev/null 2>&1; then
  cd "$REPO_ROOT"
  CI=1 npm_config_manage_package_manager_versions=false pnpm install --frozen-lockfile
else
  echo "Error: pnpm not found"
  echo "Please install pnpm from https://pnpm.io/"
  exit 1
fi

if [[ "$INSTALL_CLI" -eq 1 ]]; then
  step "Building and installing the llm CLI"
  cd "$REPO_ROOT"
  cargo "${CARGO_ARGS[@]}"

  mkdir -p "$LOCAL_BIN"
  install -m 755 "$REPO_ROOT/target/release/llm" "$LOCAL_BIN/llm"
  if [ -d "$CARGO_BIN" ]; then
    install -m 755 "$REPO_ROOT/target/release/llm" "$CARGO_BIN/llm"
  fi
  echo "✓ llm CLI installed to $LOCAL_BIN/llm"

  if [[ ":$PATH:" != *":$LOCAL_BIN:"* ]]; then
    echo "Add the following to your shell profile (~/.zshrc or ~/.bashrc):"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
  fi
else
  echo
  echo "Skipping CLI build/install"
fi

if [[ "$INSTALL_DESKTOP" -eq 1 ]]; then
  step "Building and installing the desktop application"
  cd "$DESKTOP_DIR"
  pnpm run build
  pnpm run build:tauri

  APP_BUNDLE=""
  for bundle_dir in \
    "$REPO_ROOT/packages/desktop/src-tauri/target/release/bundle/macos"; do
    if [ -d "$bundle_dir" ]; then
      APP_BUNDLE="$(find "$bundle_dir" -maxdepth 1 -type d -name '*.app' -print -quit)"
      if [ -n "$APP_BUNDLE" ]; then
        break
      fi
    fi
  done

  if [ -z "$APP_BUNDLE" ]; then
    echo "Error: no macOS .app bundle was produced by 'pnpm run build:tauri'."
    echo "Check the Tauri build output for the bundle location on this platform."
    exit 1
  fi

  if [ ! -w "/Applications" ]; then
    INSTALL_ROOT="$HOME/Applications"
    mkdir -p "$INSTALL_ROOT"
  else
    INSTALL_ROOT="/Applications"
  fi

  APP_NAME="$(basename "$APP_BUNDLE")"
  rm -rf "$INSTALL_ROOT/$APP_NAME"
  ditto "$APP_BUNDLE" "$INSTALL_ROOT/$APP_NAME"
  echo "✓ Desktop application installed to $INSTALL_ROOT/$APP_NAME"
else
  echo
  echo "Skipping desktop app build/install"
fi

step "Installation complete"
if [[ "$INSTALL_CLI" -eq 1 ]]; then
  echo "✓ llm CLI:       $LOCAL_BIN/llm"
fi
if [ -n "${APP_NAME:-}" ]; then
  echo "✓ Desktop app:   $INSTALL_ROOT/$APP_NAME"
fi
if [ -n "${LLM_CARGO_FEATURES:-}" ]; then
  echo "✓ Cargo features: $LLM_CARGO_FEATURES"
fi
