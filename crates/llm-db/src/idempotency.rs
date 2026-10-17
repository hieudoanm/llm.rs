//! Idempotent job creation: `Idempotency-Key` storage and replay.
//!
//! A key is live for [`KEY_TTL_HOURS`] hours. Replaying a live key with the
//! same canonical request hash returns the cached response body; replaying it
//! with a different body is a `409 IDEMPOTENCY_KEY_EXISTS`.

use chrono::{Duration, Utc};
use serde_json::Value;
use sqlx::SqlitePool;
use uuid::Uuid;

use llm_core::error::{LlmError, LlmResult};

use crate::convert::{database_error, parse_timestamp, timestamp};

pub const KEY_TTL_HOURS: i64 = 24;

/// SHA-256 of the canonicalised request body (sorted keys, no whitespace).
pub fn request_hash(body: &Value) -> String {
    use sha2::{Digest, Sha256};
    let canonical = serde_json::to_string(body).unwrap_or_else(|_| body.to_string());
    let digest = Sha256::digest(canonical.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[derive(Debug, sqlx::FromRow)]
struct IdempotencyRow {
    request_hash: String,
    response_body: String,
    expires_at: String,
}

/// Return the cached response body for `key`, `None` on a miss.
pub async fn lookup(pool: &SqlitePool, key: &str, body_hash: &str) -> LlmResult<Option<Value>> {
    let row = sqlx::query_as::<_, IdempotencyRow>(
        "SELECT request_hash, response_body, expires_at FROM idempotency_keys WHERE key = ?1",
    )
    .bind(key)
    .fetch_optional(pool)
    .await
    .map_err(|err| database_error("loading an idempotency key", err))?;

    let Some(row) = row else {
        return Ok(None);
    };
    if parse_timestamp(&row.expires_at) <= Utc::now() {
        return Ok(None);
    }
    if row.request_hash != body_hash {
        return Err(LlmError::new(
            409,
            "IDEMPOTENCY_KEY_EXISTS",
            "Idempotency-Key already used with a different request body",
        )
        .with_details(serde_json::json!({ "idempotencyKey": key })));
    }
    let body = serde_json::from_str(&row.response_body).map_err(|err| {
        LlmError::internal(format!("failed to decode a cached idempotent response: {err}"))
    })?;
    Ok(Some(body))
}

/// Persist the `202` response body under `key` for future replay.
pub async fn record(
    pool: &SqlitePool,
    key: &str,
    body_hash: &str,
    job_id: Uuid,
    response_body: &Value,
) -> LlmResult<()> {
    let now = Utc::now();
    let expires = now + Duration::hours(KEY_TTL_HOURS);
    let encoded = serde_json::to_string(response_body).map_err(|err| {
        LlmError::internal(format!("failed to encode an idempotent response: {err}"))
    })?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|err| database_error("starting an idempotency transaction", err))?;
    sqlx::query("DELETE FROM idempotency_keys WHERE key = ?1")
        .bind(key)
        .execute(&mut *tx)
        .await
        .map_err(|err| database_error("clearing an idempotency key", err))?;
    sqlx::query(
        r#"
        INSERT INTO idempotency_keys (
            key, request_hash, job_id, response_status, response_body,
            created_at, expires_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
    )
    .bind(key)
    .bind(body_hash)
    .bind(job_id.to_string())
    .bind(202_i64)
    .bind(encoded)
    .bind(timestamp(now))
    .bind(timestamp(expires))
    .execute(&mut *tx)
    .await
    .map_err(|err| database_error("recording an idempotency key", err))?;
    tx.commit()
        .await
        .map_err(|err| database_error("committing an idempotency key", err))?;
    Ok(())
}
