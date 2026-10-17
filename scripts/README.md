# llm.rs Scripts

This directory contains installation and setup scripts for the llm.rs project.

## install.sh

Rebuilds and installs every llm.rs application:

```bash
./scripts/install.sh
```

This is equivalent to `./scripts/install.sh --all` and will:

1. Check for required dependencies (Rust/Cargo, Node.js/pnpm)
2. Install the desktop JavaScript dependencies
3. Build the llm CLI from source using `cargo build --release --locked`
4. Install the llm CLI to `~/.local/bin/llm` (and `~/.cargo/bin/llm` when present)
5. Build the native Tauri desktop application and install `Local Learning Machine.app`
   to `/Applications` or `~/Applications`

Every run builds new artifacts; it does not reuse an existing desktop bundle.

### Installing a single target

Pass one or more target flags to install the specified targets. If no targets are
selected, all components are installed by default.

```bash
./scripts/install.sh --cli          # llm CLI only
./scripts/install.sh --desktop      # Desktop app only
./scripts/install.sh --cli --desktop # CLI plus desktop (default)
```

Use `./scripts/install.sh --help` for the full option list.

To include optional Cargo features in both the CLI and desktop build, set
`LLM_CARGO_FEATURES`, for example:

```bash
LLM_CARGO_FEATURES=experimental ./scripts/install.sh
```

### Requirements

- Rust and Cargo (install from https://rustup.rs/) when installing the CLI or
  desktop app
- Node.js 26 or newer with pnpm when installing the desktop app
- Tauri desktop prerequisites for the current platform when installing the
  desktop app (system dependencies such as `libwebkit2gtk-4.1-dev` and
  `libssl-dev` on Linux)

On macOS, the desktop build produces `Local Learning Machine.app` under
`packages/desktop/src-tauri/target/release/bundle/macos/`. The script installs
the `.app`.

### Notes

The script adds `~/.local/bin` to PATH for future shells when it is missing. To
make this permanent, add the following to your shell profile (`~/.zshrc` or
`~/.bashrc`):

```bash
export PATH="$HOME/.local/bin:$PATH"
```
