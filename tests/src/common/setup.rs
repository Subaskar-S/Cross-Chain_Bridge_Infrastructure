//! Test setup utilities

use super::{TestConfig, TestResult};
use relayer::{BridgeCoordinator, config::RelayerConfig};
use threshold::{SimpleThresholdManager, ThresholdConfig};
use std::sync::Arc;

/// Setup a test bridge coordinator (mock version for testing without database)
pub async fn setup_test_coordinator() -> TestResult<Arc<BridgeCoordinator>> {
    // For testing, we'll create a mock coordinator that doesn't require database connection
    // In a real test environment, you would set up a test database

    // For now, return an error that indicates database setup is needed
    Err("Database setup required for coordinator tests. Use setup_mock_coordinator() for unit tests.".into())
}

/// Setup a mock bridge coordinator for unit tests (without database)
pub async fn setup_mock_coordinator() -> TestResult<MockBridgeStats> {
    Ok(MockBridgeStats {
        ethereum_processed_txs: 0,
        polkadot_processed_txs: 0,
        pending_signatures: 0,
        active_validators: 3,
    })
}

/// Mock bridge statistics for testing
#[derive(Debug, Clone)]
pub struct MockBridgeStats {
    pub ethereum_processed_txs: u64,
    pub polkadot_processed_txs: u64,
    pub pending_signatures: u64,
    pub active_validators: u64,
}

/// Create test relayer configuration
pub fn create_test_relayer_config() -> RelayerConfig {
    RelayerConfig {
        ethereum: relayer::config::EthereumConfig {
            rpc_url: "http://localhost:8545".to_string(),
            ws_url: "ws://localhost:8545".to_string(),
            chain_id: 1337,
            bridge_contract: "0x0000000000000000000000000000000000000000".to_string(),
            confirmations: 1, // Fast confirmations for testing
            gas_limit: 300000,
            gas_price: 20000000000,
            private_key: Some("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()),
            start_block: Some(0),
        },
        polkadot: relayer::config::PolkadotConfig {
            ws_url: "ws://localhost:9944".to_string(),
            pallet_name: "bridge".to_string(),
            confirmations: 1, // Fast confirmations for testing
            account_seed: Some("//Alice".to_string()),
            start_block: Some(0),
        },
        threshold: relayer::config::ThresholdConfig {
            scheme: "ecdsa".to_string(),
            threshold: 2,
            total_validators: 3,
            key_size: 256,
            signature_timeout: 60, // Shorter timeout for testing
        },
        database: relayer::config::DatabaseConfig {
            url: "sqlite::memory:".to_string(), // Use in-memory SQLite for testing
            max_connections: 5,
            min_connections: 1,
            connect_timeout: 10,
            query_timeout: 30,
        },
        monitoring: relayer::config::MonitoringConfig {
            poll_interval: 1, // Fast polling for testing
            max_retries: 3,
            retry_delay: 1,
            batch_size: 10,
            metrics_port: 9002,
            log_level: "debug".to_string(),
        },
        validator: relayer::config::ValidatorConfig {
            validator_id: "test_validator".to_string(),
            private_key: Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string()),
            peers: vec![],
            enabled: true,
        },
    }
}

/// Setup test threshold manager
pub async fn setup_test_threshold_manager() -> TestResult<Arc<SimpleThresholdManager>> {
    let config = ThresholdConfig::new(2, 3, 256)?;
    let manager = SimpleThresholdManager::new(config)?;
    Ok(Arc::new(manager))
}

/// Setup test validators
pub async fn setup_test_validators() -> TestResult<Vec<String>> {
    Ok(vec![
        "test_validator_0".to_string(),
        "test_validator_1".to_string(),
        "test_validator_2".to_string(),
    ])
}

/// Initialize test logging
pub fn init_test_logging() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    
    let _ = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("debug"))
        .with(tracing_subscriber::fmt::layer().with_test_writer())
        .try_init();
}

/// Wait for services to be ready
pub async fn wait_for_services_ready() -> TestResult<()> {
    // In a real test environment, this would:
    // - Wait for Ethereum node to be ready
    // - Wait for Polkadot node to be ready
    // - Wait for database to be ready
    // - Deploy test contracts
    
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    Ok(())
}
