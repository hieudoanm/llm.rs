# mlrs — Test Organisation

Backend: `crates/mlrs-api/tests/`, grouped by module (`api/`, `db/`, `export/`,
`storage/`, `training/`) with a shared test harness and the end-to-end
`test_e2e.rs`. Training RL tests live in `tests/training/rl/`.

Frontend: `**/__tests__/` folders alongside the code under `src/lib` and `src/components`.
Each `__tests__` folder contains `*.test.ts(x)` files for the adjacent source files.

**Test files follow the same naming convention as their source:**

- `Component.tsx` → `Component.test.tsx` (PascalCase components)
- `hook.ts` → `hook.test.ts` (camelCase hooks)
- `util.ts` → `util.test.ts` (camelCase utilities)

Playwright: `packages/frontend/e2e/`. Name tests after behaviour
(`test_completed_job_has_onnx`).
