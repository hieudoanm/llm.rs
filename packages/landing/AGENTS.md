# AGENTS.md — llm.rs landing page

Guidance for AI agents working on the landing/documentation site (`packages/landing`). See the root [`AGENTS.md`](../../AGENTS.md) for repository-wide conventions.

## Overview

The landing page is a statically-exported Next.js 16 site (App Router) deployed to GitHub Pages at `https://hieudoanm.github.io/llm.rs`. It runs at `basePath: '/llm.rs'`.

## Layout

- `src/app/` — App Router pages:
  - `page.tsx` — home (landing)
  - `docs/page.tsx` — docs index
  - `docs/[slug]/page.tsx` — individual doc page
  - `downloads/page.tsx` — download page
  - `not-found.tsx`, `loading.tsx`, `error.tsx`, `global-error.tsx`
  - `layout.tsx` — root layout with metadata + basePath-prefixed assets
- `src/components/` — React components:
  - `shared/` — `Footer`, `Navbar` (used across pages)
  - `landing/` — `Hero`, `Bands`, `WhyLLM`, `Workflow`, `BuiltDifferent`, `Tools`, `GetStarted`, `Markdown` (home page sections)
  - `docs/` — `DocsSearch`
- `src/lib/` — `site.ts` (site metadata + `basePath`), `docs.ts` (doc registry)
- `public/` — static assets (`images/`, `manifest.json`, `sw.js`)
- `src/lib/__tests__/` — lib unit tests
- `src/components/__tests__/` — component tests (co-located: `shared/__tests__/`, `landing/__tests__/`, `docs/__tests__/`)

## Commands

```sh
cd packages/landing

# Dev server (port 4173)
pnpm dev

# Production static build (output: out/ → out/llm.rs for basePath)
pnpm build

# Typecheck
pnpm exec tsc --noEmit   # or pnpm check / pnpm typecheck

# Lint
pnpm lint

# Format
pnpm format

# Unit tests (Vitest)
pnpm test

# Serve the built output locally
pnpm serve
```

## Conventions

- **`basePath: '/llm.rs'`** — all internal navigation uses Next `<Link>` with root-relative paths (`href='/'`, `href='/docs'`, `href='/downloads'`). Images and assets must be prefixed with `basePath` from `src/lib/site.ts` (e.g. `src={`${basePath}/images/svg/llm-logo.svg`}`).
- **Internal links:** use Next.js `<Link />` from `next/link`, not `<a>`, for same-site navigation. External links (GitHub, etc.) stay as `<a>`.
- **Tests:** Vitest + React Testing Library, co-located `__tests__/` folders next to each component.
- **Styling:** Tailwind CSS 4 with custom tokens; see `src/app/globals.css`.
- **Deploy:** CI builds and deploys to GitHub Pages via `actions/upload-pages-artifact` + `actions/deploy-pages` (see `.github/workflows/ci.yaml`).

## Tests

89 tests across 23 files run via `pnpm test`. Tests are co-located with their components in `__tests__/` directories:

- `src/components/shared/__tests__/` — Footer, Navbar
- `src/components/landing/__tests__/` — Hero, Bands, WhyLLM, Workflow, BuiltDifferent, Tools, GetStarted, Markdown
- `src/components/docs/__tests__/` — DocsSearch
- `src/lib/__tests__/` — site, docs
