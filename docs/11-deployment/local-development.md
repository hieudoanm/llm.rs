# Local Development

## Prerequisites

- Rust toolchain (cargo)
- Node.js 26+ and pnpm (desktop frontend)
- Tauri system dependencies for `packages/desktop` (see the
  [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/))
- Git

## Repository structure

```text
llm.rs/
├── crates/
│   ├── Cargo.toml    # Rust workspace
│   ├── llm-core/     # Pure-Rust core (config, CSV, training, ONNX export, storage)
│   ├── llm-db/       # SQLite persistence (sqlx)
│   └── llm-cli/      # `llm` command-line binary
├── packages/
│   ├── desktop/      # Next.js + Tauri desktop app
│   └── landing/      # Static landing page (plain HTML/CSS)
├── docs/             # Product & implementation specification
├── demo/             # Recorded demo video and assets
└── .env.example      # Environment template (copy to .env)
```

## Configuration

Copy `.env.example` to `.env` to override defaults. The full variable list —
`DATABASE_URL`, `LLM_DATA_DIR`, `LLM_LOG_LEVEL`, `RUST_LOG`, and the `LLM_*`
job/worker tunables — is documented in
[`environment-variables.md`](./environment-variables.md), which is the single
source of truth. Unset variables fall back to the defaults compiled into
`llm-core`.

## Workflow

### Rust workspace

```bash
cd crates
cargo build --release -p llm-cli   # build the `llm` binary
cargo test --workspace             # run all Rust tests
cargo check --workspace --all-targets
```

Or from the repository root: `make cli`, `make cli-test`, `make rust-check`,
`make rust-test`.

### Desktop app

```bash
cd packages/desktop
pnpm install
pnpm dev          # Next.js dev server + Tauri dev concurrently
pnpm lint         # eslint
pnpm exec tsc --noEmit   # typecheck
pnpm test         # Vitest unit tests
pnpm build        # production Next.js build
pnpm build:tauri  # Tauri bundle
```

Or from the repository root: `make desktop-dev`, `make desktop-build`,
`make desktop-test`.

### CLI

The `llm` CLI is a standalone tool backed by the same SQLite database and
data directory as the desktop app. See [`cli.md`](./cli.md) for the full
command reference; source lives in `crates/llm-cli/`.

## Tests

```bash
make test         # Rust workspace tests + desktop Vitest suite
make cli-test     # llm-cli tests only
make desktop-test # desktop frontend tests only
```

Desktop e2e (Playwright) specs live in `packages/desktop/e2e/`; see
`packages/desktop/AGENTS.md` for how to run them.

## Data

Both the CLI and the desktop app persist to `.llm-data/` by default: the
SQLite database at `DATABASE_URL` (default `.llm-data/llm.db`) plus uploaded
datasets and ONNX artefacts under `LLM_DATA_DIR`.

To reset local state, stop the app and remove the data directory:

```bash
rm -rf .llm-data
```
