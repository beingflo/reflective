use crate::error::AppError;
use axum::{http::StatusCode, Extension, Json};
use serde::{Deserialize, Serialize};

use crate::State;

#[derive(Serialize, Deserialize)]
pub struct User {
    username: String,
    password: String,
}

pub async fn signup(Extension(state): Extension<State>, Json(user): Json<User>) -> StatusCode {
    StatusCode::OK
}

pub async fn login(
    Extension(state): Extension<State>,
    Json(user): Json<User>,
) -> Result<StatusCode, AppError> {
    let connection = state.conn.lock().await;

    let mut stmt = connection
        .prepare("SELECT username, password FROM users WHERE username = ?1")
        .unwrap();
    let mut rows = stmt.query([user.username]).unwrap();

    let user = match rows.next().expect("DB failed") {
        Some(row) => User {
            username: row.get(0)?,
            password: row.get(1)?,
        },
        None => return Ok(StatusCode::UNAUTHORIZED),
    };

    Ok(StatusCode::OK)
}
