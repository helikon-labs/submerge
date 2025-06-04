use sqlx::{Pool, Postgres};
use std::time::Duration;

pub async fn new_postgres_connection_pool(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    database_name: &str,
    connection_timeout_secs: u64,
    pool_max_connections: u32,
) -> anyhow::Result<Pool<Postgres>> {
    log::info!("⚙️ Establishing PostgreSQL connection pool.");
    let connection_str =
        format!("postgres://{username}:{password}@{host}:{port}/{database_name}?sslmode=disable");
    let connection_pool = sqlx::postgres::PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(connection_timeout_secs))
        .max_connections(pool_max_connections)
        .connect(&connection_str)
        .await?;
    log::info!("✅ PostgreSQL connection pool established.");
    Ok(connection_pool)
}
