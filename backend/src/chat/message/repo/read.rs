use crate::chat::message::model::Message;
use crate::common::model::{Error, PageRequest, PageResponse};
use sqlx::{Pool, Postgres};
use std::future::Future;
use std::sync::Arc;

pub trait MessageReadRepo {
    fn find_by_id(
        &self,
        message_id: i64,
    ) -> impl Future<Output = Result<Option<Message>, Error>> + Send;

    fn find_by_conversation_id(
        &self,
        conversation_id: i64,
        req: PageRequest,
    ) -> impl Future<Output = Result<PageResponse<Message>, Error>> + Send;
}

pub struct PostgresMessageReadRepo {
    pool: Arc<Pool<Postgres>>,
}

impl PostgresMessageReadRepo {
    pub fn new(pool: Arc<Pool<Postgres>>) -> Self {
        Self { pool }
    }
}

impl MessageReadRepo for PostgresMessageReadRepo {
    async fn find_by_id(&self, message_id: i64) -> Result<Option<Message>, Error> {
        let query = r#"
            SELECT 
                id, conversation_id, sender_id, text, deleted_at, created_at, updated_at
            FROM 
                "message"
            WHERE 
                id = $1 AND deleted_at IS NULL
        "#;

        sqlx::query_as::<_, Message>(query)
            .bind(message_id)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }

    async fn find_by_conversation_id(
        &self,
        conversation_id: i64,
        req: PageRequest,
    ) -> Result<PageResponse<Message>, Error> {
        let query = r#"
            SELECT 
                id, conversation_id, sender_id, text, deleted_at, created_at, updated_at
            FROM 
                "message"
            WHERE 
                deleted_at IS NULL AND conversation_id = $1 AND id < $2
            ORDER BY 
                id DESC
            LIMIT 
                $3
        "#;

        let messages: Vec<Message> = sqlx::query_as::<_, Message>(query)
            .bind(conversation_id)
            .bind(req.cursor())
            .bind(req.size())
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))?;

        let next_cursor = messages.last().map(|m| m.id);

        let page = PageResponse {
            data: messages,
            size: req.size(),
            next_cursor,
        };

        Ok(page)
    }
}
