# llm — Backend Best Practices

## Documents

| Document                               | Covers                           |
| -------------------------------------- | -------------------------------- |
| [`rust.md`](./rust.md)             | Layout, typing, clippy           |
| [`fastapi.md`](./fastapi.md)           | Routers, errors, uploads         |
| [`llm.md`](./llm.md) | Pipelines, metrics               |
| [`models.md`](./models.md)             | The estimator model catalogue    |
| [`neural-networks.md`](./neural-networks.md) | Tabular MLPs (P1)         |
| [`onnx.md`](./onnx.md)                 | Required export                  |
| [`sqliteql.md`](./sqliteql.md)     | SQLite access, worker claim      |

## Shared rules

1. Fit/export off the event loop (tokio `spawn_blocking`).
2. Allow-listed `algorithmId` only.
3. Never deserialize untrusted binary bytes.
4. Parameterised SQL.
5. Completed jobs have ONNX or they are not completed.
6. Structured logs carry `requestId` and `jobId` where relevant.
7. Relative storage keys.
8. Serde models match OpenAPI schemas.
