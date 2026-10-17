# llm — pnpm

Package manager for the workspace (`packages/*`).

## Choice

pnpm (pinned in `package.json` / CI).

## Why

- Fast, disk-efficient installs via content-addressable store.
- Strict `node_modules` (no phantom dependencies) — matches the "boring, mature" rule.
- Already used by sibling projects in the org; consistent CI tooling.

## Rejected

- npm — slower installs, less strict dependency tree.
- Yarn Classic — deprecated lockfile format; Plug'n'Play adds friction.

## Lockfile

`pnpm-lock.yaml` is committed and regenerated only when dependencies change. Never edit by hand.

## Usage

```sh
pnpm install
pnpm add <pkg>
pnpm run dev
```
