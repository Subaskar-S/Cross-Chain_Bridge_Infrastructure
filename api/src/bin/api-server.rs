//! API server binary

use api::{ApiServer, server::ApiConfig};
use relayer::{BridgeCoordinator, config::RelayerConfig};
use std::sync::Arc;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging();

    info!("Starting Cross-Chain Bridge API Server");

    // Load relayer configuration
    let relayer_config = RelayerConfig::from_env()?;
    
    // Create bridge coordinator (without starting it)
    let coordinator = Arc::new(BridgeCoordinator::new(relayer_config).await?);

    // Create API configuration
    let api_config = ApiConfig {
        host: std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
        port: std::env::var("API_PORT")
            .unwrap_or_else(|_| "3001".to_string())
            .parse()
            .unwrap_or(3001),
        cors_origins: vec!["http://localhost:3000".to_string()],
        enable_metrics: true,
        metrics_path: "/metrics".to_string(),
    };

    // Create and start API server
    let api_server = ApiServer::new(api_config, coordinator);
    
    info!("API server starting...");
    if let Err(e) = api_server.start().await {
        error!("API server failed: {}", e);
        return Err(e.into());
    }

    Ok(())
}

/// Initialize logging
fn init_logging() {
    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&log_level));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
