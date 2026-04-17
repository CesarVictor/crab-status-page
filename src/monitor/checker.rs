use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::Client;
use sqlx::PgPool;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::models::{
    health_check::insert_check,
    incident::{has_open_incident, open_incident, resolve_incident},
    service::Service,
};

pub async fn run_checker(pool: Arc<PgPool>, client: Client) {
    loop {
        match fetch_due_services(&pool).await {
            Ok(services) => {
                let handles: Vec<_> = services
                    .into_iter()
                    .map(|svc| {
                        let pool = pool.clone();
                        let client = client.clone();
                        tokio::spawn(async move {
                            check_service(&pool, &client, svc).await;
                        })
                    })
                    .collect();

                for handle in handles {
                    if let Err(e) = handle.await {
                        error!("checker task panicked: {e}");
                    }
                }
            }
            Err(e) => error!("failed to fetch due services: {e}"),
        }

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

async fn fetch_due_services(pool: &PgPool) -> Result<Vec<Service>, sqlx::Error> {
    sqlx::query_as!(
        Service,
        r#"SELECT s.id, s.name, s.url, s.check_interval_seconds, s.expected_status_code,
                  s.is_active, s.created_at, s.updated_at
           FROM services s
           WHERE s.is_active = true
           AND (
               NOT EXISTS (SELECT 1 FROM health_checks hc WHERE hc.service_id = s.id)
               OR (
                   SELECT MAX(hc2.checked_at) FROM health_checks hc2 WHERE hc2.service_id = s.id
               ) < now() - make_interval(secs => s.check_interval_seconds)
           )"#
    )
    .fetch_all(pool)
    .await
}

async fn check_service(pool: &PgPool, client: &Client, service: Service) {
    let start = Instant::now();

    let result = client
        .get(&service.url)
        .timeout(Duration::from_secs(10))
        .send()
        .await;

    let elapsed_ms = start.elapsed().as_millis() as i32;

    match result {
        Ok(response) => {
            let status_code = response.status().as_u16() as i32;
            let is_up = status_code == service.expected_status_code;
            let status = if is_up { "up" } else { "down" };

            if let Err(e) = insert_check(
                pool,
                service.id,
                status,
                Some(elapsed_ms),
                Some(status_code),
                None,
            )
            .await
            {
                error!(service_id = %service.id, "failed to insert check: {e}");
                return;
            }

            if is_up {
                if let Err(e) = resolve_incident(pool, service.id).await {
                    error!(service_id = %service.id, "failed to resolve incident: {e}");
                }
                info!(service = %service.name, status = "up", response_ms = elapsed_ms);
            } else {
                let cause = format!("unexpected status code: {status_code}");
                handle_down(pool, service.id, Some(&cause)).await;
                warn!(service = %service.name, status = "down", http_code = status_code);
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            if let Err(db_err) =
                insert_check(pool, service.id, "down", None, None, Some(&error_msg)).await
            {
                error!(service_id = %service.id, "failed to insert check: {db_err}");
                return;
            }
            handle_down(pool, service.id, Some(&error_msg)).await;
            warn!(service = %service.name, "check failed: {e}");
        }
    }
}

async fn handle_down(pool: &PgPool, service_id: Uuid, cause: Option<&str>) {
    match has_open_incident(pool, service_id).await {
        Ok(false) => {
            if let Err(e) = open_incident(pool, service_id, cause).await {
                error!(service_id = %service_id, "failed to open incident: {e}");
            }
        }
        Ok(true) => {}
        Err(e) => error!(service_id = %service_id, "failed to check open incident: {e}"),
    }
}
