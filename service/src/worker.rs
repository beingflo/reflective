use std::{io::Cursor, time::Duration};

use async_channel::Receiver;
use image::{ImageReader, GenericImageView};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::{query, Pool, Postgres};
use tokio::time::sleep;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    error::AppError,
    s3_utils::{cleanup_s3_objects, delete_s3_object, download_image_from_s3},
    utils::{compress_image, get_object_name},
    AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageProcessingJob {
    pub image_id: Uuid,
    pub account_id: Uuid,
    pub original_object_name: String,
}

pub struct Worker {
    receiver: Receiver<ImageProcessingJob>,
    pool: Pool<Postgres>,
    s3_bucket: s3::Bucket,
}

impl Worker {
    pub fn new(
        receiver: Receiver<ImageProcessingJob>,
        pool: Pool<Postgres>,
        s3_bucket: s3::Bucket,
    ) -> Self {
        Self {
            receiver,
            pool,
            s3_bucket,
        }
    }

    pub async fn run(&self) {
        info!("Starting image processing worker");
        
        while let Ok(job) = self.receiver.recv().await {
            info!(
                "Processing image job for image_id: {}, account_id: {}",
                job.image_id, job.account_id
            );

            if let Err(e) = self.process_image(job).await {
                error!("Failed to process image: {:?}", e);
            }
        }
    }

    async fn process_image(&self, job: ImageProcessingJob) -> Result<(), AppError> {
        // Download the original image from S3
        let original_data = download_image_from_s3(&self.s3_bucket, &job.original_object_name).await?;
        
        // Process the image
        let image = ImageReader::new(Cursor::new(&original_data))
            .with_guessed_format()?
            .decode()?;

        let dimensions = image.dimensions();

        // Create medium and small variants
        let medium_dimension = (dimensions.0 / 2, dimensions.1 / 2);
        let small_dimension = (dimensions.0 / 4, dimensions.1 / 4);

        let medium_quality = 80;
        let small_quality = 80;

        let medium_image = compress_image(&image, medium_dimension, medium_quality)?;
        let small_image = compress_image(&image, small_dimension, small_quality)?;

        // Generate object names for variants
        let object_name_medium = get_object_name();
        let object_name_small = get_object_name();

        // Upload compressed variants to S3 first
        let client = Client::new();
        
        let medium_url = self.s3_bucket.presign_put(&object_name_medium, 600, None)?;
        let small_url = self.s3_bucket.presign_put(&object_name_small, 600, None)?;

        let medium_upload = client.put(medium_url).body(medium_image).send();
        let small_upload = client.put(small_url).body(small_image).send();

        let (medium_res, small_res) = tokio::join!(medium_upload, small_upload);

        if let Err(e) = medium_res {
            error!("Failed to upload medium image: {:?}", e);
            return Err(AppError::Status(axum::http::StatusCode::INTERNAL_SERVER_ERROR));
        }

        if let Err(e) = small_res {
            error!("Failed to upload small image: {:?}", e);
            // Clean up medium image that was successfully uploaded
            if let Err(cleanup_err) = delete_s3_object(&self.s3_bucket, &object_name_medium).await {
                error!("Failed to cleanup medium image after small upload failure: {:?}", cleanup_err);
            }
            return Err(AppError::Status(axum::http::StatusCode::INTERNAL_SERVER_ERROR));
        }

        // Start database transaction for atomic variant insertion
        let mut tx = self.pool.begin().await?;

        // Insert medium variant record within transaction
        let medium_insert_result = query!(
            "INSERT INTO variant (id, object_name, width, height, compression_quality, quality, version, image_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            Uuid::now_v7(), &object_name_medium, medium_dimension.0 as i32, medium_dimension.1 as i32, medium_quality as i32, "medium", 1 as i64, &job.image_id
        ).execute(&mut *tx).await;

        if let Err(e) = medium_insert_result {
            error!("Failed to insert medium variant record: {:?}", e);
            // Rollback transaction
            tx.rollback().await?;
            // Clean up uploaded S3 objects
            if let Err(cleanup_err) = cleanup_s3_objects(&self.s3_bucket, &[&object_name_medium, &object_name_small]).await {
                error!("Failed to cleanup S3 objects after DB failure: {:?}", cleanup_err);
            }
            return Err(AppError::DBError(e));
        }

        // Insert small variant record within transaction
        let small_insert_result = query!(
            "INSERT INTO variant (id, object_name, width, height, compression_quality, quality, version, image_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            Uuid::now_v7(), &object_name_small, small_dimension.0 as i32, small_dimension.1 as i32, small_quality as i32, "small", 1 as i64, &job.image_id
        ).execute(&mut *tx).await;

        if let Err(e) = small_insert_result {
            error!("Failed to insert small variant record: {:?}", e);
            // Rollback transaction
            tx.rollback().await?;
            // Clean up uploaded S3 objects
            if let Err(cleanup_err) = cleanup_s3_objects(&self.s3_bucket, &[&object_name_medium, &object_name_small]).await {
                error!("Failed to cleanup S3 objects after DB failure: {:?}", cleanup_err);
            }
            return Err(AppError::DBError(e));
        }

        // Commit transaction - everything succeeded
        if let Err(e) = tx.commit().await {
            error!("Failed to commit transaction: {:?}", e);
            // Clean up uploaded S3 objects since commit failed
            if let Err(cleanup_err) = cleanup_s3_objects(&self.s3_bucket, &[&object_name_medium, &object_name_small]).await {
                error!("Failed to cleanup S3 objects after commit failure: {:?}", cleanup_err);
            }
            return Err(AppError::DBError(e));
        }

        info!(
            "Successfully processed image variants for image_id: {}",
            job.image_id
        );

        Ok(())
    }

}

pub async fn start_workers(
    state: AppState,
    receiver: Receiver<ImageProcessingJob>,
    worker_count: usize,
) {
    for i in 0..worker_count {
        let worker_receiver = receiver.clone();
        let worker_pool = state.pool.clone();
        let worker_bucket = {
            let bucket_guard = state.bucket.lock().await;
            bucket_guard.clone()
        };

        tokio::spawn(async move {
            info!("Starting worker {}", i);
            let worker = Worker::new(worker_receiver, worker_pool, worker_bucket);
            
            // Add retry logic for worker failures
            loop {
                worker.run().await;
                warn!("Worker {} stopped, restarting in 5 seconds", i);
                sleep(Duration::from_secs(5)).await;
            }
        });
    }
}