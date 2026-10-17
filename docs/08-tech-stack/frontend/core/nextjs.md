# mlrs — Next.js

App Router. Package path: `packages/frontend`.

Routes as in [`layout.md`](../../../07-ui-ux-design/system/layout.md).

Dockerfile: multi-stage `node:22-alpine` build, `next start` (or standalone output) in runtime. Do not export a fully static site: job pages are dynamic.

Env:

- `NEXT_PUBLIC_API_URL` (browser)
- Do not put Postgres URLs in the frontend.

API client: `fetch` wrapper returning the error envelope; throw a typed `ApiError`.
