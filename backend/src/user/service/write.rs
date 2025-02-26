use crate::common::model::Error;
use crate::user::model::{UpdateUserRequest, UserResponse};
use crate::user::repo::UserReadRepo;
use crate::user::repo::UserWriteRepo;
use std::future::Future;
use std::sync::Arc;

pub trait UserWriteService: Send + Sync {
    fn update(
        &self,
        user_id: i64,
        request: UpdateUserRequest,
    ) -> impl Future<Output = Result<UserResponse, Error>> + Send;

    fn delete(&self, user_id: i64) -> impl Future<Output = Result<UserResponse, Error>> + Send;
}

pub struct UserWriteServiceImpl<W, R>
where
    W: UserWriteRepo + Send + Sync + 'static,
    R: UserReadRepo + Send + Sync + 'static,
{
    user_write_repo: Arc<W>,
    user_read_repo: Arc<R>,
}

impl<W: UserWriteRepo + Send + Sync, R: UserReadRepo + Send + Sync> UserWriteServiceImpl<W, R> {
    pub fn new(user_write_repo: Arc<W>, user_read_repo: Arc<R>) -> Self {
        UserWriteServiceImpl {
            user_write_repo,
            user_read_repo,
        }
    }
}

impl<W, R> UserWriteService for UserWriteServiceImpl<W, R>
where
    W: UserWriteRepo + Send + Sync,
    R: UserReadRepo + Send + Sync,
{
    async fn update(&self, user_id: i64, req: UpdateUserRequest) -> Result<UserResponse, Error> {
        let user = self
            .user_read_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("User with id {} not found", user_id)))?;

        let password = req
            .password
            .map(|password| {
                bcrypt::hash(password, bcrypt::DEFAULT_COST)
                    .map_err(|e| Error::InternalServerError(e.to_string()))
            })
            .unwrap_or(Ok(user.password))?;

        if let Some(username) = &req.username {
            let is_present = self.user_read_repo.exists_by_username(username).await?;

            if username != &user.username && is_present {
                return Err(Error::Conflict("Username already exists".to_string()));
            }
        }

        if let Some(email) = &req.email {
            let is_present = self.user_read_repo.exists_by_email(email).await?;
            if email != &user.email && is_present {
                return Err(Error::Conflict("Email already exists".to_string()));
            }
        }

        let updated_request = UpdateUserRequest {
            name: req.name.or(Some(user.name)),
            password: Some(password),
            photo_url: req.photo_url.or(user.photo_url),
            username: req.username.or(Some(user.username)),
            email: req.email.or(Some(user.email)),
        };

        let updated_user = self
            .user_write_repo
            .update(user_id, updated_request)
            .await?;

        Ok(UserResponse::from(updated_user))
    }

    async fn delete(&self, user_id: i64) -> Result<UserResponse, Error> {
        let user = self
            .user_read_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("User with id {} not found", user_id)))?;

        let user = self.user_write_repo.delete(user.id).await?;

        Ok(UserResponse::from(user))
    }
}
