use crate::common::config::Config;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Error, Pool, Postgres};
use std::sync::Arc;

pub(crate) struct Database {
    pub(crate) pool: Pool<Postgres>,
}

impl Database {
    pub(crate) async fn init(config: Arc<Config>) -> Result<Self, Error> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(config.acquire_timeout)
            .idle_timeout(config.idle_timeout)
            .connect(&config.database_url)
            .await?;

        sqlx::query("SELECT 1+1").execute(&pool).await?;

        Ok(Database { pool })
    }
}
