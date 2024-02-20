use crate::{
    auth::AuthenticatedUser,
    error::AppError,
    user::S3Data,
    utils::{format_filename, get_bucket, get_file_name},
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Redirect,
    Json,
};
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

    let bucket = get_bucket(config)?;

    let mut files = Vec::new();
    for _ in 0..data.number {
        let filename = get_file_name();
        let small = format_filename(&filename, "small");
        let medium = format_filename(&filename, "medium");
        let original = format_filename(&filename, "original");

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
            "INSERT INTO images (filename, quality, user_id) VALUES (?1, ?2, ?3)",
            (&filename, "small", &user.id),
        )?;
        connection.execute(
            "INSERT INTO images (filename, quality, user_id) VALUES (?1, ?2, ?3)",
            (&filename, "medium", &user.id),
        )?;
        connection.execute(
            "INSERT INTO images (filename, quality, user_id) VALUES (?1, ?2, ?3)",
            (&filename, "original", &user.id),
        )?;
    }

    Ok((StatusCode::OK, Json(files)))
}

pub async fn get_images(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<String>>), AppError> {
    let connection = state.conn.lock().await;

    let mut stmt = connection.prepare(
        "
            SELECT filename 
            FROM images
            WHERE user_id = ?1 AND quality = 'small'
        ",
    )?;

    let files = stmt.query_map([user.id], |row| Ok(row.get(0)?))?;

    let files = files.collect::<Result<_, _>>()?;

    Ok((StatusCode::OK, Json(files)))
}

#[derive(Deserialize)]
pub struct QueryParams {
    quality: String,
}

pub async fn get_image(
    user: AuthenticatedUser,
    Path(id): Path<String>,
    params: Query<QueryParams>,
    State(state): State<AppState>,
) -> Result<Redirect, AppError> {
    let connection = state.conn.lock().await;

    let mut stmt = connection.prepare(
        "
            SELECT filename
            FROM images
            WHERE user_id = ?1 AND quality = ?2 AND filename = ?3
        ",
    )?;

    let mut files = stmt.query_map([&user.id.to_string(), &params.quality, &id], |row| {
        Ok(row.get(0)?)
    })?;

    let config: S3Data = match user.config {
        Some(c) => c,
        None => return Err(AppError::Status(StatusCode::NOT_FOUND)),
    };

    let bucket = get_bucket(config)?;

    let file = files.next();

    if let Some(file) = file {
        let file: String = file?;
        let name = format_filename(&file, &params.quality);
        let url = bucket.presign_get(format!("/{}", name), UPLOAD_LINK_TIMEOUT_SEC, None)?;
        return Ok(Redirect::temporary(&url));
    }

    return Err(AppError::Status(StatusCode::NOT_FOUND));
}
