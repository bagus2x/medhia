use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    BadRequest(String),
    UnAuthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    InternalServerError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

#[derive(Serialize, Deserialize)]
pub struct PageRequest {
    pub cursor: Option<i64>,
    pub size: Option<i32>,
}

impl PageRequest {
    pub fn size(&self) -> i32 {
        if let Some(size) = self.size {
            size
        } else {
            10
        }
    }

    pub fn cursor(&self) -> i64 {
        if let Some(cursor) = self.cursor {
            cursor
        } else {
            i64::MAX
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PageResponse<T> {
    pub data: Vec<T>,
    pub next_cursor: Option<i64>,
    pub size: i32,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub status: u16,
    pub message: String,
}
