use crate::chat::conversation::model::ConversationResponse;
use crate::chat::conversation::repo::read::ConversationReadRepo;
use crate::common::model::{Error, PageRequest, PageResponse};
use std::future::Future;
use std::sync::Arc;

pub trait ConversationReadService {
    fn find_by_id(
        &self,
        conversation_id: i64,
    ) -> impl Future<Output = Result<ConversationResponse, Error>> + Send;

    fn find_by_author_id(
        &self,
        author_id: i64,
        req: PageRequest,
    ) -> impl Future<Output = Result<PageResponse<ConversationResponse>, Error>> + Send;
}

pub struct ConversationReadServiceImpl<R>
where
    R: ConversationReadRepo + Send + Sync + 'static,
{
    pub conversation_read_repo: Arc<R>,
}

impl<R> ConversationReadServiceImpl<R>
where
    R: ConversationReadRepo + Send + Sync + 'static,
{
    pub fn new(conversation_read_repo: Arc<R>) -> Self {
        Self {
            conversation_read_repo,
        }
    }
}

impl<R> ConversationReadService for ConversationReadServiceImpl<R>
where
    R: ConversationReadRepo + Send + Sync + 'static,
{
    async fn find_by_id(&self, conversation_id: i64) -> Result<ConversationResponse, Error> {
        self.conversation_read_repo
            .find_by_id(conversation_id)
            .await?
            .map(ConversationResponse::from)
            .ok_or_else(|| Error::NotFound("Conversation not found".to_string()))
    }

    async fn find_by_author_id(
        &self,
        author_id: i64,
        req: PageRequest,
    ) -> Result<PageResponse<ConversationResponse>, Error> {
        let conversations = self
            .conversation_read_repo
            .find_by_author_id(author_id, req)
            .await?;

        Ok(PageResponse {
            data: conversations
                .data
                .into_iter()
                .map(ConversationResponse::from)
                .collect(),
            next_cursor: conversations.next_cursor,
            size: conversations.size,
        })
    }
}
