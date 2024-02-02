use crate::error::AppError;
use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    Json,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use bcrypt::{hash, verify};
use serde::{Deserialize, Serialize};

use crate::AppState;

const BCRYPT_COST: u32 = 12;

#[derive(Serialize, Deserialize)]
pub struct User {
    username: String,
    password: String,
}

pub struct AuthenticatedUser {
    pub id: u64,
    pub username: String,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookies = match CookieJar::from_request_parts(parts, state).await {
            Ok(cookies) => cookies,
            Err(_) => return Err(AppError::Status(StatusCode::INTERNAL_SERVER_ERROR)),
        };

        let token = match cookies.get("token") {
            Some(token) => token,
            None => return Err(AppError::Status(StatusCode::UNAUTHORIZED)),
        };

        return Ok(AuthenticatedUser {
            id: 123,
            username: "florian".to_string(),
        });
    }
}

pub async fn signup(
    State(state): State<AppState>,
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

    let password = match hash(user.password, BCRYPT_COST) {
        Ok(pw) => pw,
        Err(_) => return Err(AppError::Status(StatusCode::INTERNAL_SERVER_ERROR)),
    };

    connection.execute(
        "INSERT INTO users (username, password) VALUES (?1, ?2)",
        (&user.username, password),
    )?;

    Ok(StatusCode::OK)
}

pub async fn login(
    jar: CookieJar,
    State(state): State<AppState>,
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
        None => return Err(AppError::Status(StatusCode::UNAUTHORIZED)),
    };

    match verify(user.password, &db_user.password) {
        Err(_) => return Err(AppError::Status(StatusCode::INTERNAL_SERVER_ERROR)),
        Ok(false) => return Err(AppError::Status(StatusCode::UNAUTHORIZED)),
        Ok(true) => (),
    }

    let jar = jar.add(Cookie::build(("token", "test")).path("/").http_only(true));
    Ok((jar, StatusCode::OK))
}
