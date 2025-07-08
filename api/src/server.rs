//! API server implementation

use crate::{
    error::{ApiError, Result},
    handlers,
    routes,
    middleware,
    websocket,
};
use axum::{
    extract::Extension,
    http::{header, Method},
    Router,
};
use relayer::BridgeCoordinator;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{info, error};

/// Configuration for the API server
#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub enable_metrics: bool,
    pub metrics_path: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3001,
            cors_origins: vec!["http://localhost:3000".to_string()],
            enable_metrics: true,
            metrics_path: "/metrics".to_string(),
        }
    }
}

/// API server state
#[derive(Clone)]
pub struct ApiState {
    pub coordinator: Arc<BridgeCoordinator>,
}

/// Main API server
pub struct ApiServer {
    config: ApiConfig,
    state: ApiState,
}

impl ApiServer {
    /// Create a new API server
    pub fn new(config: ApiConfig, coordinator: Arc<BridgeCoordinator>) -> Self {
        let state = ApiState { coordinator };

        Self { config, state }
    }

    /// Start the API server
    pub async fn start(&self) -> Result<()> {
        let app = self.create_app().await?;
        let addr = format!("{}:{}", self.config.host, self.config.port);

        info!("Starting API server on {}", addr);

        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| ApiError::Internal {
                message: format!("Failed to bind to {}: {}", addr, e),
            })?;

        axum::serve(listener, app)
            .await
            .map_err(|e| ApiError::Internal {
                message: format!("Server error: {}", e),
            })?;

        Ok(())
    }

    /// Create the Axum application
    async fn create_app(&self) -> Result<Router> {
        // Create CORS layer
        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
            .allow_origin(Any); // In production, use specific origins

        // Create middleware stack
        let middleware = ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(cors)
            .layer(middleware::request_id::RequestIdLayer::new())
            .layer(middleware::auth::AuthLayer::new());

        // Create routes
        let api_routes = routes::create_api_routes();
        let websocket_routes = routes::create_websocket_routes();

        // Combine all routes
        let app = Router::new()
            .merge(api_routes)
            .merge(websocket_routes)
            .layer(middleware)
            .layer(Extension(self.state.clone()));

        // Add metrics endpoint if enabled
        if self.config.enable_metrics {
            let metrics_routes = routes::create_metrics_routes();
            return Ok(app.merge(metrics_routes));
        }

        Ok(app)
    }
}

/// Health check response
#[derive(serde::Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime: u64,
    pub bridge_stats: BridgeStatsResponse,
}

/// Bridge statistics response
#[derive(serde::Serialize, serde::Deserialize)]
pub struct BridgeStatsResponse {
    pub ethereum_processed_txs: u64,
    pub polkadot_processed_txs: u64,
    pub pending_signatures: u64,
    pub active_validators: u64,
}

/// Transaction response
#[derive(serde::Serialize)]
pub struct TransactionResponse {
    pub tx_hash: String,
    pub chain: String,
    pub status: String,
    pub amount: String,
    pub token: String,
    pub user: String,
    pub block_number: u64,
    pub timestamp: String,
}

/// Validator response
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ValidatorResponse {
    pub id: String,
    pub address: String,
    pub active: bool,
    pub stake: String,
    pub uptime: f64,
}

/// Bridge status response
#[derive(serde::Serialize)]
pub struct BridgeStatusResponse {
    pub status: String,
    pub ethereum_block: u64,
    pub polkadot_block: u32,
    pub validators: Vec<ValidatorResponse>,
    pub recent_transactions: Vec<TransactionResponse>,
}

/// Error response
#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub code: u16,
}

impl From<ApiError> for ErrorResponse {
    fn from(error: ApiError) -> Self {
        match error {
            ApiError::NotFound { resource } => ErrorResponse {
                error: "Not Found".to_string(),
                message: format!("Resource not found: {}", resource),
                code: 404,
            },
            ApiError::Validation { message } => ErrorResponse {
                error: "Validation Error".to_string(),
                message,
                code: 400,
            },
            ApiError::Internal { message } => ErrorResponse {
                error: "Internal Server Error".to_string(),
                message,
                code: 500,
            },
            ApiError::Config { message } => ErrorResponse {
                error: "Configuration Error".to_string(),
                message,
                code: 500,
            },
            ApiError::Relayer(e) => ErrorResponse {
                error: "Relayer Error".to_string(),
                message: e.to_string(),
                code: 500,
            },
            ApiError::ThresholdSignature(e) => ErrorResponse {
                error: "Threshold Signature Error".to_string(),
                message: e.to_string(),
                code: 500,
            },
        }
    }
}

/// Pagination parameters
#[derive(Debug, serde::Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(20),
        }
    }
}

/// Filter parameters for transactions
#[derive(serde::Deserialize)]
pub struct TransactionFilters {
    pub chain: Option<String>,
    pub status: Option<String>,
    pub user: Option<String>,
    pub token: Option<String>,
    pub from_block: Option<u64>,
    pub to_block: Option<u64>,
}

/// WebSocket message types
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "bridge_event")]
    BridgeEvent {
        event_type: String,
        data: serde_json::Value,
    },
    #[serde(rename = "stats_update")]
    StatsUpdate {
        stats: BridgeStatsResponse,
    },
    #[serde(rename = "validator_update")]
    ValidatorUpdate {
        validator: ValidatorResponse,
    },
    #[serde(rename = "error")]
    Error {
        message: String,
    },
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "pong")]
    Pong,
}
