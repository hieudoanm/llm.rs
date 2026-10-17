//! SQLite persistence for the pure-Rust mlrs backend.
//!
//! [`Database`] wraps a `sqlx` pool. Repositories are free of HTTP concerns so
//! the same layer can be embedded in the Axum server or a Tauri application.

use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;

use mlrs_core::config::Settings;
use mlrs_core::error::{MlrsError, MlrsResult};

pub mod artefacts;
pub mod datasets;
pub mod idempotency;
pub mod jobs;

mod convert;
mod schema;

pub use jobs::{JobCursor, JobFilters, WorkerCounts};

/// A SQLite connection pool plus the repositories that operate on it.
#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Open (and create when needed) the database configured in [`Settings`].
    pub async fn connect(settings: &Settings) -> MlrsResult<Self> {
        let options = if let Some(path) = settings.sqlite_path() {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true)
                .foreign_keys(true)
                .journal_mode(SqliteJournalMode::Wal)
                .synchronous(SqliteSynchronous::Normal)
        } else {
            SqliteConnectOptions::from_str(settings.database_url.trim()).map_err(|err| {
                MlrsError::internal(format!("invalid DATABASE_URL: {err}"))
            })?
            .create_if_missing(true)
            .foreign_keys(true)
        };

        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(options)
            .await
            .map_err(|err| {
                MlrsError::service_unavailable(format!("Database not reachable: {err}"))
            })?;

        let database = Database { pool };
        database.migrate().await?;
        Ok(database)
    }

    /// Open an in-memory database (tests and ephemeral embeddings).
    pub async fn in_memory() -> MlrsResult<Self> {
        let options = SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .map_err(|err| MlrsError::internal(format!("failed to open in-memory database: {err}")))?;
        let database = Database { pool };
        database.migrate().await?;
        Ok(database)
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Create the schema. Every statement is idempotent.
    pub async fn migrate(&self) -> MlrsResult<()> {
        for statement in schema::SCHEMA {
            sqlx::query(statement)
                .execute(&self.pool)
                .await
                .map_err(|err| convert::database_error("applying the schema", err))?;
        }
        Ok(())
    }

    /// Readiness probe: `SELECT 1`.
    pub async fn ping(&self) -> MlrsResult<()> {
        sqlx::query_scalar::<_, i64>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|err| convert::database_error("checking readiness", err))?;
        Ok(())
    }
}
