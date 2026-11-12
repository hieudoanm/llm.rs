# Contributing to llm.rs

Thanks for contributing to llm.rs (Local Learning Machine), a local-first
training studio for tabular data: Rust backend, Next.js + Tauri desktop UI,
and a static landing page.

## Repository layout

- `crates/` — pure-Rust crates: `llm-core`, `llm-db`, `llm-cli`.
- `packages/desktop` — Next.js + Tauri desktop app.
- `packages/landing` — static landing page.
- `docs/` — the normative specification (numbered docs govern).

See [AGENTS.md](AGENTS.md) for the spec index and conventions.

## Setup

```bash
git clone https://github.com/hieudoanm/llm.rs.git
cd llm.rs

# CLI
cd crates && cargo build --release -p llm-cli

# Desktop
cd ../packages/desktop && pnpm install && pnpm dev
```

Requires a stable Rust toolchain, Node.js 26, and pnpm.

## Checks

```bash
cd crates
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

cd ../packages/desktop
pnpm typecheck && pnpm lint && pnpm test && pnpm build
```

## Conventions

- `docs/` is the source of truth: when you change behaviour, versions, or
  ports, update the affected numbered documents in the same change.
- SQLite only — no PostgreSQL, no Docker dependency for the app itself.
- Keep ports and versions in sync across `.env.example`, `Makefile`,
  `docs/`, and `README.md`.

## License

By contributing, you agree that your contributions will be licensed under the
repository license in [`LICENSE`](LICENSE).
