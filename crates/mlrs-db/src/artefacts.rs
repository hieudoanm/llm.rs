//! Artefact row persistence.

use sqlx::SqlitePool;
use uuid::Uuid;

use mlrs_core::domain::Artefact;
use mlrs_core::error::MlrsResult;

use crate::convert::{database_error, timestamp, ArtefactRow};

pub async fn insert_artefact(pool: &SqlitePool, artefact: &Artefact) -> MlrsResult<()> {
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

pub async fn list_artefacts(pool: &SqlitePool, job_id: Uuid) -> MlrsResult<Vec<Artefact>> {
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
) -> MlrsResult<Vec<Artefact>> {
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
) -> MlrsResult<Option<Artefact>> {
    let row = sqlx::query_as::<_, ArtefactRow>(
        "SELECT * FROM artefacts WHERE id = ?1 AND job_id = ?2",
    )
    .bind(artefact_id.to_string())
    .bind(job_id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(|err| database_error("loading an artefact", err))?;
    row.map(ArtefactRow::into_domain).transpose()
}

pub async fn delete_artefacts(pool: &SqlitePool, job_id: Uuid) -> MlrsResult<()> {
    sqlx::query("DELETE FROM artefacts WHERE job_id = ?1")
        .bind(job_id.to_string())
        .execute(pool)
        .await
        .map_err(|err| database_error("deleting artefacts", err))?;
    Ok(())
}

/// The storage keys of a job's artefacts, so the caller can remove the files.
pub async fn storage_keys(pool: &SqlitePool, job_id: Uuid) -> MlrsResult<Vec<String>> {
    let keys = sqlx::query_scalar::<_, String>(
        "SELECT storage_key FROM artefacts WHERE job_id = ?1",
    )
    .bind(job_id.to_string())
    .fetch_all(pool)
    .await
    .map_err(|err| database_error("listing artefact storage keys", err))?;
    Ok(keys)
}
