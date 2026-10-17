# scikit-learn-client (mlrs) — Testing Strategy

## 1. Purpose

Ensure CSV validation, task-specific label rules, job lifecycle, ONNX export, and UI workflow are correct and repeatable across supervised, semi-supervised, and unsupervised tasks.

## 2. Test Layers

- [`01-test-layers.md`](./09-testing-strategy/01-test-layers.md)
- [`02-test-fixtures.md`](./09-testing-strategy/02-test-fixtures.md)
- [`03-test-organization.md`](./09-testing-strategy/03-test-organization.md)
- [`04-quality-gates.md`](./09-testing-strategy/04-quality-gates.md)
- [`05-critical-test-scenarios.md`](./09-testing-strategy/05-critical-test-scenarios.md)
- [`06-definition-of-done.md`](./09-testing-strategy/06-definition-of-done.md)

## 3. Guiding Principles

- Tests assert observable behaviour.
- SQLite-dependent tests use a real database.
- ONNX tests load the file with ONNX Runtime.
- Fixtures are synthetic; never commit private data.
- Do not mock mlrs `fit` in export tests — use tiny CSVs.
- Every task type gets a fixture, a catalogue ONNX-parity test, and an API-level job test.

## 4. Contract testing (OpenAPI)

Phase 12 adds schema drift protection:

- `crates/mlrs-api/tests/api/test_openapi_contract.rs` asserts that the Axum app's generated
  OpenAPI document equals the committed
  [`docs/06-api-specification/backend/openapi.yaml`](../06-api-specification/backend/openapi.yaml),
  that `GET /openapi.json` serves the committed document, that every route and schema is present,
  and that `cargo test --openapi --check` reports the document as current.
- CI runs `cargo test --workspace` as a hard gate so the FE/BE contract cannot drift without updating the committed schema.

## 5. Frontend E2E gotchas

- The Next.js route announcer also carries `role="alert"`. Playwright locators that match all alerts
  (e.g. `page.getByRole('alert')`) must be filtered (e.g. by text content) to avoid strict-mode
  violations when asserting error states.
- Prefer role/label locators, but avoid ambiguous `aria-label` values — Playwright strict mode
  fails on multiple matches.
- Pages that use `useSearchParams()` must be wrapped in a `<Suspense>` boundary to avoid the
  Next.js CSR bailout / build failure.
