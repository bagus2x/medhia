use crate::chat::participant::model::Participant;
use crate::common::model::{Error, PageRequest, PageResponse};
use axum::async_trait;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

#[async_trait]
pub trait ConversationParticipantReadRepo {
    async fn find_by_id(&self, id: i64) -> Result<Option<Participant>, Error>;

    async fn find_by_conversation_id(
        &self,
        conversation_id: i64,
        req: PageRequest,
    ) -> Result<PageResponse<Participant>, Error>;

    async fn exists_by_conversation_and_user(
        &self,
        conversation_id: i64,
        user_id: i64,
    ) -> Result<bool, Error>;
}

pub struct ParticipantReadRepoPg {
    pool: Arc<Pool<Postgres>>,
}

impl ParticipantReadRepoPg {
    pub fn new(pool: Arc<Pool<Postgres>>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ConversationParticipantReadRepo for ParticipantReadRepoPg {
    async fn find_by_id(&self, id: i64) -> Result<Option<Participant>, Error> {
        let query = r#"
            SELECT 
                id, conversation_id, user_id, joined_at, roles, deleted_at, created_at
            FROM 
                "conversation_participant"
            WHERE 
                id = $1 AND deleted_at IS NULL
        "#;

        sqlx::query_as::<_, Participant>(query)
            .bind(id)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }

    async fn find_by_conversation_id(
        &self,
        conversation_id: i64,
        req: PageRequest,
    ) -> Result<PageResponse<Participant>, Error> {
        let query = r#"
            SELECT 
                id, conversation_id, user_id, joined_at, roles, deleted_at, created_at
            FROM 
                "conversation_participant"
            WHERE 
                deleted_at IS NULL AND conversation_id = $1 AND id < $2
            ORDER BY 
                id DESC
            LIMIT 
                $3
        "#;

        let participants: Vec<Participant> = sqlx::query_as::<_, Participant>(query)
            .bind(conversation_id)
            .bind(req.cursor())
            .bind(req.size())
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))?;

        let next_cursor = participants.last().map(|p| p.id);

        let page = PageResponse {
            data: participants,
            size: req.size(),
            next_cursor,
        };

        Ok(page)
    }

    async fn exists_by_conversation_and_user(
        &self,
        conversation_id: i64,
        user_id: i64,
    ) -> Result<bool, Error> {
        let query = r#"
            SELECT EXISTS(
                SELECT 
                    1 
                FROM 
                    "conversation_participant"
                WHERE 
                    conversation_id = $1 AND user_id = $2 AND deleted_at IS NULL
            )
        "#;

        sqlx::query_scalar(query)
            .bind(conversation_id)
            .bind(user_id)
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }
}
