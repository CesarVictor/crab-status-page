use axum::{extract::State, Json};
use serde::Serialize;
use sqlx::PgPool;
use std::sync::Arc;

use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct GlobalStats {
    pub total_services: i64,
    pub active_services: i64,
    pub avg_uptime_24h: f64,
    pub total_checks_today: i64,
}

pub async fn handle_get_stats(
    State(pool): State<Arc<PgPool>>,
) -> Result<Json<GlobalStats>, AppError> {
    let row = sqlx::query!(
        r#"SELECT
            (SELECT COUNT(*) FROM services) AS "total_services!: i64",
            (SELECT COUNT(*) FROM services WHERE is_active = true) AS "active_services!: i64",
            COALESCE(
                (SELECT 100.0 * COUNT(*) FILTER (WHERE status = 'up')::float8
                    / NULLIF(COUNT(*), 0)
                 FROM health_checks
                 WHERE checked_at > now() - INTERVAL '24 hours'),
                100.0
            ) AS "avg_uptime_24h!: f64",
            (SELECT COUNT(*) FROM health_checks
             WHERE checked_at > now() - INTERVAL '24 hours') AS "total_checks_today!: i64""#
    )
    .fetch_one(&*pool)
    .await?;

    Ok(Json(GlobalStats {
        total_services: row.total_services,
        active_services: row.active_services,
        avg_uptime_24h: row.avg_uptime_24h,
        total_checks_today: row.total_checks_today,
    }))
}
