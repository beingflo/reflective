use crate::{auth::AuthenticatedUser, error::AppError};
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Serialize, Deserialize)]
pub struct S3Data {
    pub bucket: String,
    pub endpoint: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
}

pub async fn update_config(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(data): Json<S3Data>,
) -> Result<StatusCode, AppError> {
    let connection = state.conn.lock().await;

    let mut stmt = connection
        .prepare("UPDATE users SET config = ?1 WHERE id = ?2")
        .unwrap();

    stmt.execute([serde_json::to_string(&data)?, user.id.to_string()])?;

    Ok(StatusCode::OK)
}
