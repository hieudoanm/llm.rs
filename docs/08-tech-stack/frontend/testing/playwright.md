# llm — Playwright

E2E against Compose: upload `docs/fixtures` sample CSV (when added) or `crates/llm-api/tests/fixtures/iris.csv`, map columns, train logistic regression, wait for completed, download ONNX. Second spec: invalid CSV shows alert. `baseURL` `http://127.0.0.1:3000`.
