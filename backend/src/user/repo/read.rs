use crate::common::model::{Error, PageRequest, PageResponse};
use crate::user::model::User;
use sqlx::{Pool, Postgres};
use std::future::Future;
use std::sync::Arc;

pub trait UserReadRepo: Send + Sync {
    fn find_by_id(&self, user_id: i64) -> impl Future<Output = Result<Option<User>, Error>> + Send;

    fn find_all(
        &self,
        req: PageRequest,
    ) -> impl Future<Output = Result<PageResponse<User>, Error>> + Send;

    fn find_by_username_or_email(
        &self,
        username_or_email: &str,
    ) -> impl Future<Output = Result<Option<User>, Error>> + Send;

    fn exists_by_username(
        &self,
        username: &str,
    ) -> impl Future<Output = Result<bool, Error>> + Send;

    fn exists_by_email(&self, email: &str) -> impl Future<Output = Result<bool, Error>> + Send;
}

pub struct UserReadRepoPg {
    pool: Arc<Pool<Postgres>>,
}

impl UserReadRepoPg {
    pub fn new(pool: Arc<Pool<Postgres>>) -> Self {
        Self { pool }
    }
}

impl UserReadRepo for UserReadRepoPg {
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

    async fn find_by_username_or_email(
        &self,
        username_or_email: &str,
    ) -> Result<Option<User>, Error> {
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
