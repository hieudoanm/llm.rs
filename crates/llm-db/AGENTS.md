# AGENTS.md — crates/llm-db

Guidance for AI agents working on `llm-db`, the SQLite persistence layer for llm.rs.

## Purpose

`llm-db` wraps a `sqlx::SqlitePool` and provides repository-style access to datasets, jobs, artefacts, and idempotency keys. It has no HTTP concerns so the same layer can be embedded in the Axum server or a Tauri application.

## Layout

```
src/
  lib.rs          — Database struct (connect, in_memory, migrate, ping, pool)
  schema.rs       — idempotent SQL schema statements (SCHEMA constant)
  convert.rs      — sqlx error → LlmError conversion
  datasets.rs     — dataset repository (insert_dataset, get_dataset, ...)
  jobs.rs         — job repository (insert_job, get_job, list_jobs, set_status, record_metrics, ...)
  artefacts.rs    — artefact repository (insert_artefact, list_artefacts, ...)
  idempotency.rs  — idempotency key repository
```

## Key public API

- `llm_db::Database::connect(settings) -> LlmResult<Database>` — open/create SQLite DB at `Settings.sqlite_path()` or `DATABASE_URL`, run migrations
- `llm_db::Database::in_memory() -> LlmResult<Database>` — ephemeral in-memory DB (tests)
- `llm_db::Database::pool() -> &SqlitePool` — access the raw pool
- `llm_db::Database::migrate() -> LlmResult<()>` — apply idempotent schema
- `llm_db::jobs::{JobFilters, JobCursor, WorkerCounts}` — job filter/list types
- Repository modules: `datasets`, `jobs`, `artefacts`, `idempotency`

## Conventions

- **No HTTP** — repository functions take `&SqlitePool` or `&Database`, never `Request`/`Response`
- **Errors** use `llm_core::error::{LlmError, LlmResult}` — sqlx errors are converted in `convert.rs`
- **All schema statements are idempotent** — `CREATE TABLE IF NOT EXISTS`, etc.
- **WAL journal mode** + `synchronous = Normal` for the connection pool (8 max connections)
- **Foreign keys enabled** on every connection

## Dependencies

- `llm-core` — for `Settings`, `LlmError`, `LlmResult`
- `sqlx` — SQLite async driver (runtime-tokio, sqlite, macros features)
- `serde`/`serde_json` — serialisation
- `sha2`, `uuid` — hashing and IDs
- `base64`, `chrono`, `tracing`, `thiserror`, `tokio`

## Commands (run from `crates/`)

```sh
cargo check -p llm-db
cargo test -p llm-db
cargo build -p llm-db
cargo clippy -p llm-db -- -D warnings
```

## Notes

- Database path defaults to `.llm-data/llm.db` relative to the working directory (see `llm-core::config::Settings`)
- Migrations run automatically on `Database::connect` — no separate migration step needed
- In-memory database is used by tests and ephemeral Tauri embeddings
