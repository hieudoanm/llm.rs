# llm — Prettier

Formatter for `packages/frontend`.

## Choice

Prettier.

## Why

- Single, opinionated formatter; no bikeshedding over style.
- ESLint is kept formatting-free so Prettier remains the sole source of truth for layout.

## Rejected

- dprint — less ecosystem traction than Prettier in the Next.js community.

## Config

[`packages/frontend/.prettierrc.json`](../../../packages/frontend/.prettierrc.json) (next to `.prettierignore`): print width 100, single quotes (including JSX), semicolons, `trailingComma: "all"`, 2-space indent, LF, and `prettier-plugin-tailwindcss` for class sorting.

Notes:

- Prettier is not a declared dependency yet — it arrives as the auto-installed peer of `prettier-plugin-tailwindcss`. Add it to `devDependencies` (and pin it) before relying on it.
- `pnpm run format` (Turbo) runs `prettier --cache --write .` for the frontend, plus `cargo fmt` for `crates/`; `pnpm run format:check` verifies without writing. No Prettier check runs in CI.

## Usage

```sh
pnpm run format          # packages/frontend: prettier --cache --write .
pnpm run format:check    # verify only
pnpm exec prettier --write .
pnpm exec prettier --check .
```

ESLint's `format` rules are disabled to avoid conflicts; run Prettier separately.
