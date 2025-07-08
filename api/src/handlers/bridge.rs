//! Bridge operation handlers

use crate::{error::Result, server::ApiState};
use axum::{extract::Extension, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LockRequest {
    pub token: String,
    pub amount: String,
    pub polkadot_address: String,
}

#[derive(Serialize)]
pub struct LockResponse {
    pub tx_hash: String,
    pub status: String,
}

pub async fn initiate_lock(
    Extension(_state): Extension<ApiState>,
    Json(_request): Json<LockRequest>,
) -> Result<Json<LockResponse>> {
    let response = LockResponse {
        tx_hash: "0x1234567890abcdef".to_string(),
        status: "pending".to_string(),
    };
    Ok(Json(response))
}

pub async fn initiate_unlock(
    Extension(_state): Extension<ApiState>,
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"status": "pending"})))
}

pub async fn mint_tokens(
    Extension(_state): Extension<ApiState>,
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"status": "pending"})))
}

pub async fn burn_tokens(
    Extension(_state): Extension<ApiState>,
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"status": "pending"})))
}
