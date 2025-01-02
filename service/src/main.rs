mod auth;
mod error;
mod image;
mod migration;
mod user;
mod utils;

use std::sync::Arc;

use auth::{login, signup};
use axum::{
    extract::DefaultBodyLimit,
    routing::{get, patch, post},
    Router,
};
use dotenv::dotenv;
use image::{get_image, get_images, upload_image};
use migration::apply_migrations;
use rusqlite::Connection;
use tokio::sync::Mutex;
use tracing::info;
use user::update_config;

#[derive(Clone, Debug)]
pub struct AppState {
    conn: Arc<Mutex<Connection>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let subscriber = tracing_subscriber::fmt().finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let mut conn = Connection::open("./db.sqlite")?;

    apply_migrations(&mut conn);

    let app = Router::new()
        .route("/api/auth/signup", post(signup))
        .route("/api/auth/login", post(login))
        .route("/api/user/config", patch(update_config))
        .route("/api/images/upload", post(upload_image))
        .route("/api/images", get(get_images))
        .route("/api/images/:id", get(get_image))
        .with_state(AppState {
            conn: Arc::new(Mutex::new(conn)),
        })
        .layer(DefaultBodyLimit::disable());

    let port = 3001;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    info!(message = "Starting server", port);
    axum::serve(listener, app).await?;

    Ok(())
}
