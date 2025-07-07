use sqlx::{Pool, Postgres};
use std::time::Duration;
use submerge_base::args::PostgreSQLArgs;

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

pub struct PostgreSQLStorage {
    pub connection_pool: Pool<Postgres>,
}

impl PostgreSQLStorage {
    pub async fn new(args: &PostgreSQLArgs) -> anyhow::Result<Self> {
        Self::new_inner(
            &args.postgres_host,
            args.postgres_port,
            &args.postgres_username,
            &args.postgres_password,
            &args.postgres_db_name,
            args.postgres_connection_timeout_secs,
            args.postgres_pool_max_connections,
        )
        .await
    }

    async fn new_inner(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        database_name: &str,
        connection_timeout_secs: u64,
        pool_max_connections: u32,
    ) -> anyhow::Result<PostgreSQLStorage> {
        Ok(PostgreSQLStorage {
            connection_pool: new_postgres_connection_pool(
                host,
                port,
                username,
                password,
                database_name,
                connection_timeout_secs,
                pool_max_connections,
            )
            .await?,
        })
    }
}
