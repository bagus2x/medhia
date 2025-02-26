use crate::common::model::Error;
use crate::user::model::{CreateUserRequest, UpdateUserRequest, User};
use chrono::{DateTime, Local, Utc};
use sqlx::{Pool, Postgres};
use std::future::Future;
use std::sync::Arc;

pub trait UserWriteRepo: Send + Sync {
    fn create(
        &self,
        request: CreateUserRequest,
    ) -> impl Future<Output = Result<User, Error>> + Send;

    fn update(
        &self,
        user_id: i64,
        request: UpdateUserRequest,
    ) -> impl Future<Output = Result<User, Error>> + Send;

    fn delete(&self, user_id: i64) -> impl Future<Output = Result<User, Error>> + Send;
}

pub struct UserWriteRepoPg {
    pool: Arc<Pool<Postgres>>,
}

impl UserWriteRepoPg {
    pub fn new(pool: Arc<Pool<Postgres>>) -> Self {
        Self { pool }
    }
}

impl UserWriteRepo for UserWriteRepoPg {
    async fn create(&self, request: CreateUserRequest) -> Result<User, Error> {
        let query = r#"
            INSERT INTO "user" (
                id, username, email, password, name, photo_url, deleted_at, created_at, updated_at
            ) VALUES (
                default, $1, $2, $3, $4, $5, $6, $7, $8
            )
            RETURNING 
                *
        "#;

        sqlx::query_as::<_, User>(query)
            .bind(request.username)
            .bind(&request.email)
            .bind(&request.password)
            .bind(&request.name)
            .bind(&request.photo_url)
            .bind(Option::<DateTime<Local>>::None) // deleted_at is optional
            .bind(Utc::now()) // created_at
            .bind(Utc::now()) // updated_at
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }

    async fn update(&self, user_id: i64, request: UpdateUserRequest) -> Result<User, Error> {
        let query = r#"
            UPDATE 
                "user"
            SET 
                username = $1,
                email = $2,
                password = $3,
                name = $4,
                photo_url = $5,
                updated_at = $6
            WHERE
                id = $7
            RETURNING 
                id, username, email, password, name, photo_url, deleted_at, created_at,  updated_at
        "#;

        sqlx::query_as::<_, User>(query)
            .bind(&request.username)
            .bind(&request.email)
            .bind(&request.password)
            .bind(&request.name)
            .bind(&request.photo_url)
            .bind(Utc::now()) // updated_at
            .bind(user_id)
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }

    async fn delete(&self, user_id: i64) -> Result<User, Error> {
        let query = r#"
            UPDATE
                "user"
            SET 
                deleted_at = $1,
                updated_at = $2
            WHERE 
                id = $3
            RETURNING 
                id, username, email, password, name, photo_url, deleted_at, created_at,  updated_at
        "#;

        sqlx::query_as::<_, User>(query)
            .bind(Some(Utc::now()))
            .bind(Utc::now())
            .bind(user_id)
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }
}
