use std::{collections::HashMap, io::Cursor, sync::Arc};

use crate::{
    auth::AuthenticatedUser,
    error::AppError,
    utils::{compress_image, get_id, get_object_name},
};
use axum::{
    Json,
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::Redirect,
};
use futures::join;
use image::{GenericImageView, ImageDecoder, ImageReader};
use jiff::{Timestamp, fmt::strtime, tz};
use reqwest::Client;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::AppState;

const UPLOAD_LINK_TIMEOUT_SEC: u32 = 600;

#[tracing::instrument(skip_all, fields(
    username = %user.username,
))]
#[axum::debug_handler]
pub async fn upload_image(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<StatusCode, AppError> {
    let mut filename: Option<String> = None;
    let mut last_modified: Option<String> = None;
    let mut image_data: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await? {
        match field.name() {
            Some("filename") => {
                filename = Some(field.text().await?);
            }
            Some("last_modified") => {
                last_modified = Some(field.text().await?);
            }
            _ => {
                let data = field.bytes().await?;

                image_data = Some(data.to_vec());
            }
        }
    }

    if filename.is_none() || last_modified.is_none() || image_data.is_none() {
        error!(message = "missing field in multipart body");
        return Err(AppError::Status(StatusCode::BAD_REQUEST));
    }

    let Some(filename) = filename else {
        unreachable!()
    };
    let Some(last_modified) = last_modified else {
        unreachable!()
    };
    let Some(image_data) = image_data else {
        unreachable!()
    };

    let image_id: String;

    info!(message = "uploading image", filename);

    let image = ImageReader::new(Cursor::new(&image_data)).with_guessed_format()?;
    let mut exif_map = HashMap::new();
    let exif = ImageReader::new(Cursor::new(&image_data))
        .with_guessed_format()?
        .into_decoder()?
        .exif_metadata()?;

    if let Some(exif) = exif {
        let exif_reader = exif::Reader::new();
        let exif = exif_reader.read_raw(exif)?;

        for f in exif.fields() {
            exif_map.insert(f.tag.to_string(), f.display_value().to_string());
        }
    }

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

        let new_id = get_id();

        let timestamp;

        if let Some(captured_at) = exif_map.get("DateTimeOriginal") {
            let mut captured_at = strtime::parse("%Y-%m-%d %H:%M:%S", captured_at)?;
            captured_at.set_offset(Some(tz::offset(0)));
            timestamp = captured_at.to_timestamp()?.to_string();
        } else {
            timestamp = Timestamp::from_millisecond(
                last_modified
                    .parse()
                    .unwrap_or(Timestamp::now().as_millisecond()),
            )?
            .to_string();
        }

        let mut stmt = connection.prepare(
            "INSERT INTO image (id, filename, captured_at, metadata, user_id) VALUES (?1, ?2, ?3, ?4, ?5) RETURNING id",
        )?;

        let mut result = stmt.query(params![
            new_id,
            filename,
            timestamp,
            serde_json::to_string(&exif_map)?,
            user.id
        ])?;
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

    let original_image = image.decode()?;

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

    let medium_image = compress_image(&original_image, medium_dimension, medium_quality)?;
    let small_image = compress_image(&original_image, small_dimension, small_quality)?;

    let object_name_original = get_object_name();
    let object_name_medium = get_object_name();
    let object_name_small = get_object_name();

    {
        let connection = state.conn.lock().await;

        connection.execute(
                "INSERT INTO variant (object_name, width, height, compression_quality, quality, image_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (&object_name_original, dimensions.0, dimensions.1, original_quality, "original", &image_id),
            )?;
        connection.execute(
                "INSERT INTO variant (object_name, width, height, compression_quality, quality, image_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (&object_name_medium, medium_dimension.0, medium_dimension.1, medium_quality, "medium", &image_id),
            )?;
        connection.execute(
                "INSERT INTO variant (object_name, width, height, compression_quality, quality, image_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (&object_name_small, small_dimension.0, small_dimension.1, small_quality, "small", &image_id),
            )?;
    }

    let [original_url, medium_url, small_url] = {
        let bucket = state.bucket.lock().await;

        let original = bucket.presign_put(&object_name_original, UPLOAD_LINK_TIMEOUT_SEC, None)?;
        let medium = bucket.presign_put(&object_name_medium, UPLOAD_LINK_TIMEOUT_SEC, None)?;
        let small = bucket.presign_put(&object_name_small, UPLOAD_LINK_TIMEOUT_SEC, None)?;

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

    Ok(StatusCode::OK)
}

#[derive(Serialize)]
pub struct Image {
    id: String,
    captured_at: String,
}

#[tracing::instrument(skip_all, fields(
    username = %user.username,
))]
pub async fn get_images(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<Image>>), AppError> {
    let connection = state.conn.lock().await;

    let mut stmt = connection.prepare(
        "
            SELECT id, captured_at
            FROM image
            WHERE user_id = ?1;
        ",
    )?;

    let files = stmt.query_map([user.id], |row| {
        Ok(Image {
            id: row.get::<usize, String>(0)?.to_string(),
            captured_at: row.get(1)?,
        })
    })?;

    let mut files = files.collect::<Result<Vec<_>, _>>()?;
    files.sort_by(|a, b| b.captured_at.cmp(&a.captured_at));

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
    info!(message = "get image");

    check_image_exists(state.conn.clone(), &user.id.to_string(), &id).await?;

    let connection = state.conn.lock().await;
    let bucket = state.bucket.lock().await;

    let mut stmt = connection.prepare(
        "
            SELECT variant.object_name
            FROM variant INNER JOIN image ON variant.image_id = image.id
            WHERE image.user_id = ?1 AND image.id = ?2 AND variant.quality = ?3;
        ",
    )?;

    let mut files = stmt.query_map(
        [
            user.id.to_string(),
            id.to_string(),
            params.quality.to_string(),
        ],
        |row| Ok(row.get(0)?),
    )?;

    let file: Option<Result<String, _>> = files.next();

    match file {
        Some(Ok(object_name)) => {
            let url =
                bucket.presign_get(format!("/{}", object_name), UPLOAD_LINK_TIMEOUT_SEC, None)?;

            return Ok(Redirect::temporary(&url));
        }
        _ => {
            warn!(message = "image with requested quality doesn't exist");
            return Err(AppError::Status(StatusCode::NOT_FOUND));
        }
    }
}

#[tracing::instrument(skip_all, fields(
    user_id = %user_id,
    image_id = %image_id,
))]
async fn check_image_exists(
    connection: Arc<Mutex<Connection>>,
    user_id: &str,
    image_id: &str,
) -> Result<(), AppError> {
    let connection = connection.lock().await;
    let mut stmt = connection.prepare(
        "
            SELECT filename
            FROM image
            WHERE user_id = ?1 AND id = ?2
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
