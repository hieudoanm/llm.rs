# mlrs — Vitest

**Implemented.** `packages/frontend` has a unit-test layer over `src/lib/**` and
`src/components/**`.

- Config: [`packages/frontend/vitest.config.mts`](../../../../packages/frontend/vitest.config.mts) —
  `environment: 'jsdom'`, the `@/*` → `src/*` alias, and V8 coverage.
- Setup: `packages/frontend/vitest.setup.ts` — `@testing-library/jest-dom/vitest`
  matchers plus a React Testing Library `cleanup()` after every test.
- Tests sit next to the code they cover: `src/**/*.test.ts` / `*.test.tsx`.
- `fetch` is stubbed with `vi.stubGlobal('fetch', vi.fn())`; `next/link` and
  `next/image` are mocked per file where a component needs them.

What is covered: `src/lib/api.ts` (URL building, JSON/text parsing, error
envelope → `ApiError`), `src/lib/cn.ts`, `src/lib/metrics.ts`,
`src/components/ui/*` (status badge, button, alert, file drop),
`src/components/layout/Navbar.tsx` (theme toggle and the `localStorage`
fallback) and `Providers.tsx` (query-client defaults).

| Command              | Purpose                                       |
| -------------------- | --------------------------------------------- |
| `pnpm test`          | Run the suite once                            |
| `pnpm test:watch`    | Watch mode                                    |
| `pnpm test:coverage` | Run with V8 coverage; enforces the gate below |

**Coverage gate:** statements, branches, functions and lines must each be
≥ **80%** — Vitest exits non-zero below the threshold and CI runs
`pnpm run test:coverage` in the `frontend` job. Coverage is scoped to
`src/lib/**` + `src/components/**`; the App Router pages in `src/app/**` are
exercised end to end by [Playwright](./playwright.md), so they are out of scope.
