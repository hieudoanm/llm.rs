# mlrs — Axum (Rust)

- Routers under `/api/v1` in `crates/mlrs-api/src/routes/`.
- Exception handlers emit the error envelope; `requestId` middleware.
- CSV upload spooled to disk; max size via `DefaultBodyLimit` matching NFR-030.
- CORS: frontend origin only.
- `GET /healthz` no DB; `GET /readyz` SQLite `SELECT 1`.
- Background worker loop in `crates/mlrs-api/src/worker.rs`.
