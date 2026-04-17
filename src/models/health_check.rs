use serde::Serialize;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct HealthCheck {
    pub id: Uuid,
    pub service_id: Uuid,
    pub status: String,
    pub response_time_ms: Option<i32>,
    pub status_code: Option<i32>,
    pub error_message: Option<String>,
    pub checked_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct UptimeStats {
    pub uptime_24h: f64,
    pub uptime_7d: f64,
    pub uptime_30d: f64,
}

pub async fn list_checks(
    pool: &PgPool,
    service_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<HealthCheck>, AppError> {
    let checks = sqlx::query_as!(
        HealthCheck,
        "SELECT id, service_id, status, response_time_ms, status_code, error_message, checked_at
         FROM health_checks
         WHERE service_id = $1
         ORDER BY checked_at DESC
         LIMIT $2 OFFSET $3",
        service_id,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;
    Ok(checks)
}

pub async fn get_uptime(pool: &PgPool, service_id: Uuid) -> Result<UptimeStats, AppError> {
    let row = sqlx::query!(
        r#"SELECT
            COALESCE(
                100.0 * COUNT(*) FILTER (WHERE status = 'up' AND checked_at > now() - INTERVAL '24 hours')
                / NULLIF(COUNT(*) FILTER (WHERE checked_at > now() - INTERVAL '24 hours'), 0),
                100.0
            ) AS "uptime_24h!: f64",
            COALESCE(
                100.0 * COUNT(*) FILTER (WHERE status = 'up' AND checked_at > now() - INTERVAL '7 days')
                / NULLIF(COUNT(*) FILTER (WHERE checked_at > now() - INTERVAL '7 days'), 0),
                100.0
            ) AS "uptime_7d!: f64",
            COALESCE(
                100.0 * COUNT(*) FILTER (WHERE status = 'up' AND checked_at > now() - INTERVAL '30 days')
                / NULLIF(COUNT(*) FILTER (WHERE checked_at > now() - INTERVAL '30 days'), 0),
                100.0
            ) AS "uptime_30d!: f64"
         FROM health_checks
         WHERE service_id = $1"#,
        service_id
    )
    .fetch_one(pool)
    .await?;

    Ok(UptimeStats {
        uptime_24h: row.uptime_24h,
        uptime_7d: row.uptime_7d,
        uptime_30d: row.uptime_30d,
    })
}

pub async fn insert_check(
    pool: &PgPool,
    service_id: Uuid,
    status: &str,
    response_time_ms: Option<i32>,
    status_code: Option<i32>,
    error_message: Option<&str>,
) -> Result<HealthCheck, AppError> {
    let check = sqlx::query_as!(
        HealthCheck,
        "INSERT INTO health_checks (service_id, status, response_time_ms, status_code, error_message)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, service_id, status, response_time_ms, status_code, error_message, checked_at",
        service_id,
        status,
        response_time_ms,
        status_code,
        error_message,
    )
    .fetch_one(pool)
    .await?;
    Ok(check)
}
