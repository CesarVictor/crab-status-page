use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::health_check::{get_uptime, list_checks, HealthCheck, UptimeStats},
};

#[derive(Debug, Deserialize)]
pub struct Pagination {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

pub async fn handle_list_checks(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<HealthCheck>>, AppError> {
    let checks = list_checks(&pool, id, pagination.limit.min(200), pagination.offset).await?;
    Ok(Json(checks))
}

pub async fn handle_get_uptime(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<Uuid>,
) -> Result<Json<UptimeStats>, AppError> {
    let stats = get_uptime(&pool, id).await?;
    Ok(Json(stats))
}
