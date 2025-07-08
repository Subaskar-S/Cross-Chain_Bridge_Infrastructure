//! Event handlers

use crate::{error::Result, server::ApiState};
use axum::{extract::Extension, Json};

pub async fn list_events(
    Extension(_state): Extension<ApiState>,
) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"events": []})))
}

pub async fn ethereum_events(
    Extension(_state): Extension<ApiState>,
) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"events": []})))
}

pub async fn polkadot_events(
    Extension(_state): Extension<ApiState>,
) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"events": []})))
}
