use std::{
    collections::HashMap,
    fs::{self},
    vec,
};

use crate::{
    auth::AuthenticatedAccount,
    error::AppError,
    tag::{add_tags, TagChangeRequest},
    utils::{compress_image, get_object_name},
};
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::Response,
    Json,
};
use image::{GenericImageView, ImageDecoder, ImageReader};
use jiff::{fmt::strtime, tz, Timestamp};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, FromRow};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tracing::{error, info, warn};
use uuid::Uuid;
use walkdir::{DirEntry, WalkDir};

use crate::AppState;

#[tracing::instrument(skip_all)]
pub async fn scan_disk(state: AppState) -> Result<(), AppError> {
    info!(message = "Starting scan for new images");
    verify_images(&state).await?;
    info!(message = "Finished scanning for new images");

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn verify_images(state: &AppState) -> Result<(), AppError> {
    let images_dir = std::env::var("IMAGE_DIR").expect("IMAGE_DIR must be set");

    let mut files = Vec::new();

    for entry in WalkDir::new(images_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            files.push(entry);
        }
    }

    // For each image check that
    // - the image is in the DB
    // - all variants exist
    for file in files {
        // Is image in image table
        let image_id = match image_indexed(state, &file).await {
            Ok(Some(id)) => id,
            Ok(None) => {
                if let Ok(id) = add_image(state, &file).await {
                    id
                } else {
                    error!(message="indexing image failed", file_name=%file.file_name().to_str().unwrap());
                    continue;
                }
            }
            Err(error) => {
                error!(message="indexing image failed", file_name=%file.file_name().to_str().unwrap(), %error);
                continue;
            }
        };

        match index_compressed_image(state, image_id, &file, 2.0, "medium").await {
            Ok(_) => {}
            Err(error) => {
                error!(message="indexing compressed image failed", file_name=%file.file_name().to_str().unwrap(), %error);
            }
        }
        match index_compressed_image(state, image_id, &file, 4.0, "small").await {
            Ok(_) => {}
            Err(error) => {
                error!(message="indexing compressed image failed", file_name=%file.file_name().to_str().unwrap(), %error);
            }
        }
    }

    Ok(())
}

async fn index_compressed_image(
    state: &AppState,
    image_id: Uuid,
    file: &DirEntry,
    dimension_reduction: f32,
    quality: &str,
) -> Result<(), AppError> {
    #[derive(FromRow)]
    struct Variant {
        object_name: String,
    }

    let result = query_as!(
        Variant,
        "
            SELECT variant.object_name
            FROM variant INNER JOIN image ON variant.image_id = image.id
            WHERE image.id = $1 AND variant.quality = $2;
        ",
        image_id,
        quality
    )
    .fetch_optional(&state.pool)
    .await?;

    let mut tx = state.pool.begin().await?;

    let object_name = match result {
        None => {
            info!(message = "variant is not indexed", %quality, image_id = %image_id);

            let image = ImageReader::open(file.path())?.with_guessed_format()?;
            let original_image = image.decode()?;

            let dimensions = original_image.dimensions();

            let object_name = get_object_name();
            let variant_insert_result = query!(
                    "INSERT INTO variant (id, object_name, width, height, compression_quality, quality, version, image_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                    Uuid::now_v7(), &object_name, (dimensions.0 as f32 / dimension_reduction as f32) as i32, (dimensions.1 as f32 / dimension_reduction as f32) as i32, 80, quality, 1 as i64, &image_id
                ).execute(&mut *tx).await;

            if let Err(e) = variant_insert_result {
                error!(message = "failed to insert variant record", error = ?e);
                // Rollback transaction
                tx.rollback().await?;
                return Err(AppError::DBError(e));
            }
            object_name
        }
        Some(variant) => variant.object_name,
    };

    let caches_dir = std::env::var("IMAGE_CACHE_DIR").expect("IMAGE_CACHE_DIR must be set");
    let caches_dir = std::path::Path::new(&caches_dir);

    let file_path = caches_dir.join(object_name);
    if file_path.exists() && file_path.is_file() {
        return Ok(());
    }

    info!(message = "variant file does not exist", %quality, image_id = %image_id);

    let image = ImageReader::open(file.path())?.with_guessed_format()?;
    let original_image = image.decode()?;

    let dimensions = original_image.dimensions();
    let width = (dimensions.0 as f32 / dimension_reduction as f32) as u32;
    let height = (dimensions.1 as f32 / dimension_reduction as f32) as u32;

    let compressed_image = compress_image(&original_image, (width, height), 80)?;
    fs::write(file_path, &compressed_image)?;

    tx.commit().await?;

    Ok(())
}

async fn add_image(state: &AppState, file: &DirEntry) -> Result<Uuid, AppError> {
    let image_id = Uuid::now_v7();

    let images_dir = std::env::var("IMAGE_DIR").expect("IMAGE_DIR must be set");

    let image = ImageReader::open(file.path())?.with_guessed_format()?;
    let original_image = image.decode()?;

    let dimensions = original_image.dimensions();
    let aspect_ratio = dimensions.0 as f64 / dimensions.1 as f64;

    // Generate object name for original
    let object_name_original = get_object_name();

    let mut tx = state.pool.begin().await?;

    let (captured_at, exif) = get_capture_timestamp(&file)?;

    // Insert image record within transaction
    let image_insert_result = query!(
                    "INSERT INTO image (id, filename, captured_at, aspect_ratio, metadata) VALUES ($1, $2, $3, $4, $5);",
                    image_id,
                    file.file_name().to_str(),
                    captured_at,
                    aspect_ratio,
                    serde_json::to_string(&exif)?,
                ).execute(&mut *tx).await;

    // Add path segments to tags
    // E.g. `/image_dir/france/lyon/test.jpeg` will get tags `france` and `lyon`
    let relative = file
        .path()
        .strip_prefix(images_dir)
        .expect("file does not have IMAGE_DIR prefix");
    let relative = relative.parent().expect("file has no parent");
    let folders: Vec<String> = relative
        .components()
        .map(|c| {
            c.as_os_str()
                .to_str()
                .expect("Failed to turn Path component to string")
                .to_string()
        })
        .collect();
    match add_tags(
        TagChangeRequest {
            image_ids: vec![image_id],
            tags: folders,
        },
        &mut tx,
    )
    .await
    {
        Ok(()) => {}
        Err(error) => {
            warn!(message="Adding tags failed", %image_id);
            return Err(error);
        }
    };

    if let Err(e) = image_insert_result {
        error!(message = "failed to insert image record", error = ?e);
        // Rollback transaction
        tx.rollback().await?;
        return Err(AppError::DBError(e));
    }

    // Insert original variant record within transaction
    let original_quality = 100;
    let variant_insert_result = query!(
                    "INSERT INTO variant (id, object_name, width, height, compression_quality, quality, version, image_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                    Uuid::now_v7(), &object_name_original, dimensions.0 as i32, dimensions.1 as i32, original_quality, "original", 1 as i64, &image_id
                ).execute(&mut *tx).await;

    if let Err(e) = variant_insert_result {
        error!(message = "failed to insert variant record", error = ?e);
        // Rollback transaction
        tx.rollback().await?;
        return Err(AppError::DBError(e));
    }

    // Commit transaction - everything succeeded
    tx.commit().await?;
    info!(message = "inserted image into DB", image_id = %image_id);

    Ok(image_id)
}

async fn image_indexed(state: &AppState, file: &DirEntry) -> Result<Option<Uuid>, AppError> {
    let (captured_at, _) = get_capture_timestamp(&file)?;

    #[derive(FromRow)]
    struct File {
        id: Uuid,
    }

    let result = query_as!(
        File,
        "
            SELECT id 
            FROM image
            WHERE filename = $1 AND captured_at = $2;
        ",
        file.file_name().to_str(),
        captured_at
    )
    .fetch_optional(&state.pool)
    .await?;

    if let Some(file) = result {
        return Ok(Some(file.id));
    };

    info!(message = "image is not indexed", file = %file.file_name()
        .to_str()
        .expect("file name not unicode"));

    Ok(None)
}

fn get_capture_timestamp(file: &DirEntry) -> Result<(String, HashMap<String, String>), AppError> {
    let last_modified = file.metadata()?.modified()?;
    let exif = extract_exif(&file)?;
    let timestamp;

    if let Some(captured_at) = exif.get("DateTimeOriginal") {
        let mut captured_at = strtime::parse("%Y-%m-%d %H:%M:%S", captured_at)?;
        captured_at.set_offset(Some(tz::offset(0)));
        timestamp = captured_at.to_timestamp()?.to_string();
    } else {
        let ts: Timestamp = last_modified.try_into()?;
        timestamp = ts.to_string();
    }

    Ok((timestamp, exif))
}

fn extract_exif(file: &DirEntry) -> Result<HashMap<String, String>, AppError> {
    let image = ImageReader::open(file.path())?.with_guessed_format()?;
    let mut exif_map = HashMap::new();
    let exif = image.into_decoder()?.exif_metadata()?;

    if let Some(exif) = exif {
        let exif_reader = exif::Reader::new();
        let exif = exif_reader.read_raw(exif)?;

        for f in exif.fields() {
            exif_map.insert(f.tag.to_string(), f.display_value().to_string());
        }
    }

    Ok(exif_map)
}

#[derive(Serialize, FromRow)]
pub struct Image {
    id: Uuid,
    captured_at: String,
    aspect_ratio: f64,
    tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct SearchBody {
    query: String,
}

#[tracing::instrument(skip_all, fields(
    username = %account.username,
))]
pub async fn search_images(
    account: AuthenticatedAccount,
    State(state): State<AppState>,
    body: Json<SearchBody>,
) -> Result<(StatusCode, Json<Vec<Image>>), AppError> {
    #[derive(FromRow, Debug)]
    struct Tag {
        id: Uuid,
        description: String,
    }

    let tags = query_as!(
        Tag,
        "
            SELECT tag.id, tag.description
            FROM tag;
        ",
    )
    .fetch_all(&state.pool)
    .await?;

    let tags = if body.query.len() < 3 {
        vec![]
    } else {
        let search_terms = body
            .query
            .split_whitespace()
            .filter(|term| term.len() > 2)
            .collect::<Vec<_>>();
        let mut filtered_tags = vec![];
        for term in &search_terms {
            for tag in &tags {
                if tag.description.contains(term) {
                    filtered_tags.push(tag);
                    break;
                }
            }
        }
        filtered_tags
    };

    if tags.is_empty() && body.query.len() >= 3 {
        return Ok((StatusCode::OK, Json(vec![])));
    }

    let mut images = query_as!(
        Image,
        "
            SELECT image.id, image.captured_at, image.aspect_ratio, ARRAY_REMOVE( ARRAY_AGG(tag.description), NULL) tags
            FROM image
            LEFT JOIN image_tag ON image.id = image_tag.image_id
            LEFT JOIN tag ON image_tag.tag_id = tag.id
            GROUP BY image.id
            HAVING ARRAY_AGG(tag_id::text) @> ARRAY[$1::text[]];
        ",
        &tags.iter().map(|tag| tag.id.to_string()).collect::<Vec<_>>()
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
    image_id = %image_id,
    quality = %params.quality
))]
pub async fn get_image(
    account: AuthenticatedAccount,
    Path(image_id): Path<Uuid>,
    params: Query<QueryParams>,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    info!(message = "get image");

    #[derive(FromRow)]
    struct Variant {
        object_name: String,
    }
    let result = query_as!(
        Variant,
        "
            SELECT variant.object_name
            FROM variant INNER JOIN image ON variant.image_id = image.id
            WHERE image.id = $1 AND variant.quality = $2;
        ",
        image_id,
        params.quality
    )
    .fetch_optional(&state.pool)
    .await?;

    let Some(object) = result else {
        warn!(message = "image with requested quality doesn't exist");
        return Err(AppError::Status(StatusCode::NOT_FOUND));
    };

    let caches_dir = std::env::var("IMAGE_CACHE_DIR").expect("IMAGE_CACHE_DIR must be set");
    let caches_dir = std::path::Path::new(&caches_dir);

    let file_path = caches_dir.join(object.object_name);

    if !file_path.exists() || !file_path.is_file() {
        error!(
            message = "file does not exist on disk or is not file",
            file_path = file_path.to_str()
        );
        return Err(AppError::Status(StatusCode::NOT_FOUND));
    }

    let file = File::open(&file_path).await?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/jpeg")
        .body(body)
        .unwrap())
}
