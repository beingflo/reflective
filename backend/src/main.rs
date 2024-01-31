mod auth;
mod error;
mod migration;
mod user;

use std::sync::Arc;

use auth::{login, signup};
use axum::{
    routing::{patch, post},
    Router,
};
use dotenv::dotenv;
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
        .with_state(AppState {
            conn: Arc::new(Mutex::new(conn)),
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
