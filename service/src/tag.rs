use crate::{auth::AuthenticatedAccount, error::AppError};
use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use sqlx::{query, query_as};
use tracing::{error, info};

use crate::AppState;

#[derive(Deserialize)]
pub struct TagChangeRequest {
    image_ids: Vec<String>,
    tags: Vec<String>,
}

#[tracing::instrument(skip_all, fields(
    username = %account.username,
))]
pub async fn add_tags(
    account: AuthenticatedAccount,
    State(state): State<AppState>,
    Json(body): Json<TagChangeRequest>,
) -> Result<StatusCode, AppError> {
    info!(message = "Adding tags to images");

    // check if image_ids exist
    let images = query!(
        "SELECT id FROM image WHERE id = ANY($1) AND account_id = $2",
        &body.image_ids,
        account.id
    )
    .fetch_all(&state.pool)
    .await?;

    if images.len() != body.image_ids.len() {
        error!(message = "Some images do not exist");
        return Err(AppError::Status(StatusCode::BAD_REQUEST));
    }

    // upsert tags to ensure they exist
    let mut tx = state.pool.begin().await?;

    for tag in &body.tags {
        query!(
            "INSERT INTO tag (description, account_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            tag,
            account.id
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    #[derive(Debug)]
    struct Tag {
        id: i32,
    }

    // get tags from db
    let tags = query_as!(
        Tag,
        "SELECT id FROM tag WHERE description = ANY($1);",
        &body.tags
    )
    .fetch_all(&state.pool)
    .await?;

    // upsert image_tag relations
    let mut tx = state.pool.begin().await?;
    for tag in tags {
        for image in &body.image_ids {
            query!(
                "INSERT INTO image_tag (tag_id, image_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                tag.id,
                image
            )
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;

    Ok(StatusCode::OK)
}

#[tracing::instrument(skip_all, fields(
    username = %user.username,
))]
pub async fn remove_tags(
    user: AuthenticatedAccount,
    State(_state): State<AppState>,
    Json(_body): Json<TagChangeRequest>,
) -> Result<StatusCode, AppError> {
    Ok(StatusCode::OK)
}
