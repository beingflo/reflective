use crate::{auth::AuthenticatedUser, error::AppError};
use axum::{Json, extract::State, http::StatusCode};
use rusqlite::{params, params_from_iter};
use serde::Deserialize;
use tracing::error;

use crate::AppState;

#[derive(Deserialize)]
pub struct TagChangeRequest {
    image_ids: Vec<String>,
    tags: Vec<String>,
}

#[tracing::instrument(skip_all, fields(
    username = %user.username,
))]
pub async fn add_tags(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(body): Json<TagChangeRequest>,
) -> Result<StatusCode, AppError> {
    // upsert tags to ensure they exist
    // get tag ids
    // upsert image_tag relations
    Ok(StatusCode::OK)
}

#[tracing::instrument(skip_all, fields(
    username = %user.username,
))]
pub async fn remove_tags(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(body): Json<TagChangeRequest>,
) -> Result<StatusCode, AppError> {
    Ok(StatusCode::OK)
}
