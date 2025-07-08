//! Bridge status handlers

use crate::{
    error::{ApiError, Result},
    server::{ApiState, BridgeStatusResponse, ValidatorResponse, TransactionResponse},
};
use axum::{extract::Extension, Json};
use tracing::debug;

/// Get bridge status
pub async fn bridge_status(
    Extension(state): Extension<ApiState>,
) -> Result<Json<BridgeStatusResponse>> {
    debug!("Bridge status requested");

    // Get bridge statistics
    let bridge_stats = state.coordinator.get_stats().await
        .map_err(|e| ApiError::Relayer(e))?;

    // Mock data for demonstration
    let validators = vec![
        ValidatorResponse {
            id: "validator_0".to_string(),
            address: "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".to_string(),
            active: true,
            stake: "1000".to_string(),
            uptime: 99.5,
        },
        ValidatorResponse {
            id: "validator_1".to_string(),
            address: "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC".to_string(),
            active: true,
            stake: "1000".to_string(),
            uptime: 98.2,
        },
        ValidatorResponse {
            id: "validator_2".to_string(),
            address: "0x90F79bf6EB2c4f870365E785982E1f101E93b906".to_string(),
            active: true,
            stake: "1000".to_string(),
            uptime: 99.8,
        },
    ];

    let recent_transactions = vec![
        TransactionResponse {
            tx_hash: "0x1234567890abcdef".to_string(),
            chain: "ethereum".to_string(),
            status: "confirmed".to_string(),
            amount: "1000".to_string(),
            token: "0xA0b86a33E6441e6e80D0c4C34F4F6cA4C7C7B0c8".to_string(),
            user: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string(),
            block_number: 12345,
            timestamp: "2024-01-01T12:00:00Z".to_string(),
        },
    ];

    let response = BridgeStatusResponse {
        status: "operational".to_string(),
        ethereum_block: 12345,
        polkadot_block: 6789,
        validators,
        recent_transactions,
    };

    Ok(Json(response))
}
