use crate::chat::conversation::model::Conversation;
use crate::common::model::{Error, PageRequest, PageResponse};
use sqlx::{Pool, Postgres};
use std::future::Future;
use std::sync::Arc;

pub trait ConversationReadRepo {
    fn find_by_id(
        &self,
        conversation_id: i64,
    ) -> impl Future<Output = Result<Option<Conversation>, Error>> + Send;

    fn find_by_author_id(
        &self,
        author_id: i64,
        req: PageRequest,
    ) -> impl Future<Output = Result<PageResponse<Conversation>, Error>> + Send;

    fn exists_by_private_id(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<bool, Error>> + Send;
}

pub struct ConversationReadRepoPg {
    pool: Arc<Pool<Postgres>>,
}

impl ConversationReadRepoPg {
    pub fn new(pool: Arc<Pool<Postgres>>) -> Self {
        Self { pool }
    }
}

impl ConversationReadRepo for ConversationReadRepoPg {
    async fn find_by_id(&self, conversation_id: i64) -> Result<Option<Conversation>, Error> {
        let query = r#"
            SELECT 
                id, author_id, type, name, photo_url, deleted_at, created_at, updated_at
            FROM 
                "conversation"
            WHERE 
                id = $1 AND deleted_at IS NULL
        "#;

        sqlx::query_as::<_, Conversation>(query)
            .bind(conversation_id)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }

    async fn find_by_author_id(
        &self,
        author_id: i64,
        req: PageRequest,
    ) -> Result<PageResponse<Conversation>, Error> {
        let query = r#"
            SELECT
                id, author_id, type, name, photo_url, deleted_at, created_at, updated_at
            FROM
                "conversation"
            WHERE
                deleted_at IS NULL AND author_id = $1 AND id < $2
            ORDER
                BY id DESC
            LIMIT
                $3
        "#;

        let conversations: Vec<Conversation> = sqlx::query_as::<_, Conversation>(query)
            .bind(author_id)
            .bind(req.cursor())
            .bind(req.size())
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))?;
        let next_cursor = conversations.last().map(|c| c.id);

        let page = PageResponse {
            data: conversations,
            size: req.size(),
            next_cursor,
        };

        Ok(page)
    }

    async fn exists_by_private_id(&self, private_id: &str) -> Result<bool, Error> {
        let query = r#"
            SELECT EXISTS(
                SELECT 
                    1 
                FROM 
                    "conversation" 
                WHERE
                    private_id = $1
            )
        "#;

        sqlx::query_scalar(query)
            .bind(private_id)
            .fetch_one(&*self.pool)
            .await
            .map_err(|error| Error::InternalServerError(error.to_string()))
    }
}
