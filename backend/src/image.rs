use crate::{auth::AuthenticatedUser, error::AppError};
use axum::{extract::State, http::StatusCode, Json};
use s3::{creds::Credentials, Bucket};
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

    let bucket_name = "reflective-test";
    let region = s3::Region::Custom {
        region: "XXX".to_string(),
        endpoint: "XXX".to_string(),
    };
    let credentials = Credentials::new(Some("XXX"), Some("XXX"), None, None, None).unwrap();

    let bucket = Bucket::new(bucket_name, region, credentials).unwrap();

    let url_put = bucket.presign_put("/test.file", 600, None).unwrap();

    Ok(StatusCode::OK)
}
