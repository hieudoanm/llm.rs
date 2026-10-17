# mlrs — lint-staged

Staged-file linter runner for the pnpm workspace.

## Choice

lint-staged 17 (root `devDependency`, pinned `17.6.0`).

## Why

- Runs linters and formatters only on the files in the commit, so hooks stay fast as the repo grows.
- Standard companion to Husky; configured in the root `package.json` or a `.lintstagedrc*` file.

## Rejected

- Linting the whole repository on every commit — slow, and unrelated failures block unrelated commits.

## Status

**Declared, not wired yet.** No lint-staged configuration exists (the root `package.json` has no `lint-staged` key) and `.husky/pre-commit` currently runs the workspace-wide `pnpm turbo run lint` / `pnpm turbo run format` instead. `pnpm exec lint-staged` therefore matches nothing.

Wiring it means adding a config to the root `package.json`, for example:

```json
"lint-staged": {
  "packages/frontend/**/*.{ts,tsx,js,mjs,json,md}": ["eslint --fix", "prettier --write"],
  "packages/{backend,data}/**/*.py": ["ruff check --fix", "ruff format"]
}
```

and replacing the hook body with `pnpm exec lint-staged`.

## Usage

```sh
pnpm exec lint-staged           # from the repository root, with files staged
pnpm exec lint-staged --debug
```
