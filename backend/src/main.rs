mod auth;
mod error;
mod image;
mod migration;
mod user;
mod utils;

use std::sync::Arc;

use auth::{login, signup};
use axum::{
    routing::{get, patch, post},
    Router,
};
use dotenv::dotenv;
use image::{get_image, get_images, upload_images};
use migration::apply_migrations;
use rusqlite::Connection;
use tokio::sync::Mutex;
use user::update_config;

#[derive(Clone)]
pub struct AppState {
    conn: Arc<Mutex<Connection>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let mut conn = Connection::open("./db.sqlite")?;

    apply_migrations(&mut conn);

    let app = Router::new()
        .route("/auth/signup", post(signup))
        .route("/auth/login", post(login))
        .route("/user/config", patch(update_config))
        .route("/images/upload", post(upload_images))
        .route("/images", get(get_images))
        .route("/images/:id", get(get_image))
        .with_state(AppState {
            conn: Arc::new(Mutex::new(conn)),
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
