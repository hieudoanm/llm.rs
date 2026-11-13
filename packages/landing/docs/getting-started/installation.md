# Installation

## Option 1 — Docker Compose (full stack)

The fastest way to run the whole studio — frontend, backend, and SQLite:

```sh
docker compose up --build
```

## Option 2 — CLI

```sh
cd crates
cargo build --release -p llm-cli
./target/release/llm --help
```

## Option 3 — Desktop app

```sh
cd packages/desktop
pnpm install
pnpm dev            # Next.js + Tauri in development
pnpm build && pnpm build:tauri   # production bundles
```

## Release artifacts

GitHub Actions produces a `llm` CLI binary (Linux x64) and 7 desktop bundles:
macOS `.dmg`/`.app.zip`, Linux `.rpm`/`.deb`/`.AppImage`, Windows `.msi`/`.exe`.

## Prerequisites

- Rust stable toolchain (CLI)
- Node.js 26+ and pnpm (desktop app)
- Docker + Compose v2 (full stack)
- SQLite only — no PostgreSQL, no Docker required for the CLI
