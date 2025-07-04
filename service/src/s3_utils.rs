use reqwest::Client;
use s3::Bucket;
use tracing::{error, info};

use crate::error::AppError;

/// Helper function to delete a single S3 object
pub async fn delete_s3_object(bucket: &Bucket, object_name: &str) -> Result<(), AppError> {
    match bucket.delete_object(object_name).await {
        Ok(_) => {
            info!(
                message = "successfully deleted S3 object during rollback",
                object_name
            );
            Ok(())
        }
        Err(e) => {
            error!(
                message = "failed to delete S3 object during rollback",
                object_name,
                error = ?e
            );
            Err(AppError::S3Error(e))
        }
    }
}


/// Helper function to delete multiple S3 objects
pub async fn cleanup_s3_objects(bucket: &Bucket, object_names: &[&str]) -> Result<(), AppError> {
    let mut errors = Vec::new();

    for object_name in object_names {
        if let Err(e) = delete_s3_object(bucket, object_name).await {
            errors.push(format!("Failed to delete {}: {:?}", object_name, e));
        }
    }

    if !errors.is_empty() {
        error!("S3 cleanup errors: {}", errors.join(", "));
        return Err(AppError::Status(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    info!("Successfully cleaned up all S3 objects: {:?}", object_names);
    Ok(())
}

/// Helper function to download an image from S3
pub async fn download_image_from_s3(
    bucket: &Bucket,
    object_name: &str,
) -> Result<Vec<u8>, AppError> {
    let url = bucket.presign_get(format!("/{}", object_name), 600, None)?;

    let client = Client::new();
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        error!(
            "Failed to download image from S3: {}",
            response.status()
        );
        return Err(AppError::Status(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    let data = response.bytes().await?;
    Ok(data.to_vec())
}