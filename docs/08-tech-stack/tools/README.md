# mlrs — Tools

Package managers, task runner, git hooks, linters, and formatters used across the repo.

## Documents

| Document                             | Covers                   |
| ------------------------------------ | ------------------------ |
| [`pnpm.md`](./pnpm.md)               | Frontend package manager |
| [`uv.md`](./uv.md)                   | Rust package manager   |
| [`turbo.md`](./turbo.md)             | Workspace task runner    |
| [`husky.md`](./husky.md)             | Git hooks                |
| [`lint-staged.md`](./lint-staged.md) | Staged-file checks       |
| [`eslint.md`](./eslint.md)           | Frontend lint            |
| [`prettier.md`](./prettier.md)       | Frontend formatter       |
| [`ruff.md`](./ruff.md)               | Rust lint + format     |

## Shared rules

1. One lockfile per ecosystem: `pnpm-lock.yaml` (workspace), `uv.lock` (backend).
2. Ruff is the only Rust formatter; do not add a second one.
3. Prettier is the only frontend formatter; ESLint never formats.
4. Turborepo is the only workspace task runner (`turbo.json`); behaviour belongs in a package script, not the root `package.json`.
5. CI (`.github/workflows/ci.yaml`) is the gate: Ruff check + format check, pytest, ESLint, `tsc`, `next build`, Playwright. Local hooks are convenience only.
