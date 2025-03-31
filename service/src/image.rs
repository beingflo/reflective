use std::{collections::HashMap, io::Cursor};

use crate::{
    auth::AuthenticatedAccount,
    error::AppError,
    utils::{compress_image, get_object_name},
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
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Postgres, query, query_as};
use tracing::{error, info, trace, warn};
use uuid::Uuid;

use crate::AppState;

const UPLOAD_LINK_TIMEOUT_SEC: u32 = 600;

#[tracing::instrument(skip_all, fields(
    username = %account.username,
))]
#[axum::debug_handler]
pub async fn upload_image(
    account: AuthenticatedAccount,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<StatusCode, AppError> {
    let mut filename: Option<String> = None;
    let mut image_data: Option<Vec<u8>> = None;
    let mut last_modified: Option<String> = None;

    while let Some(field) = multipart.next_field().await? {
        match field.name() {
            Some("filename") => {
                filename = Some(field.text().await?);
            }
            Some("last_modified") => {
                last_modified = Some(field.text().await?);
            }
            Some("data") => {
                let data = field.bytes().await?;
                image_data = Some(data.to_vec());
            }
            Some(name) => {
                error!(message = "unknown field in multipart body", name);
                return Err(AppError::Status(StatusCode::BAD_REQUEST));
            }
            _ => {
                error!(message = "field in multipart body with no name");
                return Err(AppError::Status(StatusCode::BAD_REQUEST));
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

    #[derive(FromRow)]
    struct File {
        filename: String,
    }

    let result = query_as!(
        File,
        "
            SELECT filename 
            FROM image
            WHERE account_id = $1 AND filename = $2;
        ",
        account.id,
        filename
    )
    .fetch_optional(&state.pool)
    .await?;

    if let Some(file) = result {
        error!(message = "image already exists", file = %file.filename);
        return Err(AppError::Status(StatusCode::CONFLICT));
    };

    let image_id = Uuid::now_v7();

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

    let original_image = image.decode()?;

    let dimensions = original_image.dimensions();
    let aspect_ratio = dimensions.0 as f64 / dimensions.1 as f64;

    query!(
        "INSERT INTO image (id, filename, captured_at,aspect_ratio, metadata, account_id) VALUES ($1, $2, $3, $4, $5, $6);",
        image_id,
        filename,
        timestamp,
        aspect_ratio,
        serde_json::to_string(&exif_map)?,
        account.id
    ).execute(&state.pool).await?;

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

    query!(
        "INSERT INTO variant (id, object_name, width, height, compression_quality, quality, version, image_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        Uuid::now_v7(), &object_name_original, dimensions.0 as i32, dimensions.1 as i32, original_quality, "original", 1 as i64, &image_id
    ).execute(&state.pool).await?;
    query!(
        "INSERT INTO variant (id, object_name, width, height, compression_quality, quality, version, image_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        Uuid::now_v7(), &object_name_medium, medium_dimension.0 as i32, medium_dimension.1 as i32, medium_quality as i32, "medium", 1 as i64, &image_id
    ).execute(&state.pool).await?;
    query!(
        "INSERT INTO variant (id, object_name, width, height, compression_quality, quality, version, image_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        Uuid::now_v7(), &object_name_small, small_dimension.0 as i32, small_dimension.1 as i32, small_quality as i32, "small", 1 as i64, &image_id
    ).execute(&state.pool).await?;

    let [original_url, medium_url, small_url] = {
        let bucket = state.bucket.lock().await;

        let original = bucket.presign_put(&object_name_original, UPLOAD_LINK_TIMEOUT_SEC, None)?;
        let medium = bucket.presign_put(&object_name_medium, UPLOAD_LINK_TIMEOUT_SEC, None)?;
        let small = bucket.presign_put(&object_name_small, UPLOAD_LINK_TIMEOUT_SEC, None)?;

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

#[derive(Serialize, FromRow)]
pub struct Image {
    id: Uuid,
    captured_at: String,
    aspect_ratio: f64,
    tags: Option<Vec<String>>,
}

#[tracing::instrument(skip_all, fields(
    username = %account.username,
))]
pub async fn get_images(
    account: AuthenticatedAccount,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<Image>>), AppError> {
    let mut images = query_as!(
        Image,
        "
            SELECT image.id, image.captured_at, image.aspect_ratio, ARRAY_REMOVE( ARRAY_AGG(tag.description), NULL) tags
            FROM image
            LEFT JOIN image_tag ON image.id = image_tag.image_id
            LEFT JOIN tag ON image_tag.tag_id = tag.id
            WHERE image.account_id = $1
            GROUP BY image.id;
        ",
        account.id
    )
    .fetch_all(&state.pool)
    .await?;

    images.sort_by(|a, b| b.captured_at.cmp(&a.captured_at));

    info!(message = "load image list", number_of_files = images.len());

    Ok((StatusCode::OK, Json(images)))
}

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    quality: String,
}

#[tracing::instrument(skip_all, fields(
    username = %account.username,
    id = %id,
    quality = %params.quality
))]
pub async fn get_image(
    account: AuthenticatedAccount,
    Path(id): Path<Uuid>,
    params: Query<QueryParams>,
    State(state): State<AppState>,
) -> Result<Redirect, AppError> {
    info!(message = "get image");

    check_image_exists(&state.pool, account.id, id).await?;

    #[derive(FromRow)]
    struct Variant {
        object_name: String,
    }

    let result = query_as!(
        Variant,
        "
            SELECT variant.object_name
            FROM variant INNER JOIN image ON variant.image_id = image.id
            WHERE image.account_id = $1 AND image.id = $2 AND variant.quality = $3;
        ",
        account.id,
        id,
        params.quality.to_string()
    )
    .fetch_optional(&state.pool)
    .await?;

    let bucket = state.bucket.lock().await;

    if let Some(variant) = result {
        let url = bucket.presign_get(
            format!("/{}", variant.object_name),
            UPLOAD_LINK_TIMEOUT_SEC,
            None,
        )?;
        return Ok(Redirect::temporary(&url));
    }

    warn!(message = "image with requested quality doesn't exist");
    return Err(AppError::Status(StatusCode::NOT_FOUND));
}

#[tracing::instrument(skip_all, fields(
    account_id = %account_id,
    image_id = %image_id,
))]
async fn check_image_exists(
    pool: &Pool<Postgres>,
    account_id: Uuid,
    image_id: Uuid,
) -> Result<(), AppError> {
    #[derive(FromRow)]
    struct Image {
        filename: String,
    }
    let result = query_as!(
        Image,
        "
            SELECT filename
            FROM image
            WHERE account_id = $1 AND id = $2;
        ",
        account_id,
        image_id
    )
    .fetch_optional(pool)
    .await?;

    if let Some(file) = result {
        trace!(message = "image exists", file = %file.filename);
        return Ok(());
    }

    warn!(message = "image doesn't exist");

    return Err(AppError::Status(StatusCode::NOT_FOUND));
}
