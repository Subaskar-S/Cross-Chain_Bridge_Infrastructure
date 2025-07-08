//! Health check handlers

use crate::{
    error::{ApiError, Result},
    server::{ApiState, HealthResponse, BridgeStatsResponse},
};
use axum::{extract::Extension, Json};
use tracing::debug;

/// Health check endpoint
pub async fn health_check(
    Extension(state): Extension<ApiState>,
) -> Result<Json<HealthResponse>> {
    debug!("Health check requested");

    // Get bridge statistics
    let bridge_stats = state.coordinator.get_stats().await
        .map_err(|e| ApiError::Relayer(e))?;

    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: get_uptime_seconds(),
        bridge_stats: BridgeStatsResponse {
            ethereum_processed_txs: bridge_stats.ethereum_processed_txs,
            polkadot_processed_txs: bridge_stats.polkadot_processed_txs,
            pending_signatures: bridge_stats.pending_signatures,
            active_validators: bridge_stats.active_validators,
        },
    };

    Ok(Json(response))
}

/// Get uptime in seconds (simplified implementation)
fn get_uptime_seconds() -> u64 {
    // In a real implementation, this would track actual uptime
    // For now, return a mock value
    3600 // 1 hour
}
