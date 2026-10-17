# llm-db

SQLite persistence layer for [llm.rs](../../README.md), built with [sqlx](https://github.com/launchbadge/sqlx).

## Features

- **Schema management**: Migrations and schema definitions
- **Datasets**: Store and retrieve uploaded CSV datasets
- **Jobs**: Track training job status, parameters, metrics, and results
- **Artefacts**: Manage trained model artefacts and paths
- **Idempotency**: Keys for safe retries
- **Conversions**: Helpers between domain types and DB models

## Dependencies

- `llm-core` for domain types
- `sqlx` with SQLite, Tokio runtime, and macros
- `tokio`, `uuid`, `chrono`, `serde`, etc.

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
llm-db = { path = "../llm-db" }
```

## Testing

```sh
cargo test -p llm-db
```
