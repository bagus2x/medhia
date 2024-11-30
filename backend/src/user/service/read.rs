use crate::common::model::{Error, PageRequest, PageResponse};
use crate::user::model::UserResponse;
use crate::user::repo::UserReadRepo;
use axum::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait UserReadService {
    async fn find_by_id(&self, user_id: i64) -> Result<UserResponse, Error>;

    async fn find_all(&self, req: PageRequest) -> Result<PageResponse<UserResponse>, Error>;
}

pub struct UserReadServiceImpl<R>
where
    R: UserReadRepo + Send + Sync + 'static,
{
    user_read_repo: Arc<R>,
}

impl<R> UserReadServiceImpl<R>
where
    R: UserReadRepo + Send + Sync + 'static,
{
    pub fn new(user_read_repo: Arc<R>) -> UserReadServiceImpl<R> {
        UserReadServiceImpl { user_read_repo }
    }
}

#[async_trait]
impl<R> UserReadService for UserReadServiceImpl<R>
where
    R: UserReadRepo + Send + Sync + 'static,
{
    async fn find_by_id(&self, user_id: i64) -> Result<UserResponse, Error> {
        self.user_read_repo
            .find_by_id(user_id)
            .await?
            .map(UserResponse::from)
            .ok_or_else(|| Error::NotFound(format!("User with id {} not found", user_id)))
    }

    async fn find_all(&self, req: PageRequest) -> Result<PageResponse<UserResponse>, Error> {
        let users = self.user_read_repo.find_all(req).await?;

        Ok(PageResponse {
            data: users.data.into_iter().map(UserResponse::from).collect(),
            next_cursor: users.next_cursor,
            size: users.size,
        })
    }
}
