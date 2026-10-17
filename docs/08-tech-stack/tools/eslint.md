# llm — ESLint

Lint for `packages/frontend`.

## Choice

ESLint with the Next.js / TypeScript flat config.

## Why

- Industry standard for JS/TS; first-class Next.js support via `eslint-config-next`.
- Catches unused vars, implicit `any` leaks, and Next.js-specific mistakes (e.g. `next/link` misuse).

## Rejected

- Biome as the sole linter — ESLint + Next.js config is the better fit for this stack.

## Rules

- Extends `next/core-web-vitals` plus `eslint-config-next/typescript` (flat config in `eslint.config.mjs`).
- From those presets: `@typescript-eslint/no-explicit-any` is an **error** and unused variables are warnings (the core `no-unused-vars` rule is off). No `eqeqeq` rule is enabled.
- ESLint lints only; it does **not** format (Prettier owns formatting).

## Usage

```sh
pnpm exec eslint .          # same as: pnpm run lint
pnpm exec eslint . --fix
```
