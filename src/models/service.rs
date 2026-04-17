use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Service {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub check_interval_seconds: i32,
    pub expected_status_code: i32,
    pub is_active: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateService {
    pub name: String,
    pub url: String,
    pub check_interval_seconds: Option<i32>,
    pub expected_status_code: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateService {
    pub name: Option<String>,
    pub url: Option<String>,
    pub check_interval_seconds: Option<i32>,
    pub expected_status_code: Option<i32>,
    pub is_active: Option<bool>,
}

pub async fn list_services(pool: &PgPool) -> Result<Vec<Service>, AppError> {
    let services = sqlx::query_as!(
        Service,
        "SELECT id, name, url, check_interval_seconds, expected_status_code,
                is_active, created_at, updated_at
         FROM services
         ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?;
    Ok(services)
}

pub async fn get_service(pool: &PgPool, id: Uuid) -> Result<Service, AppError> {
    sqlx::query_as!(
        Service,
        "SELECT id, name, url, check_interval_seconds, expected_status_code,
                is_active, created_at, updated_at
         FROM services WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

pub async fn create_service(pool: &PgPool, input: CreateService) -> Result<Service, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("name is required".into()));
    }
    if input.url.trim().is_empty() {
        return Err(AppError::Validation("url is required".into()));
    }

    let service = sqlx::query_as!(
        Service,
        "INSERT INTO services (name, url, check_interval_seconds, expected_status_code)
         VALUES ($1, $2, $3, $4)
         RETURNING id, name, url, check_interval_seconds, expected_status_code,
                   is_active, created_at, updated_at",
        input.name,
        input.url,
        input.check_interval_seconds.unwrap_or(60),
        input.expected_status_code.unwrap_or(200),
    )
    .fetch_one(pool)
    .await?;
    Ok(service)
}

pub async fn update_service(
    pool: &PgPool,
    id: Uuid,
    input: UpdateService,
) -> Result<Service, AppError> {
    let existing = get_service(pool, id).await?;

    let service = sqlx::query_as!(
        Service,
        "UPDATE services
         SET name = $1, url = $2, check_interval_seconds = $3,
             expected_status_code = $4, is_active = $5, updated_at = now()
         WHERE id = $6
         RETURNING id, name, url, check_interval_seconds, expected_status_code,
                   is_active, created_at, updated_at",
        input.name.unwrap_or(existing.name),
        input.url.unwrap_or(existing.url),
        input.check_interval_seconds.unwrap_or(existing.check_interval_seconds),
        input.expected_status_code.unwrap_or(existing.expected_status_code),
        input.is_active.unwrap_or(existing.is_active),
        id,
    )
    .fetch_one(pool)
    .await?;
    Ok(service)
}

pub async fn delete_service(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query!("DELETE FROM services WHERE id = $1", id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}
