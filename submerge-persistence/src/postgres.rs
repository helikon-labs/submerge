use sqlx::{Pool, Postgres};
use std::time::Duration;
use submerge_config::Config;

fn get_postgres_url(config: &Config) -> String {
    format!(
        "postgres://{}:{}@{}:{}/{}?sslmode=disable",
        config.postgres.username,
        config.postgres.password,
        config.postgres.host,
        config.postgres.port,
        config.postgres.database_name,
    )
}

pub async fn new_postgres_connection_pool(config: &Config) -> anyhow::Result<Pool<Postgres>> {
    log::info!("Establishing PostgreSQL connection pool...");
    let connection_pool = sqlx::postgres::PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(
            config.postgres.connection_timeout_seconds,
        ))
        .max_connections(config.postgres.pool_max_connections)
        .connect(&get_postgres_url(config))
        .await?;
    log::info!("PostgreSQL connection pool established.");
    Ok(connection_pool)
}
