use std::io::Cursor;

use crate::{
    auth::AuthenticatedUser,
    error::AppError,
    utils::{compress_image, format_filename, get_file_name},
};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::Redirect,
    Json,
};
use image::{GenericImageView, ImageReader};
use reqwest::Client;
use rusqlite::Connection;
use serde::Deserialize;
use tokio::sync::MutexGuard;
use tracing::{info, warn};

use crate::AppState;

const UPLOAD_LINK_TIMEOUT_SEC: u32 = 600;

#[tracing::instrument(skip_all, fields(
    username = %user.username,
))]
pub async fn upload_image(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<StatusCode, AppError> {
    if let Some(field) = multipart.next_field().await.unwrap() {
        let filename = get_file_name();
        let original_name = format_filename(&filename, "original");
        let medium_name = format_filename(&filename, "medium");
        let small_name = format_filename(&filename, "small");

        {
            let connection = state.conn.lock().await;

            connection.execute(
                "INSERT INTO image (filename, user_id) VALUES (?1, ?2)",
                (&filename, &user.id),
            )?;

            drop(connection);
        }

        let image_data = field.bytes().await.unwrap();

        let image = ImageReader::new(Cursor::new(&image_data))
            .with_guessed_format()
            .unwrap();

        let original_image = image.decode().unwrap();
        let medium_dimension = std::cmp::min(original_image.dimensions().0 / 2, 2000);
        let medium_image = compress_image(&original_image, medium_dimension, 6, 80);
        let small_dimension = std::cmp::min(original_image.dimensions().0 / 4, 1000);
        let small_image = compress_image(&original_image, small_dimension, 6, 80);

        let [original_url, medium_url, small_url] = {
            let bucket = state.bucket.lock().await;

            let original = bucket.presign_put(&original_name, UPLOAD_LINK_TIMEOUT_SEC, None)?;
            let medium = bucket.presign_put(&medium_name, UPLOAD_LINK_TIMEOUT_SEC, None)?;
            let small = bucket.presign_put(&small_name, UPLOAD_LINK_TIMEOUT_SEC, None)?;

            drop(bucket);

            [original, medium, small]
        };

        info!(
            filename,
            filesize_original = image_data.len(),
            filesize_medium = medium_image.len(),
            filesize_small = small_image.len(),
        );

        let client = Client::new();
        client
            .put(original_url)
            .body(image_data)
            .send()
            .await
            .unwrap();
        client
            .put(medium_url)
            .body(medium_image)
            .send()
            .await
            .unwrap();
        client
            .put(small_url)
            .body(small_image)
            .send()
            .await
            .unwrap();
    }

    Ok(StatusCode::OK)
}

#[tracing::instrument(skip_all, fields(
    username = %user.username,
))]
pub async fn get_images(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<String>>), AppError> {
    let connection = state.conn.lock().await;

    let mut stmt = connection.prepare(
        "
            SELECT filename 
            FROM image
            WHERE user_id = ?1
        ",
    )?;

    let files = stmt.query_map([user.id], |row| Ok(row.get(0)?))?;

    let files = files.collect::<Result<Vec<_>, _>>()?;

    info!(message = "load image list", number_of_files = files.len());

    Ok((StatusCode::OK, Json(files)))
}

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    quality: String,
}

#[tracing::instrument(skip_all, fields(
    username = %user.username,
    id = %id,
    quality = %params.quality
))]
pub async fn get_image(
    user: AuthenticatedUser,
    Path(id): Path<String>,
    params: Query<QueryParams>,
    State(state): State<AppState>,
) -> Result<Redirect, AppError> {
    let connection = state.conn.lock().await;

    info!(message = "get image");

    check_image_exists(connection, &user.id.to_string(), &id).await?;

    let bucket = state.bucket.lock().await;

    let name = format_filename(&id, &params.quality);
    let url = bucket.presign_get(format!("/{}", name), UPLOAD_LINK_TIMEOUT_SEC, None)?;

    return Ok(Redirect::temporary(&url));
}

#[tracing::instrument(skip_all, fields(
    user_id = %user_id,
    image_id = %image_id,
))]
async fn check_image_exists(
    connection: MutexGuard<'_, Connection>,
    user_id: &str,
    image_id: &str,
) -> Result<(), AppError> {
    let mut stmt = connection.prepare(
        "
            SELECT filename
            FROM image
            WHERE user_id = ?1 AND filename = ?2
        ",
    )?;

    let mut files = stmt.query_map([user_id.to_string(), image_id.to_string()], |row| {
        Ok(row.get(0)?)
    })?;

    let file: Option<Result<String, _>> = files.next();

    if let Some(_) = file {
        return Ok(());
    }

    warn!(message = "image doesn't exist");

    return Err(AppError::Status(StatusCode::NOT_FOUND));
}
