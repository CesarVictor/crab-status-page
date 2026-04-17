use std::net::SocketAddr;
use std::sync::Arc;

mod config;
mod db;
mod error;
mod handlers;
mod models;
mod monitor;
mod routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "crab_status_page=debug,info".into()),
        )
        .init();

    let cfg = config::Config::from_env();
    let pool = Arc::new(db::pool::create_pool(&cfg.database_url).await);

    let client = reqwest::Client::builder()
        .user_agent("crab-status-page/0.1")
        .build()
        .expect("failed to build HTTP client");

    tokio::spawn(monitor::checker::run_checker(pool.clone(), client));

    let app = routes::build_router(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.port));
    tracing::info!("listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
