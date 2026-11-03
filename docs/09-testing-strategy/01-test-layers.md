# llm — Test Layers

## Unit

**Frontend — implemented.** Vitest 5 + jsdom + React Testing Library over
`src/lib/**` and `src/components/**` (config
`packages/frontend/vitest.config.mts`, specs next to the code as
`src/**/*.test.ts(x)`). Hard **80%** threshold on statements, branches,
functions and lines, enforced by `pnpm run test:coverage` in the CI `frontend`
job. Exercises the API error envelope, metric copy, status-badge mapping,
button/alert variants, file-drop validation and the theme toggle; `fetch` is
stubbed with `vi.stubGlobal`. App Router pages (`src/app/**`) are out of unit
scope — Playwright covers them.

**Backend — module + API boundary.** cargo test in `crates/llm-api/tests/` is
grouped by module (`api/`, `db/`, `export/`, `storage/`, `training/`) plus
`test_e2e.rs`, covering the algorithm catalogue, training pipeline, storage and
DB helpers, ONNX export and the HTTP surface. `[lints]` in
`crates/llm-api/Cargo.toml` carries the same **80%** gate: `cargo-tarpaulin`
with `--fail-under=80` over `src/`, omitting `src/training/worker.rs`.

## Integration

pytest + real SQLite + temp `DATA_DIR`. Upload, create job, wait for worker, assert artefact row.

## Contract

OpenAPI schema validation of responses (schemathesis or pytest + jsonschema).

## E2E

Playwright: CSV → completed → download ONNX.

## Security

Oversize upload, path-like filenames, missing artefact ids.

## Performance

Job accept < 300 ms on tiny CSV (NFR-002) in CI as a soft assertion where hardware allows.
