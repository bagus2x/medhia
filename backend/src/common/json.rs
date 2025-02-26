use crate::common::model::ApiResponse;
use crate::common::model::Error;
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

pub trait IntoApiResponse<T>
where
    T: Serialize,
{
    fn into_json_with_code(
        self,
        code: u16,
        message: String,
    ) -> (StatusCode, Json<ApiResponse<Option<T>>>);

    fn into_json(self) -> (StatusCode, Json<ApiResponse<Option<T>>>);
}

impl<T> IntoApiResponse<T> for Result<T, Error>
where
    T: Serialize,
{
    fn into_json_with_code(
        self,
        code: u16,
        message: String,
    ) -> (StatusCode, Json<ApiResponse<Option<T>>>) {
        let result = match self {
            Ok(data) => extract_success(code, data, message),
            Err(error) => extract_error(error),
        };

        result
    }

    fn into_json(self) -> (StatusCode, Json<ApiResponse<Option<T>>>) {
        self.into_json_with_code(200, "Success!".to_string())
    }
}

impl<T> IntoApiResponse<T> for Error
where
    T: Serialize,
{
    fn into_json_with_code(
        self,
        _code: u16,
        _message: String,
    ) -> (StatusCode, Json<ApiResponse<Option<T>>>) {
        extract_error::<T>(self)
    }

    fn into_json(self) -> (StatusCode, Json<ApiResponse<Option<T>>>) {
        self.into_json_with_code(400, "".to_string())
    }
}

fn extract_success<T>(
    status: u16,
    data: T,
    message: String,
) -> (StatusCode, Json<ApiResponse<Option<T>>>)
where
    T: Serialize,
{
    (
        StatusCode::from_u16(status).unwrap(),
        Json(ApiResponse {
            data: Some(data),
            status,
            message,
        }),
    )
}

fn extract_error<T>(error: Error) -> (StatusCode, Json<ApiResponse<Option<T>>>)
where
    T: Serialize,
{
    let (status, message) = match error {
        Error::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
        Error::UnAuthorized(message) => (StatusCode::UNAUTHORIZED, message),
        Error::Forbidden(message) => (StatusCode::FORBIDDEN, message),
        Error::NotFound(message) => (StatusCode::NOT_FOUND, message),
        Error::Conflict(message) => (StatusCode::CONFLICT, message),
        Error::InternalServerError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
    };

    (
        status,
        Json(ApiResponse {
            data: None,
            status: status.as_u16(),
            message,
        }),
    )
}
