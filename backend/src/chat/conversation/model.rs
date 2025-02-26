use crate::common::model::Error;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use std::str::FromStr;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Conversation {
    pub id: i64,
    pub private_id: Option<String>,
    pub author_id: i64,
    pub r#type: ConversationType,
    pub name: Option<String>,
    pub photo_url: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Type)]
#[sqlx(type_name = "conversation_type")]
#[sqlx(rename_all = "UPPERCASE")]
pub enum ConversationType {
    PRIVATE,
    GROUP,
}

impl FromStr for ConversationType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "PRIVATE" => Ok(ConversationType::PRIVATE),
            "GROUP" => Ok(ConversationType::GROUP),
            _ => Err(Error::InternalServerError(format!(
                "unknown conversation type: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ConversationResponse {
    pub id: i64,
    pub private_id: Option<String>,
    pub author_id: i64,
    pub r#type: ConversationType,
    pub name: Option<String>,
    pub photo_url: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ConversationResponse {
    pub fn from(conversation: Conversation) -> Self {
        Self {
            id: conversation.id,
            private_id: conversation.private_id,
            author_id: conversation.author_id,
            r#type: conversation.r#type,
            name: conversation.name,
            photo_url: conversation.photo_url,
            deleted_at: conversation.deleted_at,
            created_at: conversation.created_at,
            updated_at: conversation.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateConversationRequest {
    #[validate(range(min = 1))]
    pub author_id: i64,
    pub r#type: ConversationType,
    #[validate(length(min = 3, max = 50))]
    pub name: Option<String>,
    #[validate(url)]
    pub photo_url: Option<String>,
    pub participants: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateConversationRequest {
    pub r#type: ConversationType,
    pub name: Option<String>,
    pub photo_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteConversationRequest {
    pub author_id: i64,
    pub conversation_id: i64,
}
