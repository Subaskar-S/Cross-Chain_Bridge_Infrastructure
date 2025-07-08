//! Bridge statistics handlers

use crate::{
    error::{ApiError, Result},
    server::{ApiState, BridgeStatsResponse},
};
use axum::{extract::Extension, Json};
use tracing::debug;

/// Get bridge statistics
pub async fn bridge_stats(
    Extension(state): Extension<ApiState>,
) -> Result<Json<BridgeStatsResponse>> {
    debug!("Bridge stats requested");

    let bridge_stats = state.coordinator.get_stats().await
        .map_err(|e| ApiError::Relayer(e))?;

    let response = BridgeStatsResponse {
        ethereum_processed_txs: bridge_stats.ethereum_processed_txs,
        polkadot_processed_txs: bridge_stats.polkadot_processed_txs,
        pending_signatures: bridge_stats.pending_signatures,
        active_validators: bridge_stats.active_validators,
    };

    Ok(Json(response))
}
