//! Validator handlers

use crate::{
    error::{ApiError, Result},
    server::{ApiState, ValidatorResponse},
};
use axum::{
    extract::{Extension, Path},
    Json,
};

pub async fn list_validators(
    Extension(_state): Extension<ApiState>,
) -> Result<Json<Vec<ValidatorResponse>>> {
    let validators = vec![
        ValidatorResponse {
            id: "validator_0".to_string(),
            address: "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".to_string(),
            active: true,
            stake: "1000".to_string(),
            uptime: 99.5,
        },
    ];
    Ok(Json(validators))
}

pub async fn get_validator(
    Extension(_state): Extension<ApiState>,
    Path(validator_id): Path<String>,
) -> Result<Json<ValidatorResponse>> {
    let validator = ValidatorResponse {
        id: validator_id,
        address: "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".to_string(),
        active: true,
        stake: "1000".to_string(),
        uptime: 99.5,
    };
    Ok(Json(validator))
}
