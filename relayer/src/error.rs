//! Error types for the relayer service

use thiserror::Error;

/// Result type for relayer operations
pub type Result<T> = std::result::Result<T, RelayerError>;

/// Errors that can occur in the relayer service
#[derive(Error, Debug)]
pub enum RelayerError {
    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Ethereum error: {message}")]
    Ethereum { message: String },

    #[error("Polkadot error: {message}")]
    Polkadot { message: String },

    #[error("Threshold signature error: {0}")]
    ThresholdSignature(#[from] threshold::ThresholdError),

    #[error("Database error: {message}")]
    Database { message: String },

    #[error("Network error: {message}")]
    Network { message: String },

    #[error("Generic error: {message}")]
    Generic { message: String },
}
