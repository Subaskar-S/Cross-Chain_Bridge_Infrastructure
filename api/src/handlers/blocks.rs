//! Block handlers

use crate::{error::Result, server::ApiState};
use axum::{extract::Extension, Json};

pub async fn latest_ethereum_block(
    Extension(_state): Extension<ApiState>,
) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"block_number": 12345})))
}

pub async fn latest_polkadot_block(
    Extension(_state): Extension<ApiState>,
) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"block_number": 6789})))
}
