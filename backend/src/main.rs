mod auth;
mod migration;

use std::sync::Arc;

use auth::{login, signup};
use axum::{routing::post, Extension, Router};
use dotenv::dotenv;
use migration::apply_migrations;
use rusqlite::Connection;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct State {
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
        .layer(Extension(State {
            conn: Arc::new(Mutex::new(conn)),
        }));

    axum::Server::bind(&"0.0.0.0:3001".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
