use crate::{error::AppError, utils::get_auth_token};
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    Json,
};
use axum_extra::extract::{
    cookie::{self, Cookie},
    CookieJar,
};
use bcrypt::verify;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query, query_as};
use time::Duration;
use tracing::{error, info, Span};
use uuid::Uuid;

use crate::AppState;

const BCRYPT_COST: u32 = 12;
const SESSION_DURATION_DAYS: i64 = 30;

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub username: String,
    pub password: String,
}

#[derive(FromRow)]
pub struct DBAccount {
    id: Uuid,
    username: String,
    password: String,
}

pub struct AuthenticatedAccount {
    pub id: Uuid,
    pub username: String,
}

impl FromRequestParts<AppState> for AuthenticatedAccount {
    type Rejection = AppError;

    #[tracing::instrument(skip_all, name = "authenticate_user", fields(user_id, username))]
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

        let result = query_as!(
            DBAccount,
            "
                SELECT account.id, account.username, account.password
                FROM account INNER JOIN token ON token.account_id = account.id 
                WHERE token.token = $1;
            ",
            token.value()
        )
        .fetch_optional(&state.pool)
        .await?;

        let Some(account) = result else {
            error!(message = "No account found for token");
            return Err(AppError::Status(StatusCode::UNAUTHORIZED));
        };

        Span::current().record("user_id", account.id.to_string());
        Span::current().record("username", account.username.to_string());

        Ok(AuthenticatedAccount {
            id: account.id,
            username: account.username,
        })
    }
}

#[tracing::instrument(skip_all, fields( account = %account.username ))]
pub async fn login(
    jar: CookieJar,
    State(state): State<AppState>,
    Json(account): Json<Account>,
) -> Result<(CookieJar, StatusCode), AppError> {
    let result = query_as!(
        DBAccount,
        "SELECT id, username, password FROM account WHERE username = $1;",
        &account.username
    )
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

    query!(
        "INSERT INTO token (id, token, account_id) VALUES ($1, $2, $3);",
        Uuid::now_v7(),
        &auth_token,
        db_account.id
    )
    .execute(&state.pool)
    .await?;

    let jar = jar.add(
        Cookie::build(("token", auth_token))
            .path("/")
            .http_only(true)
            .max_age(Duration::days(SESSION_DURATION_DAYS))
            .secure(true)
            .same_site(cookie::SameSite::Strict),
    );

    info!(message = "Account logged in");

    Ok((jar, StatusCode::OK))
}
