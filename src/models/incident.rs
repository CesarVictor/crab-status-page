use serde::Serialize;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Incident {
    pub id: Uuid,
    pub service_id: Uuid,
    pub started_at: OffsetDateTime,
    pub resolved_at: Option<OffsetDateTime>,
    pub cause: Option<String>,
}

pub async fn list_incidents(pool: &PgPool) -> Result<Vec<Incident>, AppError> {
    let incidents = sqlx::query_as!(
        Incident,
        "SELECT id, service_id, started_at, resolved_at, cause
         FROM incidents
         ORDER BY started_at DESC"
    )
    .fetch_all(pool)
    .await?;
    Ok(incidents)
}

pub async fn list_active_incidents(pool: &PgPool) -> Result<Vec<Incident>, AppError> {
    let incidents = sqlx::query_as!(
        Incident,
        "SELECT id, service_id, started_at, resolved_at, cause
         FROM incidents
         WHERE resolved_at IS NULL
         ORDER BY started_at DESC"
    )
    .fetch_all(pool)
    .await?;
    Ok(incidents)
}

pub async fn open_incident(
    pool: &PgPool,
    service_id: Uuid,
    cause: Option<&str>,
) -> Result<Incident, AppError> {
    let incident = sqlx::query_as!(
        Incident,
        "INSERT INTO incidents (service_id, cause)
         VALUES ($1, $2)
         RETURNING id, service_id, started_at, resolved_at, cause",
        service_id,
        cause,
    )
    .fetch_one(pool)
    .await?;
    Ok(incident)
}

pub async fn has_open_incident(pool: &PgPool, service_id: Uuid) -> Result<bool, AppError> {
    let row = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM incidents WHERE service_id = $1 AND resolved_at IS NULL) AS \"exists!\"",
        service_id
    )
    .fetch_one(pool)
    .await?;
    Ok(row.exists)
}

pub async fn resolve_incident(pool: &PgPool, service_id: Uuid) -> Result<(), AppError> {
    sqlx::query!(
        "UPDATE incidents SET resolved_at = now()
         WHERE service_id = $1 AND resolved_at IS NULL",
        service_id
    )
    .execute(pool)
    .await?;
    Ok(())
}
