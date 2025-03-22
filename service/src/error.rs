use std::io;

use axum::{
    extract::multipart::MultipartError,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use rusqlite::Error;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Status code {0}")]
    Status(StatusCode),
    #[error("DB error {0}")]
    DBError(#[from] Error),
    #[error("Serde error {0}")]
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
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!(message = "app error", error = %self);

        match self {
            AppError::Status(code) => code.into_response(),
            AppError::DBError(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
            }
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
        }
    }
}
