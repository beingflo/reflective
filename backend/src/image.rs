use crate::{auth::AuthenticatedUser, error::AppError, user::S3Data, utils::get_file_name};
use axum::{extract::State, http::StatusCode, Json};
use s3::{creds::Credentials, Bucket};
use serde::{Deserialize, Serialize};

use crate::AppState;

const MAX_FILES_PER_REQUEST: u32 = 32;
const UPLOAD_LINK_TIMEOUT_SEC: u32 = 600;

#[derive(Serialize, Deserialize)]
pub struct UploadRequest {
    number: u32,
}

#[derive(Serialize, Deserialize)]
pub struct FileGroup {
    small: String,
    medium: String,
    original: String,
}

pub async fn upload_images(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(data): Json<UploadRequest>,
) -> Result<(StatusCode, Json<Vec<FileGroup>>), AppError> {
    if data.number > MAX_FILES_PER_REQUEST {
        return Err(AppError::Status(StatusCode::BAD_REQUEST));
    }
    let connection = state.conn.lock().await;

    let mut stmt = connection
        .prepare(
            "
                SELECT config 
                FROM users 
                WHERE id = ?1
            ",
        )
        .unwrap();

    let mut rows = stmt.query([user.id])?;

    let config: Option<String> = match rows.next()? {
        Some(row) => row.get(0)?,
        None => return Err(AppError::Status(StatusCode::INTERNAL_SERVER_ERROR)),
    };

    let config: S3Data = match config {
        Some(c) => serde_json::from_str(&c)?,
        None => return Err(AppError::Status(StatusCode::NOT_FOUND)),
    };

    let bucket_name = config.bucket;
    let region = s3::Region::Custom {
        region: config.region,
        endpoint: config.endpoint,
    };
    let credentials = Credentials::new(
        Some(&config.access_key),
        Some(&config.secret_key),
        None,
        None,
        None,
    )
    .unwrap();

    let bucket = Bucket::new(&bucket_name, region, credentials)?;

    // Create data.number * 3 presigned PUT URLs
    // Insert them into db with NULL metadata
    let mut files = Vec::new();
    for _ in 0..data.number {
        let small = get_file_name();
        let medium = get_file_name();
        let original = get_file_name();

        let url_small = bucket.presign_put(format!("/{}", small), UPLOAD_LINK_TIMEOUT_SEC, None)?;
        let url_medium =
            bucket.presign_put(format!("/{}", medium), UPLOAD_LINK_TIMEOUT_SEC, None)?;
        let url_original =
            bucket.presign_put(format!("/{}", original), UPLOAD_LINK_TIMEOUT_SEC, None)?;

        files.push(FileGroup {
            small: url_small,
            medium: url_medium,
            original: url_original,
        });
    }

    Ok((StatusCode::OK, Json(files)))
}
