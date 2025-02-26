use crate::auth::model::Claim;
use crate::common::config::Config;
use crate::common::model::Error;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use std::future::Future;
use std::sync::Arc;

pub trait AuthReadService {
    fn verify_token(&self, token: &str) -> impl Future<Output = Result<Claim, Error>> + Send;
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
