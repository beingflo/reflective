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

    let config: S3Data = match user.config {
        Some(c) => c,
        None => return Err(AppError::Status(StatusCode::BAD_REQUEST)),
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
    )?;

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

        connection.execute(
            "INSERT INTO images (filename_small, filename_medium, filename_original, user_id) VALUES (?1, ?2, ?3, ?4)",
            (&small, &medium, &original, &user.id),
        )?;
    }

    Ok((StatusCode::OK, Json(files)))
}

pub async fn get_images(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<FileGroup>>), AppError> {
    let connection = state.conn.lock().await;

    let mut stmt = connection.prepare(
        "
            SELECT filename_small, filename_medium, filename_original 
            FROM images
            WHERE user_id = ?1
        ",
    )?;

    let file_groups = stmt.query_map([user.id], |row| {
        Ok(FileGroup {
            small: row.get(0)?,
            medium: row.get(1)?,
            original: row.get(2)?,
        })
    })?;

    let config: S3Data = match user.config {
        Some(c) => c,
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
    )?;

    let bucket = Bucket::new(&bucket_name, region, credentials)?;

    let mut links = Vec::new();
    for group in file_groups {
        if let Ok(group) = group {
            let url_small =
                bucket.presign_get(format!("/{}", group.small), UPLOAD_LINK_TIMEOUT_SEC, None)?;
            let url_medium =
                bucket.presign_get(format!("/{}", group.medium), UPLOAD_LINK_TIMEOUT_SEC, None)?;
            let url_original = bucket.presign_get(
                format!("/{}", group.original),
                UPLOAD_LINK_TIMEOUT_SEC,
                None,
            )?;
            links.push(FileGroup {
                small: url_small,
                medium: url_medium,
                original: url_original,
            });
        } else {
            return Err(AppError::Status(StatusCode::INTERNAL_SERVER_ERROR));
        }
    }

    Ok((StatusCode::OK, Json(links)))
}
