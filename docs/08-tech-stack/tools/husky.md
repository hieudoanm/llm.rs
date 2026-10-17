# mlrs — Husky

Git hooks for the pnpm workspace.

## Choice

Husky 9 (root `devDependency`, pinned `9.1.7`).

## Why

- Hooks are committed in `.husky/`, so every contributor gets the same checks.
- Installed automatically by the root `prepare` script (`husky || true`) on `pnpm install`.

## Rejected

- `pre-commit` (the Rust framework) — a second toolchain for the same job; the hooks here are pnpm/Turbo based.
- Hand-rolled `core.hooksPath` scripts — no install, upgrade, or cross-platform story.

## Hooks

| Hook                | Runs                                                |
| ------------------- | --------------------------------------------------- |
| `.husky/pre-commit` | `pnpm turbo run lint`, then `pnpm turbo run format` |

Only `pre-commit` is committed. `.husky/_/` holds Husky's generated shims, sets `core.hooksPath`, and is itself ignored (its `.gitignore` is `*`).

## Rules

- Keep hooks fast — they run on every commit.
- A non-zero exit aborts the commit.
- `format` runs in write mode (`cargo fmt` for `crates/`, Prettier for `packages/frontend`); the hook does not re-stage files — check `git status` before committing.
- Hooks are convenience, not the contract: CI (`.github/workflows/ci.yaml`) is the authoritative gate.

## Usage

```sh
pnpm install            # installs the hooks via the prepare script
pnpm exec husky         # re-install the hooks
git commit --no-verify  # bypass the hooks when you must
```
