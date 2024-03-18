use std::io::Cursor;

use crate::{
    auth::AuthenticatedUser,
    error::AppError,
    user::S3Data,
    utils::{format_filename, get_bucket, get_file_name},
};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::Redirect,
    Json,
};
use image::{codecs::jpeg::JpegEncoder, io::Reader};
use serde::Deserialize;

use crate::AppState;

const UPLOAD_LINK_TIMEOUT_SEC: u32 = 600;

pub async fn upload_image(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<StatusCode, AppError> {
    println!("upload image");
    //let connection = state.conn.lock().await;

    let config: S3Data = match user.config {
        Some(c) => c,
        None => return Err(AppError::Status(StatusCode::BAD_REQUEST)),
    };

    let bucket = get_bucket(config)?;

    let filename = get_file_name();
    let original = format_filename(&filename, "original");

    let url_original = bucket.presign_put(&original, UPLOAD_LINK_TIMEOUT_SEC, None)?;

    // connection.execute(
    //     "INSERT INTO images (filename, user_id) VALUES (?1, ?2)",
    //     (&filename, &user.id),
    // )?;

    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        let image = Reader::new(Cursor::new(&data))
            .with_guessed_format()
            .unwrap();

        let image = image.decode().unwrap();
        let downsampled = image.resize(2000, 2000, image::imageops::FilterType::Triangle);
        let mut bytes: Vec<u8> = Vec::new();
        let write = Cursor::new(&mut bytes);
        let encoder = JpegEncoder::new_with_quality(write, 70);
        downsampled.write_with_encoder(encoder).unwrap();

        println!("Length of `{}` is {} bytes", name, data.len());
        println!("Length of `{}` is {} bytes", name, bytes.len());
        // TODO upload file here with request
    }

    Ok(StatusCode::OK)
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
            WHERE user_id = ?1
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
            WHERE user_id = ?1 AND filename = ?2
        ",
    )?;

    let mut files = stmt.query_map([&user.id.to_string(), &id], |row| Ok(row.get(0)?))?;

    let config: S3Data = match user.config {
        Some(c) => c,
        None => return Err(AppError::Status(StatusCode::NOT_FOUND)),
    };

    let bucket = get_bucket(config)?;

    let file: Option<Result<String, _>> = files.next();

    if let Some(_) = file {
        let name = format_filename(&id, &params.quality);
        let url = bucket.presign_get(format!("/{}", name), UPLOAD_LINK_TIMEOUT_SEC, None)?;
        return Ok(Redirect::temporary(&url));
    }

    return Err(AppError::Status(StatusCode::NOT_FOUND));
}
