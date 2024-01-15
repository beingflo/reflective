use axum::{http::StatusCode, Extension, Json};
use serde::{Deserialize, Serialize};

use crate::State;

#[derive(Serialize, Deserialize)]
pub struct Data {
    date: String,
    data: i32,
}

pub async fn signup(Extension(state): Extension<State>) -> StatusCode {
    StatusCode::OK
}

pub async fn login(Extension(state): Extension<State>) -> StatusCode {
    StatusCode::OK
}
