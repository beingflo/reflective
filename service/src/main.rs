mod auth;
mod error;
mod image;
mod spa;
mod tag;
mod utils;
mod worker;

use std::sync::Arc;

use async_channel::unbounded;
use auth::{login, signup};
use axum::{
    extract::DefaultBodyLimit,
    http::StatusCode,
    routing::{delete, get, post},
    Router,
};
use dotenv::dotenv;
use error::AppError;
use image::{get_image, search_images, upload_image};
use s3::Bucket;
use spa::static_handler;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tag::{add_tags, remove_tags};
use tokio::{signal, sync::Mutex};
use tracing::{error, info};
use utils::get_bucket;
use worker::{start_workers, ImageProcessingJob};

#[derive(Clone, Debug)]
pub struct AppState {
    pool: Pool<Postgres>,
    bucket: Arc<Mutex<Bucket>>,
    job_sender: async_channel::Sender<ImageProcessingJob>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let subscriber = tracing_subscriber::fmt().finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!(message = "Starting application");

    let bucket = get_bucket()?;

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(80)
        .connect(db_url.as_str())
        .await?;

    info!(message = "Connected to DB");

    sqlx::migrate!("./migrations").run(&pool).await?;

    info!(message = "Migrations applied");

    // Create job queue for async image processing
    let (job_sender, job_receiver) = unbounded::<ImageProcessingJob>();

    let state = AppState {
        pool: pool.clone(),
        bucket: Arc::new(Mutex::new(bucket)),
        job_sender,
    };

    // Start background workers (default: 2 workers)
    let worker_count = std::env::var("WORKER_COUNT")
        .unwrap_or_else(|_| "4".to_string())
        .parse::<usize>()
        .unwrap_or(4);

    start_workers(state.clone(), job_receiver, worker_count).await;
    info!(message = "Started {} background workers", worker_count);

    let app = Router::new()
        .route("/api/auth/signup", post(signup))
        .route("/api/auth/login", post(login))
        .route("/api/images", post(upload_image))
        .route("/api/images/search", post(search_images))
        .route("/api/images/{id}", get(get_image))
        .route("/api/tags", post(add_tags))
        .route("/api/tags", delete(remove_tags))
        .fallback(static_handler)
        .with_state(state)
        .layer(DefaultBodyLimit::disable());

    let port = 3001;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    info!(message = "Starting server", port);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| {
            error!(message = "Failed to start server", error=%e);
            AppError::Status(StatusCode::SERVICE_UNAVAILABLE)
        })?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {
            info!("Ctrl+C received, shutting down")
        },
        _ = terminate => {
            info!("SIGTERM received, shutting down")
        },
    }
}
