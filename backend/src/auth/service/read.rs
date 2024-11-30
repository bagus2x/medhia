use crate::auth::model::Claim;
use crate::common::config::Config;
use crate::common::model::Error;
use axum::async_trait;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use std::sync::Arc;

#[async_trait]
pub trait AuthReadService {
    async fn verify_token(&self, token: &str) -> Result<Claim, Error>;
}

#[derive(Clone)]
pub struct AuthReadServiceImpl {
    config: Arc<Config>,
}

impl AuthReadServiceImpl {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl AuthReadService for AuthReadServiceImpl {
    async fn verify_token(&self, token: &str) -> Result<Claim, Error> {
        let data = jsonwebtoken::decode::<Claim>(
            &token,
            &DecodingKey::from_secret(self.config.access_token_key_secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|error| Error::UnAuthorized(error.to_string()))?;

        Ok(data.claims)
    }
}
