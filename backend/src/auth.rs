use crate::{error::AppError, user::S3Data, utils::get_auth_token};
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

pub struct DBUser {
    id: u64,
    password: String,
}

pub struct AuthenticatedUser {
    pub id: u64,
    pub username: String,
    pub config: Option<S3Data>,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let connection = state.conn.lock().await;

        let cookies = match CookieJar::from_request_parts(parts, state).await {
            Ok(cookies) => cookies,
            Err(_) => return Err(AppError::Status(StatusCode::INTERNAL_SERVER_ERROR)),
        };

        let token = match cookies.get("token") {
            Some(token) => token,
            None => return Err(AppError::Status(StatusCode::UNAUTHORIZED)),
        };

        let mut stmt = connection.prepare(
            "
                    SELECT users.id, users.username, users.config 
                    FROM users INNER JOIN tokens ON tokens.user_id = users.id 
                    WHERE tokens.token = ?1
                ",
        )?;

        let mut rows = stmt.query([token.value()])?;

        let user = match rows.next()? {
            Some(row) => {
                let config: Option<String> = row.get(2)?;
                AuthenticatedUser {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    config: if let Some(config) = config {
                        serde_json::from_str(&config)?
                    } else {
                        None
                    },
                }
            }
            None => return Err(AppError::Status(StatusCode::UNAUTHORIZED)),
        };

        return Ok(user);
    }
}

pub async fn signup(
    State(state): State<AppState>,
    Json(user): Json<User>,
) -> Result<StatusCode, AppError> {
    {
        let connection = state.conn.lock().await;

        let mut stmt =
            connection.prepare("SELECT username, password FROM users WHERE username = ?1")?;

        let mut rows = stmt.query([&user.username])?;

        // User already exists
        if let Some(_) = rows.next()? {
            return Err(AppError::Status(StatusCode::CONFLICT));
        }
    }

    let password = match hash(user.password, BCRYPT_COST) {
        Ok(pw) => pw,
        Err(_) => return Err(AppError::Status(StatusCode::INTERNAL_SERVER_ERROR)),
    };

    let connection = state.conn.lock().await;

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
    let db_user = {
        let connection = state.conn.lock().await;

        let mut stmt = connection.prepare("SELECT id, password FROM users WHERE username = ?1")?;
        let mut rows = stmt.query([user.username])?;

        let db_user = match rows.next()? {
            Some(row) => DBUser {
                id: row.get(0)?,
                password: row.get(1)?,
            },
            None => return Err(AppError::Status(StatusCode::UNAUTHORIZED)),
        };

        db_user
    };

    match verify(user.password, &db_user.password) {
        Err(_) => return Err(AppError::Status(StatusCode::INTERNAL_SERVER_ERROR)),
        Ok(false) => return Err(AppError::Status(StatusCode::UNAUTHORIZED)),
        Ok(true) => (),
    }

    let auth_token = get_auth_token();

    let connection = state.conn.lock().await;

    connection.execute(
        "INSERT INTO tokens (token, user_id) VALUES (?1, ?2)",
        (&auth_token, &db_user.id),
    )?;

    let jar = jar.add(
        Cookie::build(("token", auth_token))
            .path("/")
            .http_only(true),
    );
    Ok((jar, StatusCode::OK))
}
