mod auth;
mod error;
mod utils;

use std::sync::Arc;

use auth::{login, signup};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{delete, get, post},
};
use dotenv::dotenv;
use s3::Bucket;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use tokio::sync::Mutex;
use tracing::info;
use utils::get_bucket;

#[derive(Clone, Debug)]
pub struct AppState {
    pool: Pool<Postgres>,
    bucket: Arc<Mutex<Bucket>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        //.route("/api/images", post(upload_image))
        //.route("/api/images", get(get_images))
        //.route("/api/images/{id}", get(get_image))
        //.route("/api/tags", post(add_tags))
        //.route("/api/tags", delete(remove_tags))
        .with_state(AppState {
            pool,
            bucket: Arc::new(Mutex::new(bucket)),
        })
        .layer(DefaultBodyLimit::disable());

    let port = 3001;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    info!(message = "Starting server", port);

    axum::serve(listener, app).await?;

    Ok(())
}
