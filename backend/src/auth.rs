use crate::error::AppError;
use axum::{http::StatusCode, Extension, Json};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use serde::{Deserialize, Serialize};

use crate::State;

#[derive(Serialize, Deserialize)]
pub struct User {
    username: String,
    password: String,
}

pub async fn signup(
    Extension(state): Extension<State>,
    Json(user): Json<User>,
) -> Result<StatusCode, AppError> {
    let connection = state.conn.lock().await;

    let mut stmt = connection
        .prepare("SELECT username, password FROM users WHERE username = ?1")
        .unwrap();

    let mut rows = stmt.query([&user.username]).unwrap();

    // User already exists
    if let Some(_) = rows.next()? {
        return Err(AppError::Status(StatusCode::CONFLICT));
    }

    // No salting for now, randomly generated password anyway
    connection.execute(
        "INSERT INTO users (username, password) VALUES (?1, ?2)",
        (&user.username, user.password),
    )?;

    Ok(StatusCode::OK)
}

pub async fn login(
    Extension(state): Extension<State>,
    jar: CookieJar,
    Json(user): Json<User>,
) -> Result<(CookieJar, StatusCode), AppError> {
    let connection = state.conn.lock().await;

    let mut stmt = connection
        .prepare("SELECT username, password FROM users WHERE username = ?1")
        .unwrap();
    let mut rows = stmt.query([user.username]).unwrap();

    let db_user = match rows.next()? {
        Some(row) => User {
            username: row.get(0)?,
            password: row.get(1)?,
        },
        None => unimplemented!(),
    };

    if user.password == db_user.password {
        let jar = jar.add(Cookie::new("session_token", "test"));
        Ok((jar, StatusCode::OK))
    } else {
        Err(AppError::Status(StatusCode::UNAUTHORIZED))
    }
}
