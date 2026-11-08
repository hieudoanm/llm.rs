# AGENTS.md — llm.rs (Local Learning Machine)

Guidance for AI agents (and orientation for humans) working in this repository.

## Read the specification first

The product and implementation specification lives in `docs/`. Start at the index, [`docs/README.md`](docs/README.md). The numbered documents `00`–`12` are normative: where the index or the code disagrees with them, the numbered documents govern. When you change behaviour, versions, or ports, update the affected documents in the same change.

| I need to…                          | Read                                                                                                                                                                 |
| ----------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Understand vision and scope         | [`docs/00-overview.md`](docs/00-overview.md)                                                                                                                         |
| Check requirements (`FR-`, `NFR-`)  | [`docs/01-requirements.md`](docs/01-requirements.md)                                                                                                                 |
| Understand user-visible behaviour   | [`docs/02-features.md`](docs/02-features.md) (`FEATURE-01`…`18`)                                                                                                     |
| Change architecture/API/data model  | [`docs/04-architecture.md`](docs/04-architecture.md), [`docs/05-data-model.md`](docs/05-data-model.md)                                                               |
| Style the UI (tokens, states, a11y) | [`docs/07-ui-ux-design.md`](docs/07-ui-ux-design.md)                                                                                                                 |
| Add or decide on a dependency       | [`docs/08-tech-stack.md`](docs/08-tech-stack.md)                                                                                                                     |
| Know how tests are organised        | [`docs/09-testing-strategy.md`](docs/09-testing-strategy.md)                                                                                                         |
| Touch CI or local setup             | [`docs/11-deployment/ci-cd-pipeline.md`](docs/11-deployment/ci-cd-pipeline.md), [`docs/11-deployment/local-development.md`](docs/11-deployment/local-development.md) |
| Operate: backup, rollback, security | [`docs/12-operations/`](docs/12-operations/)                                                                                                                         |

## Repository layout

| Path                 | What it is                                                                                                            |
| -------------------- | --------------------------------------------------------------------------------------------------------------------- |
| `crates/`            | Pure-Rust crates: `llm-core` (see `crates/llm-core/AGENTS.md`), `llm-db` (see `crates/llm-db/AGENTS.md`), `llm-cli` (see `crates/llm-cli/AGENTS.md`) |
| `packages/desktop`   | Next.js + Tauri desktop UI. See `packages/desktop/AGENTS.md`.                                                               |
| `packages/landing`   | Static landing page (`index.html` + `styles.css`, no framework). See `packages/landing/AGENTS.md`.                                        |
| `docs/`              | The specification — source of truth (see above).                                                                            |
| `demo/`              | Recorded demo video, produced by `make e2e-demo`.                                                                     |
| `.github/workflows/` | CI: Rust (cargo check + test), desktop (Tauri builds for all platforms), release artifacts (CLI + 7 desktop bundles). |

pnpm workspace (`pnpm-workspace.yaml`) driven by turbo: `pnpm run dev | lint | build | format | typecheck | test` (`format` and `typecheck` run Prettier/`tsc` for the desktop frontend; `test` runs the Vitest suite in `packages/desktop`). Rust: `cargo build | cargo test | cargo check` in `crates/`.

## Common commands

```sh
# CLI build and run
cd crates && cargo build --release -p llm-cli
./target/release/llm --help

# Desktop app development (Next.js + Tauri)
cd packages/desktop && pnpm dev  # runs Next.js dev + Tauri dev concurrently

# Desktop app production build
cd packages/desktop && pnpm build && pnpm build:tauri

# Rust build and test
cd crates && cargo build
cd crates && cargo test

# Full release (CLI binary + 7 desktop artifacts)
# Handled by GitHub Actions on tag push
```

## Environment and conventions

- **Node 26** (`.node-version`, `.nvmrc`) with pnpm.
- **SQLite** only — no PostgreSQL, no Docker. The app persists to `.llm-data/llm.db` by default.
- `.env` (copied from `.env.example`, gitignored) parameterises the app: `LLM_DATA_DIR`, `LLM_MAX_UPLOAD_MB`, `LLM_LOG_LEVEL`, etc.
- Desktop e2e selectors: use role/label locators but avoid ambiguous `aria-label` values — Playwright strict mode fails on multiple matches.
- Keep ports and versions you touch in sync across `.env.example`, `Makefile`, `docs/`, and `README.md`.
