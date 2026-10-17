# skl Frontend

Next.js (App Router) UI for skl. The design system lives in
[`docs/07-ui-ux-design`](../../docs/07-ui-ux-design); the stack rules live in
[`docs/08-tech-stack/frontend`](../../docs/08-tech-stack/frontend).

## Develop

```bash
pnpm install
pnpm dev # http://localhost:3000
```

The API defaults to `http://127.0.0.1:8000`. Override it with `NEXT_PUBLIC_API_URL`.

## Scripts

| Command              | Purpose                                         |
| -------------------- | ----------------------------------------------- |
| `pnpm dev`           | Dev server                                      |
| `pnpm build`         | Production build                                |
| `pnpm start`         | Serve the production build                      |
| `pnpm lint`          | ESLint                                          |
| `pnpm format`        | Prettier (writes)                               |
| `pnpm typecheck`     | `tsc --noEmit`                                  |
| `pnpm test`          | Vitest unit tests (`src/lib`, `src/components`) |
| `pnpm test:watch`    | Vitest watch mode                               |
| `pnpm test:coverage` | Vitest + V8 coverage, 80% gate                  |
| `pnpm test:e2e`      | Playwright e2e (needs the backend on :8000)     |

## Conventions

- Arrow functions only (no `function` declarations or expressions).
- DaisyUI first, Tailwind second, custom CSS last. Wrap DaisyUI class strings in
  `src/components/ui` so pages never copy them.
- Theme is driven by `data-theme` (`skl-light` / `skl-dark`) on `<html>` and persisted in
  `localStorage` under `skl-theme`.
