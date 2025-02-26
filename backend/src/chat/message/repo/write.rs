use crate::chat::message::model::Message;
use crate::common::model::Error;
use sqlx::{Pool, Postgres};
use std::future::Future;
use std::sync::Arc;

pub trait MessageWriteRepo {
    fn create(&self, message: Message) -> impl Future<Output = Result<Message, Error>> + Send;

    fn update(&self, message: Message) -> impl Future<Output = Result<Message, Error>> + Send;

    fn delete(&self, message_id: i64) -> impl Future<Output = Result<(), Error>> + Send;
}

pub struct PostgresMessageWriteRepo {
    pool: Arc<Pool<Postgres>>,
}

impl PostgresMessageWriteRepo {
    pub fn new(pool: Arc<Pool<Postgres>>) -> Self {
        Self { pool }
    }
}

impl MessageWriteRepo for PostgresMessageWriteRepo {
    async fn create(&self, message: Message) -> Result<Message, Error> {
        let query = r#"
            INSERT INTO "message" (
                id, conversation_id, sender_id, text, deleted_at, created_at, updated_at
            ) VALUES (
                default, $1, $2, $3, $4, $5, $6
            )
            RETURNING 
                *
        "#;

        sqlx::query_as::<_, Message>(query)
            .bind(&message.conversation_id)
            .bind(&message.sender_id)
            .bind(&message.text)
            .bind(&message.deleted_at)
            .bind(&message.created_at)
            .bind(&message.updated_at)
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }

    async fn update(&self, message: Message) -> Result<Message, Error> {
        let query = r#"
            UPDATE 
                "message"
            SET 
                text = $1,
                deleted_at = $2,
                updated_at = $3
            WHERE
                id = $4
            RETURNING 
                id, conversation_id, sender_id, text, deleted_at, created_at, updated_at
        "#;

        sqlx::query_as::<_, Message>(query)
            .bind(&message.text)
            .bind(&message.deleted_at)
            .bind(&message.updated_at)
            .bind(&message.id)
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }

    async fn delete(&self, message_id: i64) -> Result<(), Error> {
        let query = r#"
            UPDATE
                "message"
            SET 
                deleted_at = $1,
                updated_at = $2
            WHERE 
                id = $3
            RETURNING 
                id, conversation_id, sender_id, text, deleted_at, created_at, updated_at
        "#;

        sqlx::query(query)
            .bind(message_id)
            .execute(&*self.pool)
            .await
            .map(|_| ())
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }
}
