use axum::{routing::get, Router};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::services::ServeDir;

use crate::handlers::{
    checks::{handle_get_uptime, handle_list_checks},
    incidents::{handle_list_active_incidents, handle_list_incidents},
    services::{
        handle_create_service, handle_delete_service, handle_get_service, handle_list_services,
        handle_update_service,
    },
    stats::handle_get_stats,
};

pub fn build_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route("/api/services", get(handle_list_services).post(handle_create_service))
        .route(
            "/api/services/:id",
            get(handle_get_service)
                .put(handle_update_service)
                .delete(handle_delete_service),
        )
        .route("/api/services/:id/checks", get(handle_list_checks))
        .route("/api/services/:id/uptime", get(handle_get_uptime))
        .route("/api/incidents", get(handle_list_incidents))
        .route("/api/incidents/active", get(handle_list_active_incidents))
        .route("/api/stats", get(handle_get_stats))
        .with_state(pool)
        .fallback_service(ServeDir::new("static"))
}
