use crate::auth::extractor::Auth;
use crate::common::json::IntoApiResponse;
use crate::common::model::PageRequest;
use crate::common::state::AppState;
use crate::user::model::UpdateUserRequest;
use crate::user::service::UserReadService;
use crate::user::service::UserWriteService;
use axum::extract::{Path, Query};
use axum::response::IntoResponse;
use axum::routing::{delete, get, patch};
use axum::{Json, Router};
use std::sync::Arc;

#[derive(Clone)]
pub struct UserHandler<W, R>
where
    W: UserWriteService + Send + Sync + 'static,
    R: UserReadService + Send + Sync + 'static,
{
    user_write_service: Arc<W>,
    user_read_service: Arc<R>,
}

impl<W, R> UserHandler<W, R>
where
    W: UserWriteService + Send + Sync + 'static,
    R: UserReadService + Send + Sync + 'static,
{
    pub fn new(user_write_service: Arc<W>, user_read_service: Arc<R>) -> Self {
        Self {
            user_write_service,
            user_read_service,
        }
    }

    async fn find_by_id(&self, user_id: i64) -> impl IntoResponse {
        self.user_read_service.find_by_id(user_id).await.into_json()
    }

    async fn find_all(&self, req: PageRequest) -> impl IntoResponse {
        self.user_read_service.find_all(req).await.into_json()
    }

    async fn update(&self, user_id: i64, req: UpdateUserRequest) -> impl IntoResponse {
        self.user_write_service.update(user_id, req).await.into_json()
    }

    async fn delete(&self, user_id: i64) -> impl IntoResponse {
        self.user_write_service.delete(user_id).await.into_json()
    }

    pub fn create_route(handler: Arc<Self>, router: Router<AppState>) -> Router<AppState> {
        router
            .route(
                "/api/users",
                get({
                    let handler = Arc::clone(&handler);
                    move |Query(req): Query<PageRequest>| async move { handler.find_all(req).await }
                }),
            )
            .route(
                "/api/user",
                get({
                    let handler = Arc::clone(&handler);
                    |auth: Auth| async move { handler.find_by_id(auth.user_id).await }
                }),
            )
            .route(
                "/api/user/:user_id",
                get({
                    let handler = Arc::clone(&handler);
                    |Path(user_id): Path<i64>| async move { handler.find_by_id(user_id).await }
                }),
            )
            .route(
                "/api/user",
                patch({
                    let handler = Arc::clone(&handler);
                     |auth: Auth, Json(req): Json<UpdateUserRequest>| async move {
                        handler.update(auth.user_id, req).await
                    }
                }),
            )
            .route(
                "/api/user",
                delete({
                    let handler = Arc::clone(&handler);
                    |auth: Auth| async move {
                        handler.delete(auth.user_id).await
                    }
                }),
            )
    }
}
