use crate::{auth::AuthenticatedAccount, error::AppError};
use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;
use sqlx::{query, query_as};
use tracing::{error, info};
use uuid::Uuid;

use crate::AppState;

#[derive(Deserialize)]
pub struct TagChangeRequest {
    image_ids: Vec<Uuid>,
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

    if body.tags.len() > 10 {
        error!(message = "Too many tags");
        return Err(AppError::Text(
            StatusCode::BAD_REQUEST,
            "At most 10 tags allowed".to_string(),
        ));
    }

    // check if image_ids exist
    let images = query!("SELECT id FROM image WHERE id = ANY($1)", &body.image_ids,)
        .fetch_all(&state.pool)
        .await?;

    if images.len() != body.image_ids.len() {
        error!(message = "Some images do not exist");
        return Err(AppError::Status(StatusCode::BAD_REQUEST));
    }

    let tags = body
        .tags
        .iter()
        .map(|t| t.to_lowercase())
        .collect::<Vec<_>>();

    // upsert tags to ensure they exist
    let mut tx = state.pool.begin().await?;

    for tag in &tags {
        query!(
            "INSERT INTO tag (id, description) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            Uuid::now_v7(),
            tag,
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    #[derive(Debug)]
    struct Tag {
        id: Uuid,
    }

    // get tags from db
    let tags = query_as!(
        Tag,
        "SELECT id FROM tag WHERE description = ANY($1);",
        &tags
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
    State(state): State<AppState>,
    Json(body): Json<TagChangeRequest>,
) -> Result<StatusCode, AppError> {
    info!(message = "Removing tags from images");
    let mut tx = state.pool.begin().await?;

    let deleted_relations = query!(
        "DELETE FROM image_tag USING tag WHERE image_tag.tag_id = tag.id AND image_tag.image_id = ANY($1) AND tag.description = ANY($2);",
        &body.image_ids,
        &body.tags
    ).execute(&mut *tx).await?;
    info!(deleted_relations = %deleted_relations.rows_affected());

    let deleted_tags = query!(
        "DELETE FROM tag WHERE id NOT IN (SELECT tag_id FROM image_tag) AND description = ANY($1);",
        &body.tags
    )
    .execute(&mut *tx)
    .await?;

    info!(deleted_tags = %deleted_tags.rows_affected());

    tx.commit().await?;

    Ok(StatusCode::OK)
}
