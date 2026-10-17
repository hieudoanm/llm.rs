# llm — GitHub Actions

Workflow: [`.github/workflows/ci.yaml`](../../../.github/workflows/ci.yaml) on `push` and `pull_request` to `master`.

1. `backend`: `cargo clippy --all-targets`, `cargo fmt --check`, then `cargo test --workspace` against an in-memory SQLite database — 80% coverage gate via `cargo-tarpaulin`.
2. `frontend`: `pnpm install --frozen-lockfile`, `pnpm run lint`, `pnpm exec tsc --noEmit`, `pnpm run test:coverage` (80% gate), `pnpm run build`.
3. `e2e`: needs both jobs; starts the API and `pnpm run dev`, runs Playwright, uploads `playwright-report/`.

Pinned toolchain: Rust stable, Node 26, pnpm 12.16.1 — `setup-rust` and the pnpm/node caches are enabled. No cloud deploy job. A Trivy filesystem scan is welcome but not a substitute for tests.
