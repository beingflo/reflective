use crate::error::AppError;
use axum::{http::StatusCode, Extension, Json};
use serde::{Deserialize, Serialize};

use crate::State;

#[derive(Serialize, Deserialize)]
pub struct S3Data {
    endpoint: String,
    region: String,
    access_key: String,
    secret_key: String,
}

pub async fn update_config(
    Extension(state): Extension<State>,
    Json(data): Json<S3Data>,
) -> Result<StatusCode, AppError> {
    let connection = state.conn.lock().await;

    //TODO read user from cookie / session store
    let user = "test";

    let mut stmt = connection
        .prepare("UPDATE users SET endpoint = ?1, region = ?2, access_key = ?3, secret_key = ?4 WHERE username = ?5")
        .unwrap();

    stmt.execute([
        data.endpoint,
        data.region,
        data.access_key,
        data.secret_key,
        user.into(),
    ])?;

    Ok(StatusCode::OK)
}
