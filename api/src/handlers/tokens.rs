//! Token handlers

use crate::{error::Result, server::ApiState};
use axum::{extract::{Extension, Path}, Json};

pub async fn list_tokens(
    Extension(_state): Extension<ApiState>,
) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"tokens": []})))
}

pub async fn get_token(
    Extension(_state): Extension<ApiState>,
    Path(_token_address): Path<String>,
) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"token": {}})))
}
