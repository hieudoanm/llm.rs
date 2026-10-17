# mlrs — SQLite access

SQLite via `sqlx`. Schema in `crates/mlrs-db/src/schema.rs`. Connection string from env `DATABASE_URL`. Pool size small (5). Worker claim: `FOR UPDATE SKIP LOCKED`. JSON loads into `serde_json::Value`. Tests use an in-memory or file SQLite database.
