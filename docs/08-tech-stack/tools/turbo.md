# mlrs — Turborepo

Task runner for the pnpm workspace.

## Choice

Turborepo 2 (root `devDependency`, pinned `2.11.9`).

## Why

- One entry point per task across `@mlrs/backend`, `@mlrs/notebooks`, and `@mlrs/frontend`.
- Dependency-aware ordering (`dependsOn: ["^build"]`) and local caching in `.turbo/` (gitignored).
- The whole task graph is one committed `turbo.json`; the root `package.json` holds no task logic.

## Rejected

- pnpm `--recursive` scripts — no task graph, no caching.
- Nx — heavier than this repo needs.

## Tasks

`turbo.json` declares `format`, `build`, `typecheck`, `lint`, `test`, and `dev`; each is cached and excludes `docs/**` and `images/**` from its inputs (`build` also declares outputs).

| Task        | `@mlrs/frontend`              | `@mlrs/backend` / `@mlrs/notebooks` |
| ----------- | ---------------------------- | ---------------------------- |
| `lint`      | `eslint`                     | `make lint` (Ruff)           |
| `format`    | `prettier --cache --write .` | `make format` (Ruff)         |
| `build`     | `next build`                 | —                            |
| `dev`       | `next dev`                   | —                            |
| `typecheck` | `tsc --noEmit`               | — (no Rust typecheck task) |
| `test`      | `vitest run`                 | — (pytest runs separately)   |

Root scripts wrap the tasks: `pnpm run dev | lint | build | format | typecheck | test`. A task a package does not define is skipped — `pnpm run typecheck` therefore only runs the frontend's `tsc`, and `pnpm run test` only the frontend's Vitest suite (backend pytest is run directly, see `09-testing-strategy`).

## Rules

- Add behaviour as a package script; never duplicate it in the root `package.json`.
- Prefer `pnpm run <task>` locally. CI calls package-level commands directly (`.github/workflows/ci.yaml`) and does not use Turbo.

## Usage

```sh
pnpm run lint                          # turbo run lint
pnpm exec turbo run build --filter=@mlrs/frontend
pnpm exec turbo run lint --dry=json    # show what would run
pnpm exec turbo run build --force      # bypass the cache
```
