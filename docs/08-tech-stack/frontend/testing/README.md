# llm — Frontend testing

| Document                           | Covers         |
| ---------------------------------- | -------------- |
| [`vitest.md`](./vitest.md)         | Unit (jsdom)   |
| [`playwright.md`](./playwright.md) | E2E CSV → ONNX |

Vitest unit-tests `src/lib` + `src/components` with an **80%** coverage gate
(`pnpm run test:coverage`, CI job `frontend`); Playwright covers the CSV → ONNX
flows (`packages/frontend/e2e/`, CI job `e2e`).
