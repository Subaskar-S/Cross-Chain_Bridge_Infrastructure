//! Configuration management for the relayer service

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Main configuration for the relayer service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayerConfig {
    /// Ethereum configuration
    pub ethereum: EthereumConfig,
    /// Polkadot configuration
    pub polkadot: PolkadotConfig,
    /// Threshold signature configuration
    pub threshold: ThresholdConfig,
    /// Database configuration
    pub database: DatabaseConfig,
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
    /// Validator configuration
    pub validator: ValidatorConfig,
}

/// Ethereum chain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumConfig {
    /// RPC endpoint URL
    pub rpc_url: String,
    /// WebSocket endpoint URL
    pub ws_url: String,
    /// Chain ID
    pub chain_id: u64,
    /// Bridge contract address
    pub bridge_contract: String,
    /// Block confirmation requirements
    pub confirmations: u64,
    /// Gas limit for transactions
    pub gas_limit: u64,
    /// Gas price in wei
    pub gas_price: u64,
    /// Private key for signing transactions (optional)
    pub private_key: Option<String>,
    /// Starting block for event monitoring
    pub start_block: Option<u64>,
}

/// Polkadot chain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolkadotConfig {
    /// WebSocket endpoint URL
    pub ws_url: String,
    /// Bridge pallet name
    pub pallet_name: String,
    /// Block confirmation requirements
    pub confirmations: u32,
    /// Account seed for signing transactions (optional)
    pub account_seed: Option<String>,
    /// Starting block for event monitoring
    pub start_block: Option<u32>,
}

/// Threshold signature configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    /// Signature scheme (schnorr or ecdsa)
    pub scheme: String,
    /// Number of signatures required (k)
    pub threshold: u32,
    /// Total number of validators (n)
    pub total_validators: u32,
    /// Key size in bits
    pub key_size: u32,
    /// Signature timeout in seconds
    pub signature_timeout: u64,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL
    pub url: String,
    /// Maximum number of connections
    pub max_connections: u32,
    /// Minimum number of connections
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    /// Query timeout in seconds
    pub query_timeout: u64,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Event polling interval in seconds
    pub poll_interval: u64,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Retry delay in seconds
    pub retry_delay: u64,
    /// Batch size for processing events
    pub batch_size: u32,
    /// Metrics port
    pub metrics_port: u16,
    /// Log level
    pub log_level: String,
}

/// Validator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// This validator's ID
    pub validator_id: String,
    /// Private key for threshold signatures
    pub private_key: Option<String>,
    /// List of other validators
    pub peers: Vec<ValidatorPeer>,
    /// Enable validator mode
    pub enabled: bool,
}

/// Validator peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorPeer {
    /// Validator ID
    pub id: String,
    /// Public key
    pub public_key: String,
    /// Network address
    pub address: String,
    /// Whether this peer is active
    pub active: bool,
}

impl RelayerConfig {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::RelayerError::Config {
                message: format!("Failed to read config file: {}", e),
            })?;

        let config: RelayerConfig = toml::from_str(&content)
            .map_err(|e| crate::RelayerError::Config {
                message: format!("Failed to parse config: {}", e),
            })?;

        config.validate()?;
        Ok(config)
    }

    /// Load configuration from environment variables and defaults
    pub fn from_env() -> crate::Result<Self> {
        let config = RelayerConfig {
            ethereum: EthereumConfig {
                rpc_url: std::env::var("ETHEREUM_RPC_URL")
                    .unwrap_or_else(|_| "http://localhost:8545".to_string()),
                ws_url: std::env::var("ETHEREUM_WS_URL")
                    .unwrap_or_else(|_| "ws://localhost:8545".to_string()),
                chain_id: std::env::var("ETHEREUM_CHAIN_ID")
                    .unwrap_or_else(|_| "1337".to_string())
                    .parse()
                    .unwrap_or(1337),
                bridge_contract: std::env::var("ETHEREUM_BRIDGE_CONTRACT")
                    .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string()),
                confirmations: std::env::var("ETHEREUM_CONFIRMATIONS")
                    .unwrap_or_else(|_| "12".to_string())
                    .parse()
                    .unwrap_or(12),
                gas_limit: std::env::var("ETHEREUM_GAS_LIMIT")
                    .unwrap_or_else(|_| "300000".to_string())
                    .parse()
                    .unwrap_or(300000),
                gas_price: std::env::var("ETHEREUM_GAS_PRICE")
                    .unwrap_or_else(|_| "20000000000".to_string())
                    .parse()
                    .unwrap_or(20000000000),
                private_key: std::env::var("ETHEREUM_PRIVATE_KEY").ok(),
                start_block: std::env::var("ETHEREUM_START_BLOCK")
                    .ok()
                    .and_then(|s| s.parse().ok()),
            },
            polkadot: PolkadotConfig {
                ws_url: std::env::var("POLKADOT_WS_URL")
                    .unwrap_or_else(|_| "ws://localhost:9944".to_string()),
                pallet_name: std::env::var("POLKADOT_PALLET_NAME")
                    .unwrap_or_else(|_| "bridge".to_string()),
                confirmations: std::env::var("POLKADOT_CONFIRMATIONS")
                    .unwrap_or_else(|_| "6".to_string())
                    .parse()
                    .unwrap_or(6),
                account_seed: std::env::var("POLKADOT_ACCOUNT_SEED").ok(),
                start_block: std::env::var("POLKADOT_START_BLOCK")
                    .ok()
                    .and_then(|s| s.parse().ok()),
            },
            threshold: ThresholdConfig {
                scheme: std::env::var("THRESHOLD_SCHEME")
                    .unwrap_or_else(|_| "ecdsa".to_string()),
                threshold: std::env::var("THRESHOLD_K")
                    .unwrap_or_else(|_| "2".to_string())
                    .parse()
                    .unwrap_or(2),
                total_validators: std::env::var("THRESHOLD_N")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()
                    .unwrap_or(3),
                key_size: std::env::var("THRESHOLD_KEY_SIZE")
                    .unwrap_or_else(|_| "256".to_string())
                    .parse()
                    .unwrap_or(256),
                signature_timeout: std::env::var("SIGNATURE_TIMEOUT")
                    .unwrap_or_else(|_| "300".to_string())
                    .parse()
                    .unwrap_or(300),
            },
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgresql://bridge_user:bridge_pass@localhost:5432/bridge".to_string()),
                max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                min_connections: std::env::var("DATABASE_MIN_CONNECTIONS")
                    .unwrap_or_else(|_| "1".to_string())
                    .parse()
                    .unwrap_or(1),
                connect_timeout: std::env::var("DATABASE_CONNECT_TIMEOUT")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
                query_timeout: std::env::var("DATABASE_QUERY_TIMEOUT")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()
                    .unwrap_or(60),
            },
            monitoring: MonitoringConfig {
                poll_interval: std::env::var("POLL_INTERVAL")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .unwrap_or(5),
                max_retries: std::env::var("MAX_RETRIES")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()
                    .unwrap_or(3),
                retry_delay: std::env::var("RETRY_DELAY")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                batch_size: std::env::var("BATCH_SIZE")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                metrics_port: std::env::var("METRICS_PORT")
                    .unwrap_or_else(|_| "9001".to_string())
                    .parse()
                    .unwrap_or(9001),
                log_level: std::env::var("LOG_LEVEL")
                    .unwrap_or_else(|_| "info".to_string()),
            },
            validator: ValidatorConfig {
                validator_id: std::env::var("VALIDATOR_ID")
                    .unwrap_or_else(|_| "validator_0".to_string()),
                private_key: std::env::var("VALIDATOR_PRIVATE_KEY").ok(),
                peers: vec![], // Would be loaded from a separate config
                enabled: std::env::var("VALIDATOR_ENABLED")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
            },
        };

        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> crate::Result<()> {
        // Validate Ethereum config
        if self.ethereum.rpc_url.is_empty() {
            return Err(crate::RelayerError::Config {
                message: "Ethereum RPC URL cannot be empty".to_string(),
            });
        }

        if self.ethereum.bridge_contract.is_empty() {
            return Err(crate::RelayerError::Config {
                message: "Ethereum bridge contract address cannot be empty".to_string(),
            });
        }

        // Validate Polkadot config
        if self.polkadot.ws_url.is_empty() {
            return Err(crate::RelayerError::Config {
                message: "Polkadot WebSocket URL cannot be empty".to_string(),
            });
        }

        // Validate threshold config
        if self.threshold.threshold == 0 {
            return Err(crate::RelayerError::Config {
                message: "Threshold cannot be zero".to_string(),
            });
        }

        if self.threshold.threshold > self.threshold.total_validators {
            return Err(crate::RelayerError::Config {
                message: "Threshold cannot be greater than total validators".to_string(),
            });
        }

        // Validate database config
        if self.database.url.is_empty() {
            return Err(crate::RelayerError::Config {
                message: "Database URL cannot be empty".to_string(),
            });
        }

        Ok(())
    }
}

impl Default for RelayerConfig {
    fn default() -> Self {
        Self::from_env().unwrap_or_else(|_| RelayerConfig {
            ethereum: EthereumConfig {
                rpc_url: "http://localhost:8545".to_string(),
                ws_url: "ws://localhost:8545".to_string(),
                chain_id: 1337,
                bridge_contract: "0x0000000000000000000000000000000000000000".to_string(),
                confirmations: 12,
                gas_limit: 300000,
                gas_price: 20000000000,
                private_key: None,
                start_block: None,
            },
            polkadot: PolkadotConfig {
                ws_url: "ws://localhost:9944".to_string(),
                pallet_name: "bridge".to_string(),
                confirmations: 6,
                account_seed: None,
                start_block: None,
            },
            threshold: ThresholdConfig {
                scheme: "ecdsa".to_string(),
                threshold: 2,
                total_validators: 3,
                key_size: 256,
                signature_timeout: 300,
            },
            database: DatabaseConfig {
                url: "postgresql://bridge_user:bridge_pass@localhost:5432/bridge".to_string(),
                max_connections: 10,
                min_connections: 1,
                connect_timeout: 30,
                query_timeout: 60,
            },
            monitoring: MonitoringConfig {
                poll_interval: 5,
                max_retries: 3,
                retry_delay: 10,
                batch_size: 10,
                metrics_port: 9001,
                log_level: "info".to_string(),
            },
            validator: ValidatorConfig {
                validator_id: "validator_0".to_string(),
                private_key: None,
                peers: vec![],
                enabled: false,
            },
        })
    }
}
