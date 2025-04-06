mod auth;
mod error;
mod image;
mod spa;
mod tag;
mod utils;

use std::sync::Arc;

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

#[derive(Clone, Debug)]
pub struct AppState {
    pool: Pool<Postgres>,
    bucket: Arc<Mutex<Bucket>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info!(message = "Entering startup sequence");
    dotenv().ok();

    let bucket = get_bucket()?;

    let subscriber = tracing_subscriber::fmt().finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let pool = PgPoolOptions::new()
        .max_connections(100)
        .connect("postgres://postgres:postgres@localhost/reflective")
        .await?;

    info!(message = "Connected to DB");

    sqlx::migrate!("./migrations").run(&pool).await?;

    info!(message = "Migrations applied");

    let app = Router::new()
        .route("/api/auth/signup", post(signup))
        .route("/api/auth/login", post(login))
        .route("/api/images", post(upload_image))
        .route("/api/images/search", post(search_images))
        .route("/api/images/{id}", get(get_image))
        .route("/api/tags", post(add_tags))
        .route("/api/tags", delete(remove_tags))
        .fallback(static_handler)
        .with_state(AppState {
            pool,
            bucket: Arc::new(Mutex::new(bucket)),
        })
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
