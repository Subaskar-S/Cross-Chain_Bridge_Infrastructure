//! API routes definition

use crate::handlers;
use axum::{
    routing::{get, post},
    Router,
};

/// Create API routes
pub fn create_api_routes() -> Router {
    Router::new()
        // Health and status endpoints
        .route("/health", get(handlers::health::health_check))
        .route("/status", get(handlers::status::bridge_status))
        .route("/stats", get(handlers::stats::bridge_stats))
        
        // Transaction endpoints
        .route("/transactions", get(handlers::transactions::list_transactions))
        .route("/transactions/:tx_hash", get(handlers::transactions::get_transaction))
        
        // Validator endpoints
        .route("/validators", get(handlers::validators::list_validators))
        .route("/validators/:validator_id", get(handlers::validators::get_validator))
        
        // Bridge operation endpoints
        .route("/bridge/lock", post(handlers::bridge::initiate_lock))
        .route("/bridge/unlock", post(handlers::bridge::initiate_unlock))
        .route("/bridge/mint", post(handlers::bridge::mint_tokens))
        .route("/bridge/burn", post(handlers::bridge::burn_tokens))
        
        // Token endpoints
        .route("/tokens", get(handlers::tokens::list_tokens))
        .route("/tokens/:token_address", get(handlers::tokens::get_token))
        
        // Block endpoints
        .route("/blocks/ethereum/latest", get(handlers::blocks::latest_ethereum_block))
        .route("/blocks/polkadot/latest", get(handlers::blocks::latest_polkadot_block))
        
        // Event endpoints
        .route("/events", get(handlers::events::list_events))
        .route("/events/ethereum", get(handlers::events::ethereum_events))
        .route("/events/polkadot", get(handlers::events::polkadot_events))
}

/// Create WebSocket routes
pub fn create_websocket_routes() -> Router {
    Router::new()
        .route("/ws", get(handlers::websocket::websocket_handler))
        .route("/ws/events", get(handlers::websocket::events_websocket))
        .route("/ws/stats", get(handlers::websocket::stats_websocket))
}

/// Create metrics routes
pub fn create_metrics_routes() -> Router {
    Router::new()
        .route("/metrics", get(handlers::metrics::prometheus_metrics))
        .route("/metrics/bridge", get(handlers::metrics::bridge_metrics))
}
