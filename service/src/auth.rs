use crate::{error::AppError, utils::get_auth_token};
use axum::{
    Json,
    extract::{FromRequestParts, State},
    http::{StatusCode, request::Parts},
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use bcrypt::{hash, verify};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query, query_as};
use tracing::{error, info};

use crate::AppState;

const BCRYPT_COST: u32 = 12;

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub username: String,
    pub password: String,
}

#[derive(FromRow)]
pub struct DBAccount {
    id: i32,
    username: String,
    password: String,
}

pub struct AuthenticatedAccount {
    pub id: i32,
    pub username: String,
}

impl FromRequestParts<AppState> for AuthenticatedAccount {
    type Rejection = AppError;

    #[tracing::instrument(skip_all)]
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
            None => {
                error!(message = "Missing token cookie");
                return Err(AppError::Status(StatusCode::UNAUTHORIZED));
            }
        };

        let result = query_as::<_, DBAccount>(
            "
                SELECT account.id, account.username, account.password
                FROM account INNER JOIN token ON token.account_id = account.id 
                WHERE token.token = $1;
            ",
        )
        .bind(token.value())
        .fetch_optional(&state.pool)
        .await?;

        let Some(account) = result else {
            error!(message = "No account found for token");
            return Err(AppError::Status(StatusCode::UNAUTHORIZED));
        };

        Ok(AuthenticatedAccount {
            id: account.id,
            username: account.username,
        })
    }
}

#[tracing::instrument(skip_all, fields( account = %account.username ))]
pub async fn signup(
    State(state): State<AppState>,
    Json(account): Json<Account>,
) -> Result<StatusCode, AppError> {
    let result =
        query_as::<_, DBAccount>("SELECT id, username, password FROM account WHERE username = $1;")
            .bind(&account.username)
            .fetch_optional(&state.pool)
            .await?;

    // Account already exists
    if let Some(_) = result {
        error!(message = "Account already exists");
        return Err(AppError::Status(StatusCode::CONFLICT));
    };

    let password = match hash(account.password, BCRYPT_COST) {
        Ok(pw) => pw,
        Err(_) => return Err(AppError::Status(StatusCode::INTERNAL_SERVER_ERROR)),
    };

    query("INSERT INTO account (username, password) VALUES ($1, $2);")
        .bind(account.username)
        .bind(password)
        .execute(&state.pool)
        .await?;

    info!(message = "Account signed up");

    Ok(StatusCode::OK)
}

#[tracing::instrument(skip_all, fields( account = %account.username ))]
pub async fn login(
    jar: CookieJar,
    State(state): State<AppState>,
    Json(account): Json<Account>,
) -> Result<(CookieJar, StatusCode), AppError> {
    let result =
        query_as::<_, DBAccount>("SELECT id, username, password FROM account WHERE username = $1;")
            .bind(&account.username)
            .fetch_optional(&state.pool)
            .await?;

    let Some(db_account) = result else {
        error!(message = "Account doesn't exist");
        return Err(AppError::Status(StatusCode::UNAUTHORIZED));
    };

    match verify(account.password, &db_account.password) {
        Err(_) => return Err(AppError::Status(StatusCode::INTERNAL_SERVER_ERROR)),
        Ok(false) => {
            error!("Password doesn't match");
            return Err(AppError::Status(StatusCode::UNAUTHORIZED));
        }
        Ok(true) => (),
    }

    let auth_token = get_auth_token();

    query("INSERT INTO token (token, account_id) VALUES ($1, $2);")
        .bind(&auth_token)
        .bind(db_account.id)
        .execute(&state.pool)
        .await?;

    let jar = jar.add(
        Cookie::build(("token", auth_token))
            .path("/")
            .http_only(true),
    );

    info!(message = "Account logged in");

    Ok((jar, StatusCode::OK))
}
