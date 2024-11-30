use crate::auth::model::{AuthResponse, Claim, SignInRequest, SignUpRequest};
use crate::common::config::Config;
use crate::common::model::Error;
use crate::user::model::CreateUserRequest;
use crate::user::repo::UserReadRepo;
use crate::user::repo::UserWriteRepo;
use axum::async_trait;
use jsonwebtoken::Algorithm;
use std::ops::Add;
use std::sync::Arc;
use validator::Validate;

#[async_trait]
pub trait AuthWriteService {
    async fn sign_up(&self, req: SignUpRequest) -> Result<AuthResponse, Error>;

    async fn sign_in(&self, req: SignInRequest) -> Result<AuthResponse, Error>;
}

pub struct AuthWriteServiceImpl<W, R>
where
    W: UserWriteRepo + Send + Sync + 'static,
    R: UserReadRepo + Send + Sync + 'static,
{
    user_write_repo: Arc<W>,
    user_read_repo: Arc<R>,
    config: Arc<Config>,
}

impl<W, R> AuthWriteServiceImpl<W, R>
where
    W: UserWriteRepo + Send + Sync + 'static,
    R: UserReadRepo + Send + Sync + 'static,
{
    pub fn new(user_write_repo: Arc<W>, user_read_repo: Arc<R>, config: Arc<Config>) -> Self {
        AuthWriteServiceImpl {
            user_write_repo,
            user_read_repo,
            config,
        }
    }

    fn create_token(&self, key: &[u8], user_id: i64, username: &str, exp: i64, iat: i64) -> Result<String, Error> {
        let claims = serde_json::json!(Claim {
            sub: user_id.to_string(),
            username: String::from(username),
            exp,
            iat,
        });

        jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::HS256),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(key),
        )
        .map_err(|e| Error::InternalServerError(e.to_string()))
    }
}

#[async_trait]
impl<W, R> AuthWriteService for AuthWriteServiceImpl<W, R>
where
    W: UserWriteRepo + Send + Sync,
    R: UserReadRepo + Send + Sync,
{
    async fn sign_up(&self, req: SignUpRequest) -> Result<AuthResponse, Error> {
        req.validate().map_err(|errors| Error::BadRequest(errors.to_string()))?;

        if self.user_read_repo.exists_by_email(&req.email).await? {
            return Err(Error::Conflict("Email already exists".to_string()));
        }

        if self.user_read_repo.exists_by_username(&req.username).await? {
            return Err(Error::Conflict("Username already exists".to_string()));
        }

        let password = bcrypt::hash(&req.password, 12).map_err(|e| Error::InternalServerError(e.to_string()))?;

        let req = CreateUserRequest {
            username: req.username,
            email: req.email,
            password,
            name: req.name,
            photo_url: None,
        };

        let user = self.user_write_repo.create(req).await?;

        let access_token = self.create_token(
            self.config.access_token_key_secret.as_ref(),
            user.id,
            &user.username,
            chrono::Utc::now().add(chrono::Duration::minutes(10)).timestamp(),
            chrono::Utc::now().timestamp(),
        )?;
        let refresh_token = self.create_token(
            self.config.refresh_token_key_secret.as_ref(),
            user.id,
            &user.username,
            chrono::Utc::now().add(chrono::Duration::days(7)).timestamp(),
            chrono::Utc::now().timestamp(),
        )?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            user_id: user.id,
        })
    }

    async fn sign_in(&self, req: SignInRequest) -> Result<AuthResponse, Error> {
        let user = self
            .user_read_repo
            .find_by_username_or_email(&req.username)
            .await?
            .ok_or_else(|| Error::NotFound("User not found".to_string()))?;

        let does_match =
            bcrypt::verify(&req.password, &user.password).map_err(|e| Error::InternalServerError(e.to_string()))?;
        if !does_match {
            return Err(Error::BadRequest("Password does not match".to_string()));
        }

        let access_token = self.create_token(
            self.config.access_token_key_secret.as_ref(),
            user.id,
            &user.username,
            chrono::Utc::now().add(chrono::Duration::minutes(10)).timestamp(),
            chrono::Utc::now().timestamp(),
        )?;
        let refresh_token = self.create_token(
            self.config.refresh_token_key_secret.as_ref(),
            user.id,
            &user.username,
            chrono::Utc::now().add(chrono::Duration::days(7)).timestamp(),
            chrono::Utc::now().timestamp(),
        )?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            user_id: user.id,
        })
    }
}
