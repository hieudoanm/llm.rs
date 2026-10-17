-- skl SQLite 18 schema (contract). Applied via Alembic in implementation.

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE datasets (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    original_filename TEXT NOT NULL,
    sha256            CHAR(64) NOT NULL,
    byte_size         BIGINT NOT NULL CHECK (byte_size > 0),
    storage_key       TEXT NOT NULL UNIQUE,
    columns           JSONB NOT NULL,
    row_count         INTEGER NOT NULL CHECK (row_count >= 0),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE jobs (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id           UUID NOT NULL REFERENCES datasets (id),
    task_type            TEXT NOT NULL CHECK (
                             task_type IN (
                               'classification', 'regression', 'semi_supervised',
                               'clustering', 'anomaly_detection', 'reinforcement_learning'
                             )
                         ),
    algorithm_id         TEXT NOT NULL,
    feature_columns      JSONB NOT NULL,
    label_column         TEXT,
    rl_columns           JSONB,
    hyperparameters      JSONB NOT NULL DEFAULT '{}'::jsonb,
    split_ratio          DOUBLE PRECISION NOT NULL DEFAULT 0.2
                             CHECK (split_ratio > 0 AND split_ratio < 1),
    random_seed          INTEGER NOT NULL DEFAULT 42,
    export_pkl           BOOLEAN NOT NULL DEFAULT FALSE,
    export_joblib        BOOLEAN NOT NULL DEFAULT FALSE,
    status               TEXT NOT NULL CHECK (
                            status IN (
                              'queued', 'validating', 'training',
                              'evaluating', 'exporting', 'retrying',
                              'completed', 'failed'
                            )
                         ),
    error_code           TEXT,
    error_message        TEXT,
    library_versions     JSONB,
    name                 TEXT CHECK (name IS NULL OR char_length(name) <= 120),
    tags                 TEXT[] NOT NULL DEFAULT '{}'::text[],
    attempts             INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    max_attempts         INTEGER NOT NULL DEFAULT 2 CHECK (max_attempts BETWEEN 1 AND 5),
    next_attempt_at      TIMESTAMPTZ,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT jobs_label_column_matches_task CHECK (
        (task_type IN ('classification', 'regression', 'semi_supervised')
            AND label_column IS NOT NULL)
        OR
        (task_type IN ('clustering', 'anomaly_detection', 'reinforcement_learning')
            AND label_column IS NULL)
    ),
    CONSTRAINT jobs_rl_columns_matches_task CHECK (
        (task_type = 'reinforcement_learning' AND rl_columns IS NOT NULL)
        OR
        (task_type <> 'reinforcement_learning' AND rl_columns IS NULL)
    )
);

CREATE INDEX jobs_created_at_id_idx ON jobs (created_at DESC, id DESC);
CREATE INDEX jobs_status_idx ON jobs (status);
CREATE INDEX jobs_tags_idx ON jobs USING GIN (tags);
CREATE INDEX jobs_algorithm_id_idx ON jobs (algorithm_id);
CREATE INDEX jobs_name_lower_idx ON jobs (lower(name));

CREATE TABLE job_metrics (
    job_id     UUID PRIMARY KEY REFERENCES jobs (id) ON DELETE CASCADE,
    metrics    JSONB NOT NULL,
    charts     JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE artefacts (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id      UUID NOT NULL REFERENCES jobs (id) ON DELETE CASCADE,
    format      TEXT NOT NULL CHECK (format IN ('onnx', 'pkl', 'joblib')),
    filename    TEXT NOT NULL,
    byte_size   BIGINT NOT NULL CHECK (byte_size > 0),
    storage_key TEXT NOT NULL UNIQUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (job_id, format)
);

CREATE TABLE idempotency_keys (
    key             TEXT PRIMARY KEY CHECK (char_length(key) BETWEEN 1 AND 255),
    request_hash    TEXT NOT NULL,
    job_id          UUID REFERENCES jobs (id),
    response_status INTEGER NOT NULL,
    response_body   JSONB NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ NOT NULL
);

-- Per-dataset column profile computed once at upload time.
CREATE TABLE dataset_insights (
    dataset_id       UUID PRIMARY KEY REFERENCES datasets (id) ON DELETE CASCADE,
    row_count        INTEGER NOT NULL CHECK (row_count >= 0),
    column_count     INTEGER NOT NULL CHECK (column_count >= 0),
    total_missing    INTEGER NOT NULL DEFAULT 0 CHECK (total_missing >= 0),
    columns          JSONB NOT NULL,
    warnings         JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
