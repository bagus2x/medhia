use crate::chat::conversation::model::CreateConversationRequest;
use crate::chat::conversation::service::read::ConversationReadService;
use crate::chat::conversation::service::write::ConversationWriteService;
use crate::common::json::IntoApiResponse;
use crate::common::state::AppState;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use std::sync::Arc;

pub struct ConversationHandler<T1, T2>
where
    T1: ConversationWriteService + Send + Sync + 'static,
    T2: ConversationReadService + Send + Sync + 'static,
{
    conversation_write_service: Arc<T1>,
    conversation_read_service: Arc<T2>,
}

impl<T1, T2> ConversationHandler<T1, T2>
where
    T1: ConversationWriteService + Send + Sync + 'static,
    T2: ConversationReadService + Send + Sync + 'static,
{
    pub fn new(conversation_write_service: Arc<T1>, conversation_read_service: Arc<T2>) -> Self {
        Self {
            conversation_write_service,
            conversation_read_service,
        }
    }

    async fn create(&self, req: CreateConversationRequest) -> impl IntoResponse {
        self.conversation_write_service
            .create(req)
            .await
            .into_json()
    }

    pub fn create_route(handler: Arc<Self>, router: Router<AppState>) -> Router<AppState> {
        router.route(
            "/api/conversation",
            post({
                let handler = Arc::clone(&handler);
                move |Json(req): Json<CreateConversationRequest>| async move {
                    handler.create(req).await
                }
            }),
        )
    }
}
