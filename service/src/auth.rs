use crate::{error::AppError, utils::get_auth_token};
use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    Json,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use bcrypt::{hash, verify};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

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

#[derive(Debug)]
pub struct AuthenticatedUser {
    pub id: u64,
    pub username: String,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AppError;

    #[tracing::instrument(skip_all)]
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
            None => {
                error!(message = "Missing token cookie");
                return Err(AppError::Status(StatusCode::UNAUTHORIZED));
            }
        };

        let mut stmt = connection.prepare(
            "
                    SELECT user.id, user.username 
                    FROM user INNER JOIN token ON token.user_id = user.id 
                    WHERE token.token = ?1
                ",
        )?;

        let mut rows = stmt.query([token.value()])?;

        let user = match rows.next()? {
            Some(row) => AuthenticatedUser {
                id: row.get(0)?,
                username: row.get(1)?,
            },
            None => {
                error!(message = "No user found for token");
                return Err(AppError::Status(StatusCode::UNAUTHORIZED));
            }
        };

        return Ok(user);
    }
}

#[tracing::instrument(skip_all, fields( user = %user.username ))]
pub async fn signup(
    State(state): State<AppState>,
    Json(user): Json<User>,
) -> Result<StatusCode, AppError> {
    {
        let connection = state.conn.lock().await;

        let mut stmt =
            connection.prepare("SELECT username, password FROM user WHERE username = ?1")?;

        let mut rows = stmt.query([&user.username])?;

        // User already exists
        if let Some(_) = rows.next()? {
            warn!(message = "User already exists");
            return Err(AppError::Status(StatusCode::CONFLICT));
        }
    }

    let password = match hash(user.password, BCRYPT_COST) {
        Ok(pw) => pw,
        Err(_) => return Err(AppError::Status(StatusCode::INTERNAL_SERVER_ERROR)),
    };

    let connection = state.conn.lock().await;

    connection.execute(
        "INSERT INTO user (username, password) VALUES (?1, ?2)",
        (&user.username, password),
    )?;

    info!(message = "User signed up");
    Ok(StatusCode::OK)
}

#[tracing::instrument(skip_all, fields( user = %user.username ))]
pub async fn login(
    jar: CookieJar,
    State(state): State<AppState>,
    Json(user): Json<User>,
) -> Result<(CookieJar, StatusCode), AppError> {
    let db_user = {
        let connection = state.conn.lock().await;

        let mut stmt = connection.prepare("SELECT id, password FROM user WHERE username = ?1")?;
        let mut rows = stmt.query([user.username])?;

        let db_user = match rows.next()? {
            Some(row) => DBUser {
                id: row.get(0)?,
                password: row.get(1)?,
            },
            None => {
                error!(message = "User doesn't exist");
                return Err(AppError::Status(StatusCode::UNAUTHORIZED));
            }
        };

        db_user
    };

    match verify(user.password, &db_user.password) {
        Err(_) => return Err(AppError::Status(StatusCode::INTERNAL_SERVER_ERROR)),
        Ok(false) => {
            error!("Password doesn't match");
            return Err(AppError::Status(StatusCode::UNAUTHORIZED));
        }
        Ok(true) => (),
    }

    let auth_token = get_auth_token();

    let connection = state.conn.lock().await;

    connection.execute(
        "INSERT INTO token (token, user_id) VALUES (?1, ?2)",
        (&auth_token, &db_user.id),
    )?;

    let jar = jar.add(
        Cookie::build(("token", auth_token))
            .path("/")
            .http_only(true),
    );

    info!(message = "User logged in");
    Ok((jar, StatusCode::OK))
}
