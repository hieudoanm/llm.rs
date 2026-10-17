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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;
    use uuid::Uuid;
    use crate::datasets;
    use crate::jobs::writes::insert_job;

    fn create_test_dataset() -> llm_core::domain::Dataset {
        llm_core::domain::Dataset {
            id: Uuid::new_v4(),
            original_filename: "test.csv".to_string(),
            sha256: "abc123".to_string(),
            byte_size: 1000,
            storage_key: "test-key".to_string(),
            columns: vec![],
            row_count: 10,
            created_at: Utc::now(),
        }
    }

    async fn create_test_job(db: &crate::Database, dataset_id: Uuid) -> Uuid {
        let job_id = Uuid::new_v4();
        let job = llm_core::domain::Job {
            id: job_id,
            dataset_id,
            task_type: "classification".to_string(),
            algorithm_id: "algo-1".to_string(),
            feature_columns: vec![],
            label_column: Some("label".to_string()),
            rl_columns: None,
            hyperparameters: serde_json::json!({}),
            split_ratio: 0.8,
            random_seed: 42,
            export_pkl: false,
            export_joblib: true,
            status: "queued".to_string(),
            error_code: None,
            error_message: None,
            library_versions: None,
            name: None,
            tags: vec![],
            attempts: 0,
            max_attempts: 2,
            next_attempt_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        insert_job(db.pool(), &job).await.unwrap();
        job_id
    }

    fn create_test_key() -> String {
        "test-key-123".to_string()
    }

    fn create_test_body_hash() -> String {
        "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string()
    }

    fn create_test_response_body() -> serde_json::Value {
        json!({"result": "success", "data": [1, 2, 3]})
    }

    #[tokio::test]
    async fn test_lookup_miss() {
        let db = crate::Database::in_memory().await.unwrap();
        let dataset = create_test_dataset();
        datasets::insert_dataset(db.pool(), &dataset).await.unwrap();
        let job_id = create_test_job(&db, dataset.id).await;
        let result = lookup(db.pool(), &create_test_key(), &create_test_body_hash()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_lookup_hit_with_valid_key() {
        let db = crate::Database::in_memory().await.unwrap();
        let dataset = create_test_dataset();
        datasets::insert_dataset(db.pool(), &dataset).await.unwrap();
        let job_id = create_test_job(&db, dataset.id).await;
        let key = create_test_key();
        let body_hash = create_test_body_hash();
        let response_body = create_test_response_body();
        record(db.pool(), &key, &body_hash, job_id, &response_body).await.unwrap();

        let result = lookup(db.pool(), &key, &body_hash).await.unwrap();
        assert!(result.is_some());
        let body = result.unwrap();
        assert_eq!(body["result"].as_str(), Some("success"));
    }

    #[tokio::test]
    async fn test_lookup_expired_key() {
        let db = crate::Database::in_memory().await.unwrap();
        let dataset = create_test_dataset();
        datasets::insert_dataset(db.pool(), &dataset).await.unwrap();
        let job_id = create_test_job(&db, dataset.id).await;
        let key = create_test_key();
        let body_hash = create_test_body_hash();
        let response_body = create_test_response_body();

        // Record with short TTL
        record(db.pool(), &key, &body_hash, job_id, &response_body).await.unwrap();
        // Manually expire the key by setting an old expires_at
        sqlx::query(
            "UPDATE idempotency_keys SET expires_at = ?1 WHERE key = ?2",
        )
        .bind("2020-01-01T00:00:00Z")
        .bind(&key)
        .execute(db.pool())
        .await
        .unwrap();

        let result = lookup(db.pool(), &key, &body_hash).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_lookup_different_body_hash() {
        let db = crate::Database::in_memory().await.unwrap();
        let dataset = create_test_dataset();
        datasets::insert_dataset(db.pool(), &dataset).await.unwrap();
        let job_id = create_test_job(&db, dataset.id).await;
        let key = create_test_key();
        let body_hash_1 = create_test_body_hash();
        let body_hash_2 = "0987654321fedcba0987654321fedcba0987654321fedcba0987654321".to_string();
        let response_body_1 = create_test_response_body();

        record(db.pool(), &key, &body_hash_1, job_id, &response_body_1).await.unwrap();

        let result = lookup(db.pool(), &key, &body_hash_2).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_record_overwrites_existing_key() {
        let db = crate::Database::in_memory().await.unwrap();
        let dataset = create_test_dataset();
        datasets::insert_dataset(db.pool(), &dataset).await.unwrap();
        let job_id = create_test_job(&db, dataset.id).await;
        let key = create_test_key();
        let body_hash_1 = create_test_body_hash();
        let body_hash_2 = "0987654321fedcba0987654321fedcba0987654321fedcba0987654321".to_string();
        let response_body_1 = create_test_response_body();
        let response_body_2 = json!({"result": "updated"});

        record(db.pool(), &key, &body_hash_1, job_id, &response_body_1).await.unwrap();
        record(db.pool(), &key, &body_hash_2, job_id, &response_body_2).await.unwrap();

        let result = lookup(db.pool(), &key, &body_hash_2).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap()["result"].as_str(), Some("updated"));
    }

    #[tokio::test]
    async fn test_key_ttl_hours() {
        assert_eq!(KEY_TTL_HOURS, 24);
    }
}
