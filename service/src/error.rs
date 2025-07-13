use std::io;

use axum::{
    extract::multipart::MultipartError,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Status code {0}")]
    Status(StatusCode),
    #[error("Text error {0}: {1}")]
    Text(StatusCode, String),
    #[error("Serde error {0}")]
    DBError(#[from] sqlx::Error),
    #[error("DB error {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("S3 error {0}")]
    S3Error(#[from] s3::error::S3Error),
    #[error("Credentials error {0}")]
    CredentialsError(#[from] s3::creds::error::CredentialsError),
    #[error("Image error {0}")]
    ImageError(#[from] image::ImageError),
    #[error("IO error {0}")]
    IOError(#[from] io::Error),
    #[error("Exif error {0}")]
    ExifError(#[from] exif::Error),
    #[error("Multipart error {0}")]
    MultipartError(#[from] MultipartError),
    #[error("DateParseError error {0}")]
    DateParseError(#[from] jiff::Error),
    #[error("Uuid parse error {0}")]
    UuidParseError(#[from] uuid::Error),
    #[error("Reqwest error {0}")]
    ReqwestError(#[from] reqwest::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!(message = "Error", error = %self);

        match self {
            AppError::Status(code) => code.into_response(),
            AppError::Text(status, description) => (status, description).into_response(),
            AppError::DBError(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            AppError::SerdeError(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
            }
            AppError::S3Error(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
            }
            AppError::CredentialsError(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
            }
            AppError::ImageError(error) => {
                (StatusCode::BAD_REQUEST, error.to_string()).into_response()
            }
            AppError::IOError(error) => {
                (StatusCode::BAD_REQUEST, error.to_string()).into_response()
            }
            AppError::ExifError(error) => {
                (StatusCode::BAD_REQUEST, error.to_string()).into_response()
            }
            AppError::MultipartError(error) => {
                (StatusCode::BAD_REQUEST, error.to_string()).into_response()
            }
            AppError::DateParseError(error) => {
                (StatusCode::BAD_REQUEST, error.to_string()).into_response()
            }
            AppError::UuidParseError(error) => {
                (StatusCode::BAD_REQUEST, error.to_string()).into_response()
            }
            AppError::ReqwestError(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
            }
        }
    }
}
