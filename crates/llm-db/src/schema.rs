//! SQLite schema for the llm backend.
//!
//! The Python service used PostgreSQL (`JSONB`, `ARRAY`, `UUID`). The local
//! Rust/Tauri build stores the same information in SQLite:
//!
//! * identifiers and timestamps are RFC3339 `TEXT`;
//! * JSON payloads (`columns`, `hyperparameters`, `tags`, …) are `TEXT`;
//! * booleans are `INTEGER` (0/1).
//!
//! Constraints and indexes mirror the original Python `models.py` closely
//! enough that the API contract is unchanged.

/// DDL executed once at startup. Every statement is idempotent.
pub const SCHEMA: &[&str] = &[
    r#"
    CREATE TABLE IF NOT EXISTS datasets (
        id                TEXT PRIMARY KEY,
        original_filename TEXT NOT NULL,
        sha256            TEXT NOT NULL,
        byte_size         INTEGER NOT NULL,
        storage_key       TEXT NOT NULL UNIQUE,
        columns           TEXT NOT NULL,
        row_count         INTEGER NOT NULL DEFAULT 0,
        created_at        TEXT NOT NULL
    )
    "#,
    "CREATE INDEX IF NOT EXISTS datasets_sha256_idx ON datasets (sha256)",
    r#"
    CREATE TABLE IF NOT EXISTS dataset_insights (
        dataset_id    TEXT PRIMARY KEY REFERENCES datasets (id) ON DELETE CASCADE,
        row_count     INTEGER NOT NULL,
        column_count  INTEGER NOT NULL,
        total_missing INTEGER NOT NULL DEFAULT 0,
        columns       TEXT NOT NULL,
        warnings      TEXT NOT NULL DEFAULT '[]',
        created_at    TEXT NOT NULL
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS jobs (
        id             TEXT PRIMARY KEY,
        dataset_id     TEXT NOT NULL REFERENCES datasets (id) ON DELETE CASCADE,
        task_type      TEXT NOT NULL,
        algorithm_id   TEXT NOT NULL,
        feature_columns TEXT NOT NULL,
        label_column   TEXT,
        rl_columns     TEXT,
        hyperparameters TEXT NOT NULL DEFAULT '{}',
        split_ratio    REAL NOT NULL DEFAULT 0.2,
        random_seed    INTEGER NOT NULL DEFAULT 42,
        export_pkl     INTEGER NOT NULL DEFAULT 0,
        export_joblib  INTEGER NOT NULL DEFAULT 0,
        status         TEXT NOT NULL,
        error_code     TEXT,
        error_message  TEXT,
        library_versions TEXT,
        name           TEXT,
        tags           TEXT NOT NULL DEFAULT '[]',
        attempts       INTEGER NOT NULL DEFAULT 0,
        max_attempts   INTEGER NOT NULL DEFAULT 2,
        next_attempt_at TEXT,
        created_at     TEXT NOT NULL,
        updated_at     TEXT NOT NULL,
        CHECK (status IN (
            'queued', 'validating', 'training', 'evaluating',
            'exporting', 'retrying', 'completed', 'failed'
        )),
        CHECK (task_type IN (
            'classification', 'regression', 'semi_supervised',
            'clustering', 'anomaly_detection', 'reinforcement_learning'
        )),
        CHECK (
            (task_type IN ('classification', 'regression', 'semi_supervised')
                AND label_column IS NOT NULL)
            OR
            (task_type IN ('clustering', 'anomaly_detection', 'reinforcement_learning')
                AND label_column IS NULL)
        ),
        CHECK (
            (task_type = 'reinforcement_learning' AND rl_columns IS NOT NULL)
            OR
            (task_type != 'reinforcement_learning' AND rl_columns IS NULL)
        )
    )
    "#,
    "CREATE INDEX IF NOT EXISTS jobs_dataset_id_idx ON jobs (dataset_id)",
    "CREATE INDEX IF NOT EXISTS jobs_task_type_idx ON jobs (task_type)",
    "CREATE INDEX IF NOT EXISTS jobs_status_idx ON jobs (status)",
    "CREATE INDEX IF NOT EXISTS jobs_algorithm_id_idx ON jobs (algorithm_id)",
    "CREATE INDEX IF NOT EXISTS jobs_created_at_id_idx ON jobs (created_at DESC, id DESC)",
    "CREATE INDEX IF NOT EXISTS jobs_name_idx ON jobs (name COLLATE NOCASE)",
    r#"
    CREATE TABLE IF NOT EXISTS job_metrics (
        job_id     TEXT PRIMARY KEY REFERENCES jobs (id) ON DELETE CASCADE,
        metrics    TEXT NOT NULL,
        charts     TEXT,
        created_at TEXT NOT NULL
    )
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS artefacts (
        id          TEXT PRIMARY KEY,
        job_id      TEXT NOT NULL REFERENCES jobs (id) ON DELETE CASCADE,
        format      TEXT NOT NULL,
        filename    TEXT NOT NULL,
        byte_size   INTEGER NOT NULL,
        storage_key TEXT NOT NULL UNIQUE,
        created_at  TEXT NOT NULL,
        CHECK (format IN ('onnx', 'pkl', 'joblib'))
    )
    "#,
    "CREATE INDEX IF NOT EXISTS artefacts_job_id_idx ON artefacts (job_id)",
    "CREATE INDEX IF NOT EXISTS artefacts_format_idx ON artefacts (format)",
    r#"
    CREATE TABLE IF NOT EXISTS idempotency_keys (
        key             TEXT PRIMARY KEY,
        request_hash    TEXT NOT NULL,
        job_id          TEXT REFERENCES jobs (id) ON DELETE SET NULL,
        response_status INTEGER NOT NULL,
        response_body   TEXT NOT NULL,
        created_at      TEXT NOT NULL,
        expires_at      TEXT NOT NULL
    )
    "#,
];
