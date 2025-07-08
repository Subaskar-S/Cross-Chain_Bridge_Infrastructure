//! Cross-chain bridge relayer binary

use relayer::{BridgeCoordinator, config::RelayerConfig};
use tracing::{info, error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::env;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging();

    info!("Starting Cross-Chain Bridge Relayer");

    // Load configuration
    let config = load_config().await?;
    info!("Configuration loaded successfully");

    // Create and start bridge coordinator
    let mut coordinator = BridgeCoordinator::new(config).await?;
    info!("Bridge coordinator initialized");

    // Setup graceful shutdown
    let shutdown_signal = setup_shutdown_signal();

    // Start the coordinator
    tokio::select! {
        result = coordinator.start() => {
            if let Err(e) = result {
                error!("Bridge coordinator failed: {}", e);
                return Err(e.into());
            }
        }
        _ = shutdown_signal => {
            info!("Shutdown signal received");
        }
    }

    // Graceful shutdown
    info!("Shutting down bridge coordinator...");
    if let Err(e) = coordinator.shutdown().await {
        error!("Error during shutdown: {}", e);
    }

    info!("Bridge relayer shutdown complete");
    Ok(())
}

/// Initialize logging based on environment
fn init_logging() {
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&log_level));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Load configuration from file or environment
async fn load_config() -> Result<RelayerConfig, Box<dyn std::error::Error>> {
    // Try to load from config file first
    if let Ok(config_path) = env::var("CONFIG_FILE") {
        info!("Loading configuration from file: {}", config_path);
        match RelayerConfig::from_file(&config_path) {
            Ok(config) => return Ok(config),
            Err(e) => {
                warn!("Failed to load config from file: {}", e);
                info!("Falling back to environment variables");
            }
        }
    }

    // Fall back to environment variables
    info!("Loading configuration from environment variables");
    let config = RelayerConfig::from_env()?;
    
    // Log configuration summary (without sensitive data)
    info!("Configuration summary:");
    info!("  Ethereum RPC: {}", config.ethereum.rpc_url);
    info!("  Polkadot WS: {}", config.polkadot.ws_url);
    info!("  Threshold: {}/{}", config.threshold.threshold, config.threshold.total_validators);
    info!("  Validator mode: {}", config.validator.enabled);
    
    Ok(config)
}

/// Setup graceful shutdown signal handling
async fn setup_shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            info!("Received terminate signal");
        },
    }
}
