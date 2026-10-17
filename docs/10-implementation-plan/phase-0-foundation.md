# Phase 0: Foundation

_Part of [10-implementation-plan.md](../10-implementation-plan.md)._ Week 1.

## Deliverables

Monorepo `packages/frontend` (Next.js + Tailwind + DaisyUI), `crates/llm-api` (Axum hello), `docker-compose.yaml` with sqlite + backend + frontend skeletons, GitHub Actions lint/test skeleton, README run instructions, Husky.

## Acceptance

- `docker compose up` starts Postgres healthy.
- Backend `/healthz` 200 (readyz may 503 until Phase 1).
- Frontend renders Home.
- CI green on skeletons.

## Dependencies

None.
