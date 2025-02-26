use crate::auth::service::AuthReadService;
use crate::common::json::IntoApiResponse;
use crate::common::model::{ApiResponse, Error};
use crate::common::state::AppState;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::{async_trait, Json};

pub struct Auth {
    pub user_id: i64,
}

#[async_trait]
impl FromRequestParts<AppState> for Auth {
    type Rejection = (StatusCode, Json<ApiResponse<Option<()>>>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("Authorization")
            .ok_or_else(|| Error::UnAuthorized("Auth header not found".to_string()).into_json())?;
        let token = header.to_str().map_err(|_| {
            Error::UnAuthorized("Cannot extract auth header".to_string()).into_json()
        })?;
        if token.len() < 7 {
            return Err(
                Error::UnAuthorized("Invalid authorization bearer token".to_string()).into_json(),
            );
        }

        let claim = state
            .auth_read_service
            .verify_token(&token[7..])
            .await
            .map_err(|error| error.into_json())?;

        Ok(Auth {
            user_id: claim.sub.parse().unwrap(),
        })
    }
}
