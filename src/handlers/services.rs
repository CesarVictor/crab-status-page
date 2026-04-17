use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::service::{create_service, delete_service, get_service, list_services, update_service, CreateService, UpdateService},
};

pub async fn handle_list_services(
    State(pool): State<Arc<PgPool>>,
) -> Result<Json<Vec<crate::models::service::Service>>, AppError> {
    let services = list_services(&pool).await?;
    Ok(Json(services))
}

pub async fn handle_get_service(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<Uuid>,
) -> Result<Json<crate::models::service::Service>, AppError> {
    let service = get_service(&pool, id).await?;
    Ok(Json(service))
}

pub async fn handle_create_service(
    State(pool): State<Arc<PgPool>>,
    Json(input): Json<CreateService>,
) -> Result<(StatusCode, Json<crate::models::service::Service>), AppError> {
    let service = create_service(&pool, input).await?;
    Ok((StatusCode::CREATED, Json(service)))
}

pub async fn handle_update_service(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateService>,
) -> Result<Json<crate::models::service::Service>, AppError> {
    let service = update_service(&pool, id, input).await?;
    Ok(Json(service))
}

pub async fn handle_delete_service(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    delete_service(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
