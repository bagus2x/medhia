use crate::chat::conversation::model::{
    Conversation, ConversationResponse, ConversationType, CreateConversationRequest,
    DeleteConversationRequest,
};
use crate::chat::conversation::repo::read::ConversationReadRepo;
use crate::chat::conversation::repo::write::ConversationWriteRepo;
use crate::chat::participant::model::Participant;
use crate::chat::participant::repo::write::ParticipantWriteRepo;
use crate::common::database::UnitOfWork;
use crate::common::model::Error;
use std::future::Future;
use std::sync::Arc;
use validator::Validate;

pub trait ConversationWriteService {
    fn create(
        &self,
        req: CreateConversationRequest,
    ) -> impl Future<Output = Result<ConversationResponse, Error>> + Send;

    fn delete(
        &self,
        req: DeleteConversationRequest,
    ) -> impl Future<Output = Result<ConversationResponse, Error>> + Send;
}

pub struct ConversationWriteServiceImpl<T1, T2, T3, U>
where
    T1: ConversationWriteRepo + Send + Sync + 'static,
    T2: ConversationReadRepo + Send + Sync + 'static,
    T3: ParticipantWriteRepo + Send + Sync + 'static,
    U: UnitOfWork + Send + Sync + 'static,
{
    pub conversation_write_repo: Arc<T1>,
    pub conversation_read_repo: Arc<T2>,
    pub participant_write_repo: Arc<T3>,
    pub unit_of_work: Arc<U>,
}

impl<T1, T2, T3, T4> ConversationWriteServiceImpl<T1, T2, T3, T4>
where
    T1: ConversationWriteRepo + Send + Sync + 'static,
    T2: ConversationReadRepo + Send + Sync + 'static,
    T3: ParticipantWriteRepo + Send + Sync + 'static,
    T4: UnitOfWork + Send + Sync + 'static,
{
    pub fn new(
        conversation_write_repo: Arc<T1>,
        conversation_read_repo: Arc<T2>,
        participant_write_repo: Arc<T3>,
        unit_of_work: Arc<T4>,
    ) -> Self {
        Self {
            conversation_read_repo,
            conversation_write_repo,
            participant_write_repo,
            unit_of_work,
        }
    }
}

impl<T1, T2, T3, T4> ConversationWriteService for ConversationWriteServiceImpl<T1, T2, T3, T4>
where
    T1: ConversationWriteRepo + Send + Sync + 'static,
    T2: ConversationReadRepo + Send + Sync + 'static,
    T3: ParticipantWriteRepo + Send + Sync + 'static,
    T4: UnitOfWork + Send + Sync + 'static,
{
    async fn create(&self, req: CreateConversationRequest) -> Result<ConversationResponse, Error> {
        self.unit_of_work
            .run(async move {
                req.validate()
                    .map_err(|errors| Error::BadRequest(errors.to_string()))?;

                // Remove duplicate participants
                let mut participants = req.participants.clone();
                participants.push(req.author_id);
                participants.dedup();

                let (private_id, r#type) = if let ConversationType::PRIVATE = req.r#type {
                    if req.participants.len() != 2 {
                        return Err(Error::BadRequest(
                            "Private chat must have 2 participants".to_string(),
                        ));
                    }

                    let private_id = create_private_id(participants[0], participants[1]);
                    let is_present = self
                        .conversation_read_repo
                        .exists_by_private_id(&private_id)
                        .await?;
                    if is_present {
                        return Err(Error::BadRequest(
                            "Private conversation already exists".to_string(),
                        ));
                    }

                    (Some(private_id), ConversationType::PRIVATE)
                } else {
                    (None, ConversationType::GROUP)
                };

                let conversation = Conversation {
                    id: 0, // Will be replaced by database
                    private_id,
                    author_id: req.author_id,
                    r#type,
                    name: req.name,
                    photo_url: req.photo_url,
                    deleted_at: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };

                let conversation = self.conversation_write_repo.create(conversation).await?;

                for user_id in req.participants {
                    let participant = Participant {
                        id: 0,
                        conversation_id: conversation.id,
                        user_id,
                        joined_at: chrono::Utc::now(),
                        roles: if user_id == req.author_id {
                            "ADMIN,PARTICIPANT".to_string()
                        } else {
                            "PARTICIPANT".to_string()
                        },
                        deleted_at: None,
                        created_at: chrono::Utc::now(),
                    };

                    let _ = self.participant_write_repo.create(participant).await?;
                }

                Err(Error::InternalServerError(format!("{}", conversation.id)))
            })
            .await
    }

    async fn delete(&self, req: DeleteConversationRequest) -> Result<ConversationResponse, Error> {
        let conversation = self
            .conversation_read_repo
            .find_by_id(req.conversation_id)
            .await?
            .ok_or_else(|| Error::InternalServerError("Conversation not found".to_string()))?;

        self.conversation_write_repo.delete(conversation.id).await?;

        Ok(ConversationResponse::from(conversation))
    }
}

pub fn create_private_id(user_id1: i64, user_id2: i64) -> String {
    if user_id1 < user_id2 {
        format!("{}#{}", user_id1, user_id2)
    } else {
        format!("{}#{}", user_id2, user_id1)
    }
}
