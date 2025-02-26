use crate::chat::participant::model::Participant;
use crate::common::model::Error;
use axum::async_trait;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tracing::info;
use crate::common::database::TRANSACTION;

#[async_trait]
pub trait ParticipantWriteRepo {
    async fn create(&self, participant: Participant) -> Result<Participant, Error>;

    async fn update_roles(&self, participant_id: i64, roles: &str) -> Result<Participant, Error>;

    async fn delete(&self, participant_id: i64) -> Result<(), Error>;
}

pub struct ParticipantWriteRepoPg {
    pool: Arc<Pool<Postgres>>,
}

impl ParticipantWriteRepoPg {
    pub fn new(pool: Arc<Pool<Postgres>>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ParticipantWriteRepo for ParticipantWriteRepoPg {
    async fn create(&self, participant: Participant) -> Result<Participant, Error> {
        let query = r#"
            INSERT INTO "conversation_participant" (
                conversation_id, user_id, joined_at, roles, created_at
            ) VALUES (
                $1, $2, $3, $4, $5
            )
            RETURNING id, conversation_id, user_id, joined_at, roles, deleted_at, created_at
        "#;


        if let Some(mut tx) = TRANSACTION.with(|cell| cell.borrow_mut().take()) {
            info!("p using tx");
            sqlx::query_as::<_, Participant>(query)
                .bind(participant.conversation_id)
                .bind(participant.user_id)
                .bind(participant.joined_at)
                .bind(&participant.roles)
                .bind(participant.created_at)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| Error::InternalServerError(e.to_string()))
        }else {
            info!("p not using tx");
            sqlx::query_as::<_, Participant>(query)
                .bind(participant.conversation_id)
                .bind(participant.user_id)
                .bind(participant.joined_at)
                .bind(&participant.roles)
                .bind(participant.created_at)
                .fetch_one(&*self.pool)
                .await
                .map_err(|e| Error::InternalServerError(e.to_string()))
        }
    }

    async fn update_roles(&self, id: i64, roles: &str) -> Result<Participant, Error> {
        let query = r#"
            UPDATE 
                "conversation_participant"
            SET 
                roles = $1, 
                deleted_at = NULL
            WHERE 
                id = $2
            RETURNING 
                id, conversation_id, user_id, joined_at, roles, deleted_at, created_at
        "#;

        sqlx::query_as::<_, Participant>(query)
            .bind(roles)
            .bind(id)
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }

    async fn delete(&self, participant_id: i64) -> Result<(), Error> {
        let query = r#"
            UPDATE 
                "conversation_participant"
            SET 
                deleted_at = NOW()
            WHERE 
                id = $1
        "#;

        sqlx::query(query)
            .bind(participant_id)
            .execute(&*self.pool)
            .await
            .map(|_| ())
            .map_err(|e| Error::InternalServerError(e.to_string()))
    }
}
