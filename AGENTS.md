# AGENTS.md — ml.rs (mlrs)

Guidance for AI agents (and orientation for humans) working in this repository.

## Read the specification first

The product and implementation specification lives in `docs/`. Start at the index, [`docs/README.md`](docs/README.md). The numbered documents `00`–`12` are normative: where the index or the code disagrees with them, the numbered documents govern. When you change behaviour, versions, or ports, update the affected documents in the same change.

| I need to…                          | Read                                                                                                                                                                   |
| ----------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Understand vision and scope         | [`docs/00-overview.md`](docs/00-overview.md)                                                                                                                           |
| Check requirements (`FR-`, `NFR-`)  | [`docs/01-requirements.md`](docs/01-requirements.md)                                                                                                                   |
| Understand user-visible behaviour   | [`docs/02-features.md`](docs/02-features.md) (`FEATURE-01`…`18`)                                                                                                       |
| Change architecture/API/data model  | [`docs/04-architecture.md`](docs/04-architecture.md), [`docs/05-data-model.md`](docs/05-data-model.md), [`docs/06-api-specification.md`](docs/06-api-specification.md) |
| Style the UI (tokens, states, a11y) | [`docs/07-ui-ux-design.md`](docs/07-ui-ux-design.md)                                                                                                                   |
| Add or decide on a dependency       | [`docs/08-tech-stack.md`](docs/08-tech-stack.md)                                                                                                                       |
| Know how tests are organised        | [`docs/09-testing-strategy.md`](docs/09-testing-strategy.md)                                                                                                           |
| Touch CI or local setup             | [`docs/11-deployment/ci-cd-pipeline.md`](docs/11-deployment/ci-cd-pipeline.md), [`docs/11-deployment/local-development.md`](docs/11-deployment/local-development.md)   |
| Operate: backup, rollback, security | [`docs/12-operations/`](docs/12-operations/)                                                                                                                           |

## Repository layout

| Path                 | What it is                                                                                                                  |
| -------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| `crates/`            | Pure-Rust backend: `mlrs-core`, `mlrs-db`, `mlrs-api`, `mlrs-server`                                                        |
| `packages/frontend`  | Next.js App Router UI (TypeScript, Tailwind CSS, DaisyUI, Playwright). See `packages/frontend/AGENTS.md`.                   |
| `docs/`              | The specification — source of truth (see above).                                                                            |
| `demo/`              | Recorded demo video, produced by `make e2e-demo`.                                                                           |
| `.github/workflows/` | CI: Rust (cargo check + test), frontend (eslint + tsc + build), e2e (Playwright) — SQLite only, no PostgreSQL.              |

pnpm workspace (`pnpm-workspace.yaml`) driven by turbo: `pnpm run dev | lint | build | format | typecheck | test` (`format` and `typecheck` run Prettier/`tsc` for the frontend; `test` runs the Vitest suite in `packages/frontend`). Rust: `cargo build | cargo test | cargo check` in `crates/`.

## Common commands

```sh
# Full local stack: backend :8000, frontend :3000 (SQLite, no PostgreSQL)
docker compose up -d --build

# The same stack via make
make up        # / make down

# Host mode instead: backend/frontend on the host
make start   # / make stop

# Backend API health check (needs the stack up)
curl http://127.0.0.1:8000/healthz

# Playwright e2e (needs backend :8000 + frontend :3000)
make e2e
# or: cd packages/frontend && pnpm exec playwright test

# Frontend unit tests (Vitest + React Testing Library; `test:coverage` is the
# 80% gate that CI runs)
cd packages/frontend && pnpm test

# Rust build and test
cd crates && cargo build
cd crates && cargo test
```

## Environment and conventions

- **Node 26** (`.node-version`, `.nvmrc`) with pnpm.
- **SQLite** only — no PostgreSQL. The backend persists to `.mlrs-data/mlrs.db` by default.
- `.env` (copied from `.env.example`, gitignored) parameterises compose: `MLRS_DATA_DIR`, `MLRS_MAX_UPLOAD_MB`, `MLRS_LOG_LEVEL`, `MLRS_CORS_ORIGINS`, etc.
- Frontend e2e selectors: use role/label locators but avoid ambiguous `aria-label` values — Playwright strict mode fails on multiple matches.
- Keep ports and versions you touch in sync across `docker-compose.yaml`, `.env.example`, `Makefile`, `docs/`, and `README.md`.
