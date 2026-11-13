# Desktop app

`packages/desktop` is a Next.js + Tauri application — the same training studio
as a native desktop app.

## Develop

```sh
cd packages/desktop
pnpm install
pnpm dev            # Next.js dev server + Tauri concurrently
```

## Build

```sh
pnpm build && pnpm build:tauri
```

Produces bundles under `src-tauri/target/release/bundle` for the host platform.

## Tests

The desktop frontend has a Vitest suite plus Playwright e2e specs. Prefer
role/label locators; avoid ambiguous `aria-label` values — Playwright strict
mode fails on duplicates.
