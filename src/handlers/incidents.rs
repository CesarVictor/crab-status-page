use axum::{extract::State, Json};
use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    error::AppError,
    models::incident::{list_active_incidents, list_incidents, Incident},
};

pub async fn handle_list_incidents(
    State(pool): State<Arc<PgPool>>,
) -> Result<Json<Vec<Incident>>, AppError> {
    let incidents = list_incidents(&pool).await?;
    Ok(Json(incidents))
}

pub async fn handle_list_active_incidents(
    State(pool): State<Arc<PgPool>>,
) -> Result<Json<Vec<Incident>>, AppError> {
    let incidents = list_active_incidents(&pool).await?;
    Ok(Json(incidents))
}
