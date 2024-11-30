use crate::common::model::{Error, PageRequest, PageResponse};
use crate::user::model::User;
use axum::async_trait;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

#[async_trait]
pub trait UserReadRepo {
    async fn find_by_id(&self, user_id: i64) -> Result<Option<User>, Error>;

    async fn find_all(&self, req: PageRequest) -> Result<PageResponse<User>, Error>;

    async fn find_by_username_or_email(&self, username_or_email: &str) -> Result<Option<User>, Error>;

    async fn exists_by_username(&self, username: &str) -> Result<bool, Error>;

    async fn exists_by_email(&self, email: &str) -> Result<bool, Error>;
}

pub struct PostgresUserReadRepo {
    pool: Arc<Pool<Postgres>>,
}

impl PostgresUserReadRepo {
    pub fn new(pool: Arc<Pool<Postgres>>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserReadRepo for PostgresUserReadRepo {
    async fn find_by_id(&self, user_id: i64) -> Result<Option<User>, Error> {
        let query = r#"
            SELECT 
                id, username, email, password, name, photo_url, deleted_at, created_at,  updated_at
            FROM 
                "user"
            WHERE 
                id = $1 AND deleted_at IS NULL
        "#;

        sqlx::query_as::<_, User>(query)
            .bind(user_id)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }

    async fn find_all(&self, req: PageRequest) -> Result<PageResponse<User>, Error> {
        let query = r#"
            SELECT
                id, username, email, password, name, photo_url, deleted_at, created_at,  updated_at
            FROM
                "user"
            WHERE
                deleted_at IS NULL AND id < $1
            ORDER
                BY id DESC
            LIMIT
                $2
        "#;

        let users: Vec<User> = sqlx::query_as::<_, User>(query)
            .bind(req.cursor())
            .bind(req.size())
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))?;
        let next_cursor = users.last().map(|u| u.id);

        let page = PageResponse {
            data: users,
            size: req.size(),
            next_cursor,
        };

        Ok(page)
    }

    async fn find_by_username_or_email(&self, username_or_email: &str) -> Result<Option<User>, Error> {
        let query = r#"
            SELECT 
                id, username, email, password, name, photo_url, deleted_at, created_at,  updated_at
            FROM 
                "user"
            WHERE 
                (username = $1 OR email = $1) AND deleted_at IS NULL
        "#;

        sqlx::query_as::<_, User>(query)
            .bind(username_or_email)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }

    async fn exists_by_username(&self, username: &str) -> Result<bool, Error> {
        let query = r#"
            SELECT 
                COUNT(1) > 0
            FROM 
                "user"
            WHERE 
                username = $1 AND deleted_at IS NULL
        "#;

        sqlx::query_scalar(query)
            .bind(username)
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }

    async fn exists_by_email(&self, email: &str) -> Result<bool, Error> {
        let query = r#"
            SELECT 
                COUNT(1) > 0
            FROM 
                "user"
            WHERE 
                email = $1 AND deleted_at IS NULL
        "#;

        sqlx::query_scalar(query)
            .bind(email)
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }
}
