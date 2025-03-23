mod auth;
mod error;
mod image;
mod migration;
mod tag;
mod utils;

use std::sync::Arc;

use auth::{login, signup};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{delete, get, post},
};
use dotenv::dotenv;
use image::{get_image, get_images, upload_image};
use migration::apply_migrations;
use rusqlite::Connection;
use s3::Bucket;
use tag::{add_tags, remove_tags};
use tokio::sync::Mutex;
use tracing::info;
use utils::get_bucket;

#[derive(Clone, Debug)]
pub struct AppState {
    conn: Arc<Mutex<Connection>>,
    bucket: Arc<Mutex<Bucket>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let bucket = get_bucket()?;

    let subscriber = tracing_subscriber::fmt().finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let mut conn = Connection::open("./db.sqlite")?;

    apply_migrations(&mut conn);

    let app = Router::new()
        .route("/api/auth/signup", post(signup))
        .route("/api/auth/login", post(login))
        .route("/api/images", post(upload_image))
        .route("/api/images", get(get_images))
        .route("/api/images/{id}", get(get_image))
        .route("/api/tags", post(add_tags))
        .route("/api/tags", delete(remove_tags))
        .with_state(AppState {
            conn: Arc::new(Mutex::new(conn)),
            bucket: Arc::new(Mutex::new(bucket)),
        })
        .layer(DefaultBodyLimit::disable());

    let port = 3001;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    info!(message = "Starting server", port);
    axum::serve(listener, app).await?;

    Ok(())
}
