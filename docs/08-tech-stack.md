# scikit-learn-client (mlrs) — Technology Stack & Decisions

## 1. Purpose

This document records **which technologies are chosen and why**. Companion how-to docs live under [`08-tech-stack/frontend/`](./08-tech-stack/frontend/), [`08-tech-stack/backend/`](./08-tech-stack/backend/), and [`08-tech-stack/infrastructure/`](./08-tech-stack/infrastructure/).

Where this file disagrees with numbered documents `01`–`12`, the numbered documents govern.

## 2. How Decisions Are Made

1. Prefer the boring, mature option.
2. One tool per responsibility.
3. Types and validation at the boundaries (serde + TypeScript).
4. Reproducibility: pinned deps, Alembic, Compose.
5. Justify every dependency.
6. Record the rejected alternative.

Non-negotiable:

- **Fit never runs in the HTTP handler.**
- **Every successful job writes ONNX.**
- **Deployment is Docker Compose on localhost.**

## 3. Stack at a Glance

| Layer                     | Chosen technology                         | Decision record                                  |
| ------------------------- | ----------------------------------------- | ------------------------------------------------ |
| Frontend framework        | Next.js (App Router) + TypeScript         | [§4.1](#41-nextjs--typescript)                   |
| Styling                   | Tailwind CSS + DaisyUI                    | [§4.2](#42-tailwind-css--daisyui)                |
| Frontend data             | TanStack Query                            | [§4.3](#43-tanstack-query)                       |
| Frontend charts           | visx (SVG)                                | [§4.5](#45-visx)                                 |
| Jobs-list virtualisation  | TanStack Virtual                          | [§4.7](#47-tanstack-virtual)                     |
| Forms + schema validation | Zod + React Hook Form + resolvers         | [§4.8](#48-zod--react-hook-form)                 |
| Typed API client          | openapi-typescript + openapi-fetch        | [§4.9](#49-typed-api-client-from-the-contract)   |
| Frontend testing          | Vitest (jsdom) + Playwright               | [§4.4](#44-vitest--playwright)                   |
| Package manager           | pnpm                                      | [tools/pnpm.md](./08-tech-stack/tools/pnpm.md)   |
| Workspace tasks           | Turborepo                                 | [tools/turbo.md](./08-tech-stack/tools/turbo.md) |
| Git hooks                 | Husky (+ lint-staged, declared)           | [tools/husky.md](./08-tech-stack/tools/husky.md) |
| API                       | Axum + serde v2 + axum           | [§5.1](#51-fastapi)                              |
| Language                  | Rust stable                               | [§5.2](#52-rust)                               |
| Classical ML              | scikit-learn                              | [§5.3](#53-scikit-learn)                         |
| Neural nets (P1)          | scikit-learn MLPs (configurable)          | [§5.4](#54-neural-networks-tabular-mlps)         |
| Export                    | ONNX + mlrs2onnx; optional pickle / joblib | [§5.5](#55-onnx)                                 |
| Validation/serialisation  | serde v2                               | [§5.6](#56-pydantic-v2)                          |
| ORM / migrations          | SQLx                      | [§5.7](#57-sqlx)                  |
| Database                  | SQLite 18                             | [§6](#6-sqliteql)                              |
| Containers                | Docker + Compose                          | [§7](#7-docker-compose)                          |
| CI                        | GitHub Actions                            | [§8](#8-github-actions)                          |
| Rust lint               | Ruff                                      | [tools/ruff.md](./08-tech-stack/tools/ruff.md)   |

## 4. Frontend

### 4.1 Next.js + TypeScript

**Choice:** Next.js App Router, strict TypeScript.

**Why:** User asked for Next.js; App Router is the current default; SSR is useful for the home page; client components for upload and polling.

**Rejected:** Vite SPA (used in NEO Bank) — inconsistent with the Next.js requirement. Remix — extra learning without benefit.

### 4.2 Tailwind CSS + DaisyUI

**Choice:** Tailwind v4 (or v3 if DaisyUI compatibility requires it — pin what DaisyUI supports) + DaisyUI.

**Why:** Matches the requested stack and NEO Bank token approach. Semantic `btn` / `alert` classes.

**Rejected:** MUI, Chakra, CSS modules as the primary system.

### 4.3 TanStack Query

**Choice:** TanStack Query for server state (jobs poll, algorithms, history).

**Why:** `refetchInterval` with `enabled` until terminal status. Cache for catalogue.

**Rejected:** Redux, raw `useEffect` fetch.

### 4.4 Vitest + Playwright

**Choice:** Vitest for components; Playwright for CSV → ONNX E2E against Compose.

**Rejected:** Jest (overlap), Cypress (Playwright in the org standard from NEO Bank).

**Status:** both layers ship. Vitest 5 runs in `packages/frontend` over
`src/lib` + `src/components` (`vitest.config.mts`, jsdom + React Testing Library)
with an 80% coverage gate enforced by `pnpm run test:coverage` in the CI
`frontend` job; Playwright lives in `packages/frontend/e2e/` and runs in the CI
`e2e` job. See [`frontend/testing/vitest.md`](./08-tech-stack/frontend/testing/vitest.md).

### 4.5 visx

**Choice:** visx 4.x (`@visx/scale`, `@visx/axis`, `@visx/shape`, `@visx/group`, `@visx/tooltip`,
`@visx/responsive`) for the completed-job result charts.

**Why:** the chart set is fixed and small and the data arrives pre-aggregated, so the need is
D3-grade scales, axes, and shapes rather than a design system. visx provides those as small,
tree-shakeable SVG React primitives, is TypeScript-first, and is React 19-ready; SVG composes with
the DaisyUI tokens and the required per-chart text fallback.

**Rejected:** Recharts 3.x — transitive Redux Toolkit, react-redux, immer, and `victory-vendor` for
a fixed, pre-aggregated set, and no heatmap; react-chartjs-2 / echarts-for-react — Canvas renders no
selectable text and ignores CSS token theming; hand-rolled SVG — reimplements scales, axes, and
tooltips.

Details: [`07-ui-ux-design/system/charts.md`](./07-ui-ux-design/system/charts.md); how-to:
[`frontend/core/visx.md`](./08-tech-stack/frontend/core/visx.md).

### 4.6 Custom SVG → PNG rasterisation (no dependency)

**Choice:** ~30-line utility using native browser APIs (`XMLSerializer`, `Image`, `Canvas`).

```ts
// packages/frontend/src/lib/svgToPng.ts
export async function svgToPngBlob(
  svgElement: SVGSVGElement,
  pixelRatio = 2
): Promise<Blob> {
  const svgMarkup = new XMLSerializer().serializeToString(svgElement);
  const svgDataUrl = `data:image/svg+xml;base64,${btoa(svgMarkup)}`;
  const img = await loadImage(svgDataUrl);
  const canvas = document.createElement('canvas');
  const ctx = canvas.getContext('2d')!;
  canvas.width = img.width * pixelRatio;
  canvas.height = img.height * pixelRatio;
  ctx.scale(pixelRatio, pixelRatio);
  ctx.drawImage(img, 0, 0);
  return new Promise((resolve, reject) =>
    canvas.toBlob((blob) => (blob ? resolve(blob) : reject(new Error('PNG conversion failed'))), 'image/png')
  );
}

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = reject;
    img.src = src;
  });
}
```

**Why:** zero dependencies, ~500 bytes gzipped, full control over pixel ratio and background. Only used
for the "Download PNG" feature (FEATURE-12); not in the critical path.

**Handles:** DaisyUI theme colours via CSS variables (SVG uses `currentColor` / CSS vars), transparent
background, 2× DPI for crisp output.

**Rejected:** `html2canvas` (~40 KB, canvas-first), `html-to-image` (dependency for ~30 lines), `dom-to-image` (unmaintained).

### 4.7 TanStack Virtual

**Choice:** `@tanstack/react-virtual` 3.14.21 for the jobs-history table.

**Why:** The history table renders every returned job; a studio that trains a model headlessly (or
that runs a long sweep) accumulates hundreds of rows, and mounting them all costs layout work on
every poll. The virtualiser mounts only the rows in view. It comes from the TanStack family already
used for server state (§4.3), is ~5 KB, and preserves native table semantics — the `<tbody>` keeps
its `role` and column structure, so screen readers still announce header/row relationships.

**Rejected:** `react-window` / `react-virtualized` — a second virtualisation paradigm and, for
`react-window` 1.x, no React 19 support; pagination instead of virtualisation — changes the API
contract and breaks "scroll to find the job"; rendering everything — the current behaviour, and
exactly the cost this item removes.

Used by `components/jobs/VirtualTableBody.tsx`.

### 4.8 Zod + React Hook Form

**Choice:** `zod` 4.6.7 + `react-hook-form` 7.90.0 + `@hookform/resolvers` 6.0.0 for the
hyperparameter form.

**Why:** The algorithm catalogue already returns each knob's bounds (`min`, `max`, `step`). Zod lets
the form turn that metadata into a schema once, so the input's range, the inline message, and the
wizard's summary panel are all derived from the same rule instead of three hand-written checks. React
Hook Form keeps the fields uncontrolled (no re-render per keystroke, which matters in the wizard),
and `@hookform/resolvers` is the documented bridge between the two. Zod doubles as the standard for
boundary validation the stack already calls for (§2, rule 3).

**Rejected:** Yup + Formik — Formik re-renders on every keystroke and the pair is a larger surface
for the same job; hand-rolled validation — the three-way duplication this item removes; `zodResolver`
without the schema module — the rules would still live in the component.

Used by `app/jobs/new/_lib/hyperparameters.ts` and
`components/jobs/new/HyperparameterForm.tsx`.

### 4.9 Typed API client from the contract

**Choice:** `openapi-typescript` 7.13.0 (devDependency) generates `src/lib/apiTypes.ts` from the
committed [`06-api-specification/backend/openapi.yaml`](./06-api-specification/backend/openapi.yaml);
`openapi-fetch` 0.17.0 supplies the runtime path types.

**Why:** The contract is already committed and drift-checked in CI (NFR-073, FR-191). Generating the
TypeScript types from it means a backend schema change that is not mirrored in the frontend fails
`tsc`, not a user's browser. The generated types are plain types — no runtime cost — and the
resilient wrapper in `src/lib/api.ts` (timeout, retries, `ApiError`) is kept as the transport, so the
error taxonomy is unchanged.

**Rejected:** Hand-maintained interfaces — the exact drift the contract test exists to prevent (this
work found `retrying` missing from the status union and `artefacts` marked required); `openapi-fetch`
as the *transport* — it would re-implement the retry/timeout/`ApiError` behaviour `api.ts` already
has; GraphQL/codegen — no GraphQL in the stack.

Regenerate with `pnpm --filter @mlrs/frontend run openapi:types`. Used by `src/lib/apiTyped.ts`.

## 5. Backend

### 5.1 Axum

**Choice:** Axum.

**Why:** Requested; OpenAPI generated from serde; async-friendly; multipart uploads.

**Rejected:** Flask, Django Ninja (heavier), Go (requested Rust/ML).

**Interactive reference:** Axum ships Swagger UI (`/docs`), ReDoc (`/redoc`), and the raw
schema (`/openapi.json`) with no extra dependency. The committed schema is regenerated from the
running application and checked for drift in CI (NFR-073, FR-191). The committed copy is
[`06-api-specification/backend/openapi.yaml`](./06-api-specification/backend/openapi.yaml).

### 5.1.1 Configuration

**Choice:** A single settings module (`crates/mlrs-api/src/config.py`) that reads the
environment once at start-up and exposes typed values with documented defaults. No
`pydantic-settings` dependency.

**Why:** One place to read, one place to test, one place to document. The fallback behaviour
(default value plus one startup warning) is simpler than adding a dependency, and the variable
list stays short enough to validate by hand.

**Rejected:** `pydantic-settings` (an extra dependency for a handful of variables),
`os.environ` scattered across modules (untestable, unlistable).

### 5.1.2 Archives

**Choice:** Rust's stdlib `zipfile` for the artefact bundle.

**Why:** No new dependency, streaming-friendly, deterministic member order.

**Rejected:** A third-party archiver (unnecessary for a handful of files).

### 5.2 Rust

**Choice:** Rust 2021 edition in `debian:bookworm-slim`.

**Why:** current stable; linfa/onnx-protobuf crates ship for Rust stable.

### 5.3 linfa/ndarray

**Choice:** linfa as the P0 estimator library.

**Why:** Tabular baselines across supervised, semi-supervised, clustering, and anomaly-detection tasks; preprocessing + `onnx-protobuf` export.

**Rejected:** XGBoost as P0 (ONNX path extra); DBSCAN/Agglomerative/LabelSpreading (no verified ONNX path).

**Catalogue:** the 19 shipped supervised estimators plus the 5 planned semi-supervised/unsupervised estimators are documented in [`backend/models.md`](./08-tech-stack/backend/models.md). Phase 11 adds `fitted_q_iteration` for reinforcement learning.

### 5.4 Neural networks (tabular MLPs)

**Choice:** scikit-learn `MLPClassifier` / `MLPRegressor`, exposed as `mlp_classifier` and `mlp_regressor` with a configurable architecture (`hidden_layer_sizes`, `activation`, `solver`, `alpha`, `learning_rate_init`, `max_iter`, `batch_size`, `early_stopping`).

**Why:** This is a tabular training studio; multi-layer perceptrons are the appropriate neural-net model for tabular data. Riding on the existing scikit-learn estimator keeps a single `Pipeline` + `mlrs2onnx` export path, needs no new dependency, and adds no image-size cost. The architecture is still genuinely configurable — depth, width, activation, solver, regularisation, and early stopping are all hyperparameters (FR-023, FR-026, FR-027).

**Rejected:** GPU / large-net training in Compose — non-goal.

### 5.5 ONNX

**Choice:** `onnx`, `mlrs2onnx` (both required exports), and `onnxruntime` in tests. Every catalogue entry — including the MLPs and the FQI Q-function — converts through a single `mlrs2onnx` path.

**Why:** Portable contract. Pickle is Rust-only and unsafe as an interchange format.

**Rejected:** PMML, joblib-only export.

### 5.6 serde v2

**Choice:** serde v2 (BaseModel, ConfigDict, Field, model_validator, TypeAdapter).

**Why:**
- **Request/response validation** at the API boundary — every endpoint uses typed models (`CreateJobRequest`, `JobResponse`, `DatasetResponse`, etc.)
- **OpenAPI generation** — Axum derives the contract from serde models; the normative spec lives in [`06-api-specification/backend/openapi.yaml`](./06-api-specification/backend/openapi.yaml)
- **Serialization** — Jobs, datasets, algorithms, and errors are shaped by serde before JSON encoding
- **Type-safe internal contracts** — Worker, training pipelines, and export modules receive validated dicts from serde models
- **Performance** — v2's Rust core is significantly faster than v1

**Usage rules:**
- All API endpoint signatures use serde models (`request: CreateJobRequest` → `response_model=JobAcceptedResponse`)
- Models live in the `src/api/schemas/` package, grouped by domain (`shared`, `health`, `datasets`, `algorithms`, `jobs`) and re-exported from `src/api/schemas/__init__.py`; callers keep importing from `src.api.schemas`
- Validation logic stays in the model (`@model_validator(mode="after")` for cross-field rules like label/rlColumns per task type)
- Enum values mirror the OpenAPI spec (`TaskType`, `JobStatus`, `ArtefactFormat`)
- The `validate_job_request()` helper accepts both `CreateJobRequest` and `dict` for backward compatibility with tests

**Rejected:** Raw dict validation, manual JSON schema, marshmallow, attrs.

### 5.7 SQLAlchemy + Alembic

**Choice:** SQLAlchemy 2.x + Alembic.

**Why:** Typed mappings, versioned DDL matching [`schema.sql`](./05-data-model/sqliteql/schema.sql).

**Rejected:** Raw psycopg only (migrations still needed); Prisma (wrong language).

## 6. SQLite

**Choice:** Official `sqlite:18` image, named volume.

**Why:** Requested; JSONB for metrics; `SKIP LOCKED` for the worker.

**Rejected:** SQLite (weaker concurrency), Cassandra (wrong for this product).

## 7. Docker Compose

**Choice:** Three services: `frontend`, `backend`, `sqlite`. Bind `127.0.0.1`.

**Why:** User-specified local-only deployment. One Dockerfile per app service.

**Rejected:** Kubernetes, cloud run, Tilt as the required path.

## 8. GitHub Actions

Three jobs on push and pull request to `master` ([`.github/workflows/ci.yaml`](../.github/workflows/ci.yaml)): backend (Ruff lint + format check, pytest against a `sqlite:18` service container), frontend (ESLint, `tsc`, `next build`), and e2e (Playwright, needs both). Turborepo is not used in CI.

## 9. Explicitly out of stack

Kafka, Redis, MinIO, MLflow, S3, EKS/GKE, Terraform.

## 13. Package manager, tasks & git hooks

pnpm for the workspace; `uv` for the backend lockfile; Turborepo for workspace tasks; Husky for the `pre-commit` hook. `lint-staged` is declared at the root but not wired up yet.

Details:

- [`tools/pnpm.md`](./08-tech-stack/tools/pnpm.md)
- [`tools/uv.md`](./08-tech-stack/tools/uv.md)
- [`tools/turbo.md`](./08-tech-stack/tools/turbo.md)
- [`tools/husky.md`](./08-tech-stack/tools/husky.md)
- [`tools/lint-staged.md`](./08-tech-stack/tools/lint-staged.md)
- [`tools/eslint.md`](./08-tech-stack/tools/eslint.md)
- [`tools/prettier.md`](./08-tech-stack/tools/prettier.md)
- [`tools/ruff.md`](./08-tech-stack/tools/ruff.md)

See companions:

- [`frontend/README.md`](./08-tech-stack/frontend/README.md)
- [`backend/README.md`](./08-tech-stack/backend/README.md)
- [`infrastructure/README.md`](./08-tech-stack/infrastructure/README.md)
