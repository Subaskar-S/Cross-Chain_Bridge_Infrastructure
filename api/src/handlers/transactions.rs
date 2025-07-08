//! Transaction handlers

use crate::{
    error::{ApiError, Result},
    server::{ApiState, TransactionResponse, PaginationParams, TransactionFilters},
};
use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use tracing::debug;

/// List transactions with pagination and filtering
pub async fn list_transactions(
    Extension(_state): Extension<ApiState>,
    Query(pagination): Query<PaginationParams>,
    Query(_filters): Query<TransactionFilters>,
) -> Result<Json<Vec<TransactionResponse>>> {
    debug!("List transactions requested with pagination: {:?}", pagination);

    // Mock response for demonstration
    let transactions = vec![
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

    Ok(Json(transactions))
}

/// Get a specific transaction by hash
pub async fn get_transaction(
    Extension(_state): Extension<ApiState>,
    Path(tx_hash): Path<String>,
) -> Result<Json<TransactionResponse>> {
    debug!("Get transaction requested: {}", tx_hash);

    // Mock response for demonstration
    let transaction = TransactionResponse {
        tx_hash: tx_hash.clone(),
        chain: "ethereum".to_string(),
        status: "confirmed".to_string(),
        amount: "1000".to_string(),
        token: "0xA0b86a33E6441e6e80D0c4C34F4F6cA4C7C7B0c8".to_string(),
        user: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string(),
        block_number: 12345,
        timestamp: "2024-01-01T12:00:00Z".to_string(),
    };

    Ok(Json(transaction))
}
