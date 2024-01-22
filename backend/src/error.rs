use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use rusqlite::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Status code {0}")]
    Status(StatusCode),
    #[error("DB error {0}")]
    DBError(#[from] Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Status(code) => code.into_response(),
            AppError::DBError(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
            }
        }
    }
}
