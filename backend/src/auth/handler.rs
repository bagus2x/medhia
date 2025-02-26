use crate::auth::model::{SignInRequest, SignUpRequest};
use crate::auth::service::AuthWriteService;
use crate::common::json::IntoApiResponse;
use crate::common::state::AppState;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use std::sync::Arc;

pub struct AuthHandler<W>
where
    W: AuthWriteService + Send + Sync + 'static,
{
    auth_write_service: Arc<W>,
}

impl<W> AuthHandler<W>
where
    W: AuthWriteService + Send + Sync + 'static,
{
    pub fn new(auth_write_service: Arc<W>) -> Self {
        Self { auth_write_service }
    }

    async fn sign_up(&self, Json(req): Json<SignUpRequest>) -> impl IntoResponse {
        self.auth_write_service.sign_up(req).await.into_json()
    }

    async fn sign_in(&self, Json(req): Json<SignInRequest>) -> impl IntoResponse {
        self.auth_write_service.sign_in(req).await.into_json()
    }

    pub fn create_route(handler: Arc<Self>, router: Router<AppState>) -> Router<AppState> {
        router
            .route(
                "/api/auth/sign_up",
                post({
                    let handler = Arc::clone(&handler);
                    move |req: Json<SignUpRequest>| async move { handler.sign_up(req).await }
                }),
            )
            .route(
                "/api/auth/sign_in",
                post({
                    let handler = Arc::clone(&handler);
                    move |req: Json<SignInRequest>| async move { handler.sign_in(req).await }
                }),
            )
    }
}
