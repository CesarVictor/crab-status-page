use sqlx::postgres::PgConnectOptions;
use sqlx::PgPool;
use std::str::FromStr;

pub async fn create_pool(database_url: &str) -> PgPool {
    let opts = connect_options(database_url);

    let pool = PgPool::connect_with(opts)
        .await
        .expect("failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    pool
}

fn connect_options(url: &str) -> PgConnectOptions {
    // Cloud SQL Unix socket URLs look like:
    //   postgres://user:pass@/dbname?host=/cloudsql/project:region:instance
    // url::Url rejects the empty host, so we handle this case manually.
    if let Some(q_idx) = url.find("?host=/") {
        let socket = url[q_idx + "?host=".len()..]
            .split('&')
            .next()
            .unwrap_or("");

        // Make the base URL parseable by inserting a placeholder host
        let base = url[..q_idx].replacen("@/", "@localhost/", 1);

        return PgConnectOptions::from_str(&base)
            .expect("invalid DATABASE_URL")
            .socket(socket);
    }

    PgConnectOptions::from_str(url).expect("invalid DATABASE_URL")
}
