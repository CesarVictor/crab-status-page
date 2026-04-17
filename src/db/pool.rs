use sqlx::PgPool;

pub async fn create_pool(database_url: &str) -> PgPool {
    let pool = PgPool::connect(database_url)
        .await
        .expect("failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    pool
}
