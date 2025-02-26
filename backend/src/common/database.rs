use crate::common::config::Config;
use crate::common::model::Error;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Acquire, Executor, PgPool, Pool, Postgres, Transaction};
use std::cell::RefCell;
use std::future::Future;
use std::sync::Arc;
use tokio::task_local;

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
            .await
            .map_err(|err| Error::InternalServerError(err.to_string()))?;

        sqlx::query("SELECT 1+1")
            .execute(&pool)
            .await
            .map_err(|err| Error::InternalServerError(err.to_string()))?;

        Ok(Database { pool })
    }
}

task_local! {
    pub static TRANSACTION: RefCell<Option<Transaction<'static, Postgres>>>;
}

pub trait UnitOfWork {
    fn run<F, R>(&self, f: F) -> impl Future<Output = Result<R, Error>> + Send
    where
        F: Future<Output = Result<R, Error>> + Send,
        R: Send;
}

pub struct UnitOfWorkPg {
    pool: Arc<PgPool>,
}

impl UnitOfWorkPg {
    pub fn new(pool: Arc<PgPool>) -> Self {
        UnitOfWorkPg { pool }
    }
}

impl UnitOfWork for UnitOfWorkPg {
    async fn run<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: Future<Output = Result<R, Error>> + Send,
        R: Send,
    {
        let tx = self.pool.begin().await.unwrap();

        let result = TRANSACTION
            .scope(RefCell::new(Some(tx)), async {
                let result = f.await;

                match result {
                    Ok(_) => {
                        if let Some(tx) = TRANSACTION.with(|cell| cell.borrow_mut().take()) {
                            tx.commit().await.unwrap();
                        }
                    }
                    Err(_) => {
                        if let Some(tx) = TRANSACTION.with(|cell| cell.borrow_mut().take()) {
                            tx.rollback().await.unwrap();
                        }
                    }
                }
                result
            })
            .await;

        result
    }
}
