//! Artefact row persistence.

use sqlx::SqlitePool;
use uuid::Uuid;

use llm_core::domain::Artefact;
use llm_core::error::LlmResult;

use crate::convert::{database_error, timestamp, ArtefactRow};

pub async fn insert_artefact(pool: &SqlitePool, artefact: &Artefact) -> LlmResult<()> {
    sqlx::query(
        r#"
        INSERT INTO artefacts (
            id, job_id, format, filename, byte_size, storage_key, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
    )
    .bind(artefact.id.to_string())
    .bind(artefact.job_id.to_string())
    .bind(&artefact.format)
    .bind(&artefact.filename)
    .bind(artefact.byte_size)
    .bind(&artefact.storage_key)
    .bind(timestamp(artefact.created_at))
    .execute(pool)
    .await
    .map_err(|err| database_error("inserting an artefact", err))?;
    Ok(())
}

pub async fn list_artefacts(pool: &SqlitePool, job_id: Uuid) -> LlmResult<Vec<Artefact>> {
    let rows = sqlx::query_as::<_, ArtefactRow>(
        "SELECT * FROM artefacts WHERE job_id = ?1 ORDER BY created_at ASC, id ASC",
    )
    .bind(job_id.to_string())
    .fetch_all(pool)
    .await
    .map_err(|err| database_error("listing artefacts", err))?;
    rows.into_iter().map(ArtefactRow::into_domain).collect()
}

pub async fn list_artefacts_for_jobs(
    pool: &SqlitePool,
    job_ids: &[Uuid],
) -> LlmResult<Vec<Artefact>> {
    if job_ids.is_empty() {
        return Ok(Vec::new());
    }
    let mut builder =
        sqlx::QueryBuilder::<sqlx::Sqlite>::new("SELECT * FROM artefacts WHERE job_id IN (");
    let mut separated = builder.separated(", ");
    for job_id in job_ids {
        separated.push_bind(job_id.to_string());
    }
    separated.push_unseparated(") ORDER BY created_at ASC, id ASC");

    let rows = builder
        .build_query_as::<ArtefactRow>()
        .fetch_all(pool)
        .await
        .map_err(|err| database_error("listing artefacts", err))?;
    rows.into_iter().map(ArtefactRow::into_domain).collect()
}

pub async fn get_artefact(
    pool: &SqlitePool,
    job_id: Uuid,
    artefact_id: Uuid,
) -> LlmResult<Option<Artefact>> {
    let row =
        sqlx::query_as::<_, ArtefactRow>("SELECT * FROM artefacts WHERE id = ?1 AND job_id = ?2")
            .bind(artefact_id.to_string())
            .bind(job_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|err| database_error("loading an artefact", err))?;
    row.map(ArtefactRow::into_domain).transpose()
}

pub async fn delete_artefacts(pool: &SqlitePool, job_id: Uuid) -> LlmResult<()> {
    sqlx::query("DELETE FROM artefacts WHERE job_id = ?1")
        .bind(job_id.to_string())
        .execute(pool)
        .await
        .map_err(|err| database_error("deleting artefacts", err))?;
    Ok(())
}

/// The storage keys of a job's artefacts, so the caller can remove the files.
pub async fn storage_keys(pool: &SqlitePool, job_id: Uuid) -> LlmResult<Vec<String>> {
    let keys =
        sqlx::query_scalar::<_, String>("SELECT storage_key FROM artefacts WHERE job_id = ?1")
            .bind(job_id.to_string())
            .fetch_all(pool)
            .await
            .map_err(|err| database_error("listing artefact storage keys", err))?;
    Ok(keys)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datasets;
    use crate::jobs::writes::insert_job;
    use chrono::Utc;
    use llm_core::domain::{Artefact, Dataset, Job};
    use serde_json::Value;
    use uuid::Uuid;

    fn create_test_dataset() -> Dataset {
        Dataset {
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
        let job = Job {
            id: job_id,
            dataset_id,
            task_type: "classification".to_string(),
            algorithm_id: "algo-1".to_string(),
            feature_columns: vec![],
            label_column: Some("label".to_string()),
            rl_columns: None,
            hyperparameters: Value::Object(Default::default()),
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

    fn create_test_artefact(job_id: Uuid, storage_key: &str) -> Artefact {
        Artefact {
            id: Uuid::new_v4(),
            job_id,
            format: "onnx".to_string(),
            filename: "model.onnx".to_string(),
            byte_size: 2048,
            storage_key: storage_key.to_string(),
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_insert_and_list_artefacts() {
        let db = crate::Database::in_memory().await.unwrap();
        let dataset = create_test_dataset();
        datasets::insert_dataset(db.pool(), &dataset).await.unwrap();
        let job_id = create_test_job(&db, dataset.id).await;
        let artefact = create_test_artefact(job_id, "storage-key-1");
        insert_artefact(db.pool(), &artefact).await.unwrap();
        let retrieved = list_artefacts(db.pool(), job_id).await.unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].format, "onnx");
        assert_eq!(retrieved[0].filename, "model.onnx");
    }

    #[tokio::test]
    async fn test_list_artefacts_empty() {
        let db = crate::Database::in_memory().await.unwrap();
        let retrieved = list_artefacts(db.pool(), Uuid::new_v4()).await.unwrap();
        assert!(retrieved.is_empty());
    }

    #[tokio::test]
    async fn test_list_artefacts_multiple() {
        let db = crate::Database::in_memory().await.unwrap();
        let dataset = create_test_dataset();
        datasets::insert_dataset(db.pool(), &dataset).await.unwrap();
        let job_id = create_test_job(&db, dataset.id).await;
        insert_artefact(db.pool(), &create_test_artefact(job_id, "storage-key-1"))
            .await
            .unwrap();
        let mut artefact2 = create_test_artefact(job_id, "storage-key-2");
        artefact2.id = Uuid::new_v4();
        artefact2.filename = "model.pkl".to_string();
        insert_artefact(db.pool(), &artefact2).await.unwrap();
        let retrieved = list_artefacts(db.pool(), job_id).await.unwrap();
        assert_eq!(retrieved.len(), 2);
    }

    #[tokio::test]
    async fn test_list_artefacts_for_jobs_empty() {
        let db = crate::Database::in_memory().await.unwrap();
        let result = list_artefacts_for_jobs(db.pool(), &[]).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_list_artefacts_for_jobs() {
        let db = crate::Database::in_memory().await.unwrap();
        let dataset = create_test_dataset();
        datasets::insert_dataset(db.pool(), &dataset).await.unwrap();
        let job_id = create_test_job(&db, dataset.id).await;
        insert_artefact(db.pool(), &create_test_artefact(job_id, "storage-key-1"))
            .await
            .unwrap();
        let result = list_artefacts_for_jobs(db.pool(), &[job_id]).await.unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_get_artefact() {
        let db = crate::Database::in_memory().await.unwrap();
        let dataset = create_test_dataset();
        datasets::insert_dataset(db.pool(), &dataset).await.unwrap();
        let job_id = create_test_job(&db, dataset.id).await;
        let artefact = create_test_artefact(job_id, "storage-key-1");
        insert_artefact(db.pool(), &artefact).await.unwrap();
        let retrieved = get_artefact(db.pool(), job_id, artefact.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, artefact.id);
    }

    #[tokio::test]
    async fn test_get_artefact_not_found() {
        let db = crate::Database::in_memory().await.unwrap();
        let result = get_artefact(db.pool(), Uuid::new_v4(), Uuid::new_v4())
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_artefact_wrong_job() {
        let db = crate::Database::in_memory().await.unwrap();
        let dataset = create_test_dataset();
        datasets::insert_dataset(db.pool(), &dataset).await.unwrap();
        let job_id1 = create_test_job(&db, dataset.id).await;
        let job_id2 = Uuid::new_v4();
        let artefact = create_test_artefact(job_id1, "storage-key-1");
        insert_artefact(db.pool(), &artefact).await.unwrap();
        let result = get_artefact(db.pool(), job_id2, artefact.id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_artefacts() {
        let db = crate::Database::in_memory().await.unwrap();
        let dataset = create_test_dataset();
        datasets::insert_dataset(db.pool(), &dataset).await.unwrap();
        let job_id = create_test_job(&db, dataset.id).await;
        insert_artefact(db.pool(), &create_test_artefact(job_id, "storage-key-1"))
            .await
            .unwrap();
        delete_artefacts(db.pool(), job_id).await.unwrap();
        let retrieved = list_artefacts(db.pool(), job_id).await.unwrap();
        assert!(retrieved.is_empty());
    }

    #[tokio::test]
    async fn test_storage_keys() {
        let db = crate::Database::in_memory().await.unwrap();
        let dataset = create_test_dataset();
        datasets::insert_dataset(db.pool(), &dataset).await.unwrap();
        let job_id = create_test_job(&db, dataset.id).await;
        insert_artefact(db.pool(), &create_test_artefact(job_id, "storage-key-1"))
            .await
            .unwrap();
        let keys = storage_keys(db.pool(), job_id).await.unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], "storage-key-1");
    }

    #[tokio::test]
    async fn test_storage_keys_empty() {
        let db = crate::Database::in_memory().await.unwrap();
        let keys = storage_keys(db.pool(), Uuid::new_v4()).await.unwrap();
        assert!(keys.is_empty());
    }
}
