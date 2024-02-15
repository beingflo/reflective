use crate::{auth::AuthenticatedUser, error::AppError};
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Serialize, Deserialize)]
pub struct UploadRequest {
    number: u32,
}

pub async fn upload_images(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(data): Json<UploadRequest>,
) -> Result<StatusCode, AppError> {
    if data.number > 20 {
        return Err(AppError::Status(StatusCode::BAD_REQUEST));
    }
    // Create data.number * 3 presigned PUT URLs
    // Insert them into db with NULL metadata

    Ok(StatusCode::OK)
}
