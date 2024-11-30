use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claim {
    pub(crate) sub: String,
    pub(crate) exp: i64,
    pub(crate) iat: i64,
    pub(crate) username: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SignUpRequest {
    #[validate(
        email(message = "Invalid email format. Please provide a valid email address."),
        length(min = 1, max = 64, message = "Email length must be between 1 and 64 characters.")
    )]
    pub email: String,

    #[validate(length(min = 1, max = 10, message = "Username length must be between 1 and 10 characters."))]
    pub username: String,

    #[validate(length(min = 1, max = 64, message = "Name length must be between 1 and 10 characters."))]
    pub name: String,

    #[validate(length(min = 6, max = 16, message = "Password length must be between 6 and 16 characters."))]
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SignInRequest {
    pub username: String,
    pub password: String,
}
