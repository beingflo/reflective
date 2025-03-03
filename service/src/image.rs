use std::io::Cursor;

use crate::{
    auth::AuthenticatedUser,
    error::AppError,
    utils::{compress_image, format_filename, get_file_id},
};
use axum::{
    Json,
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::Redirect,
};
use futures::join;
use image::{GenericImageView, ImageReader};
use reqwest::Client;
use rusqlite::{Connection, params};
use serde::Deserialize;
use tokio::sync::MutexGuard;
use tracing::{error, info, warn};

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
        let filename = field.name().map(|n| n.into()).unwrap_or(get_file_id());

        let image_id: u64;

        info!(message = "uploading image", filename);

        let image_data = field.bytes().await.unwrap();

        let image = ImageReader::new(Cursor::new(&image_data))
            .with_guessed_format()
            .unwrap();

        {
            let connection = state.conn.lock().await;
            let mut stmt = connection.prepare(
                "
                    SELECT filename 
                    FROM image
                    WHERE user_id = ?1 AND filename = ?2
                ",
            )?;

            let mut result = stmt.query(params![user.id, filename])?;
            if let Some(_) = result.next()? {
                error!(message = "image already exists");
                return Err(AppError::Status(StatusCode::CONFLICT));
            };

            let mut stmt = connection
                .prepare("INSERT INTO image (filename, user_id) VALUES (?1, ?2) RETURNING id")?;

            let mut result = stmt.query(params![filename, user.id])?;
            match result.next()? {
                Some(row) => {
                    image_id = row.get(0)?;
                }
                None => {
                    error!(message = "failed to insert image");
                    return Err(AppError::Status(StatusCode::INTERNAL_SERVER_ERROR));
                }
            }
        }

        let original_image = image.decode().unwrap();

        let dimensions = original_image.dimensions();
        let medium_dimension = (
            original_image.dimensions().0 / 2,
            original_image.dimensions().1 / 2,
        );
        let small_dimension = (
            original_image.dimensions().0 / 4,
            original_image.dimensions().1 / 4,
        );

        let original_quality = 100;
        let medium_quality = 80;
        let small_quality = 80;

        let medium_image = compress_image(&original_image, medium_dimension, 6, medium_quality);
        let small_image = compress_image(&original_image, small_dimension, 6, small_quality);

        let id_original = get_file_id();
        let id_medium = get_file_id();
        let id_small = get_file_id();

        {
            let connection = state.conn.lock().await;

            connection.execute(
                "INSERT INTO variant (filename, width, height, quality, image_id) VALUES (?1, ?2, ?3, ?4, ?5)",
                (&id_original, dimensions.0, dimensions.1, original_quality, image_id),
            )?;
            connection.execute(
                "INSERT INTO variant (filename, width, height, quality, image_id) VALUES (?1, ?2, ?3, ?4, ?5)",
                (&id_medium, medium_dimension.0, medium_dimension.1, medium_quality, image_id),
            )?;
            connection.execute(
                "INSERT INTO variant (filename, width, height, quality, image_id) VALUES (?1, ?2, ?3, ?4, ?5)",
                (&id_small, small_dimension.0, small_dimension.1, small_quality, image_id),
            )?;
        }

        let [original_url, medium_url, small_url] = {
            let bucket = state.bucket.lock().await;

            let original = bucket.presign_put(&id_original, UPLOAD_LINK_TIMEOUT_SEC, None)?;
            let medium = bucket.presign_put(&id_medium, UPLOAD_LINK_TIMEOUT_SEC, None)?;
            let small = bucket.presign_put(&id_small, UPLOAD_LINK_TIMEOUT_SEC, None)?;

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
        let original_fut = client.put(original_url).body(image_data).send();
        let medium_fut = client.put(medium_url).body(medium_image).send();
        let small_fut = client.put(small_url).body(small_image).send();

        let (original_res, medium_res, small_res) = join!(original_fut, medium_fut, small_fut);

        if let Err(e) = original_res {
            error!(message = "failed to upload original image", error = ?e);
            return Err(AppError::Status(StatusCode::INTERNAL_SERVER_ERROR));
        }

        if let Err(e) = medium_res {
            error!(message = "failed to upload medium image", error = ?e);
            return Err(AppError::Status(StatusCode::INTERNAL_SERVER_ERROR));
        }

        if let Err(e) = small_res {
            error!(message = "failed to upload small image", error = ?e);
            return Err(AppError::Status(StatusCode::INTERNAL_SERVER_ERROR));
        }
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
