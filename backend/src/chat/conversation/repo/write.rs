use crate::chat::conversation::model::Conversation;
use crate::common::database::TRANSACTION;
use crate::common::model::Error;
use sqlx::{Executor, Pool, Postgres};
use std::future::Future;
use std::sync::Arc;

pub trait ConversationWriteRepo {
    fn create(
        &self,
        conversation: Conversation,
    ) -> impl Future<Output = Result<Conversation, Error>> + Send;

    fn update(
        &self,
        conversation: Conversation,
    ) -> impl Future<Output = Result<Conversation, Error>> + Send;

    fn delete(&self, conversation_id: i64) -> impl Future<Output = Result<(), Error>> + Send;
}

pub struct ConversationWriteRepoPg {
    pool: Arc<Pool<Postgres>>,
}

impl ConversationWriteRepoPg {
    pub fn new(pool: Arc<Pool<Postgres>>) -> Self {
        Self { pool }
    }
}

impl ConversationWriteRepo for ConversationWriteRepoPg {
    async fn create(&self, conversation: Conversation) -> Result<Conversation, Error> {
        let query = r#"
            INSERT INTO "conversation" (
                id, private_id, author_id, type, name, photo_url, deleted_at, created_at, updated_at
            ) VALUES (
                default, $1, $2, $3, $4, $5, $6, $7, $8
            )
            RETURNING 
                *
        "#;

        TRANSACTION
            .with(|cell| async {
                let tx = (*(cell.borrow_mut())).unwrap();
                sqlx::query_as::<_, Conversation>(query)
                    .bind(&conversation.private_id)
                    .bind(&conversation.author_id)
                    .bind(&conversation.r#type)
                    .bind(&conversation.name)
                    .bind(&conversation.photo_url)
                    .bind(&conversation.deleted_at)
                    .bind(&conversation.created_at)
                    .bind(&conversation.updated_at)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|e| Error::InternalServerError(e.to_string()))
            })
            .await
    }

    async fn update(&self, conversation: Conversation) -> Result<Conversation, Error> {
        let query = r#"
            UPDATE 
                "conversation"
            SET 
                name = $1,
                type = $2,
                photo_url = $3,
                deleted_at = $4
                updated_at = $5,
            WHERE
                id = $6
            RETURNING 
                id, author_id, type, name, photo_url, deleted_at, created_at, updated_at
        "#;

        sqlx::query_as::<_, Conversation>(query)
            .bind(&conversation.name)
            .bind(&conversation.r#type)
            .bind(&conversation.photo_url)
            .bind(&conversation.deleted_at)
            .bind(&conversation.updated_at)
            .bind(&conversation.id)
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }

    async fn delete(&self, conversation_id: i64) -> Result<(), Error> {
        let query = r#"
            UPDATE
                "conversation"
            SET 
                deleted_at = $1,
                updated_at = $2
            WHERE 
                id = $3
        "#;

        sqlx::query(query)
            .bind(conversation_id)
            .execute(&*self.pool)
            .await
            .map(|_| ())
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }
}
