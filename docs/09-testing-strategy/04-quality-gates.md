# mlrs — Quality Gates

Pre-commit (Husky): `pnpm turbo run lint` then `pnpm turbo run format` —
whole-workspace, writing in place; `lint-staged` (staged-file checks) is declared
but not wired.

CI is the merge gate:

- `backend` — cargo clippy + cargo fmt check (`src/` only) and `cargo test --workspace -v`
  against an in-memory SQLite database.
- `frontend` — ESLint, `tsc --noEmit`, `pnpm run test:coverage`, `next build`.
- `e2e` — Playwright against the running stack (needs `backend` + `frontend`).

The **80%** frontend coverage threshold (statements, branches, functions, lines)
is enforced whenever coverage runs: `pnpm run test:coverage` fails the job below
it, and CI runs that command in `frontend`. The backend carries the same 80% gate
through `cargo-tarpaulin` (`--fail-under=80` in
`crates/mlrs-api/Cargo.toml`), applied by the `cargo test --workspace -v`
step in CI.
