# CI/CD Pipeline

GitHub Actions only ([`.github/workflows/ci.yaml`](../../.github/workflows/ci.yaml)). No deploy job.

Triggers: `push` and `pull_request` on `master`. Toolchain: Rust stable, Node 26, pnpm 12.16.1.

| Job        | What it runs                                                                                                                                                                 |
| ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `backend`  | `cargo check --workspace --all-targets`, `cargo test --workspace` (Rust unit tests)                                                                                       |
| `frontend` | `pnpm install --frozen-lockfile`, `pnpm run lint`, `pnpm exec tsc --noEmit`, `pnpm run test:coverage` (80% gate), `pnpm run build`                                          |
| `e2e`      | needs `backend` + `frontend`; builds and starts the Rust backend on :8000, starts `pnpm run dev` on :3000, installs Playwright browsers, runs `pnpm exec playwright test`, uploads `playwright-report/` |

Both test jobs fail below **80%** coverage — Vitest for the frontend, `cargo tarpaulin` for the backend.

The `backend` job also fails when `cargo clippy` reports warnings.

OIDC/cloud credentials are not used.
