//! Error types for threshold signature operations

use thiserror::Error;

/// Result type for threshold signature operations
pub type Result<T> = std::result::Result<T, ThresholdError>;

/// Errors that can occur during threshold signature operations
#[derive(Error, Debug, Clone)]
pub enum ThresholdError {
    /// Invalid threshold configuration
    #[error("Invalid threshold configuration: threshold {threshold} must be <= total validators {total}")]
    InvalidThreshold { threshold: u32, total: u32 },

    /// Insufficient signatures for threshold
    #[error("Insufficient signatures: required {required}, received {received}")]
    InsufficientSignatures { required: u32, received: u32 },

    /// Invalid key share
    #[error("Invalid key share: {reason}")]
    InvalidKeyShare { reason: String },

    /// Invalid signature
    #[error("Invalid signature: {reason}")]
    InvalidSignature { reason: String },

    /// Key generation failed
    #[error("Key generation failed: {reason}")]
    KeyGenerationFailed { reason: String },

    /// Signature aggregation failed
    #[error("Signature aggregation failed: {reason}")]
    AggregationFailed { reason: String },

    /// Verification failed
    #[error("Signature verification failed: {reason}")]
    VerificationFailed { reason: String },

    /// Invalid validator ID
    #[error("Invalid validator ID: {id}")]
    InvalidValidatorId { id: String },

    /// Duplicate partial signature
    #[error("Duplicate partial signature from validator {validator_id}")]
    DuplicateSignature { validator_id: String },

    /// Session not found
    #[error("Signing session not found: {session_id}")]
    SessionNotFound { session_id: String },

    /// Session expired
    #[error("Signing session expired: {session_id}")]
    SessionExpired { session_id: String },

    /// Cryptographic error
    #[error("Cryptographic error: {message}")]
    CryptographicError { message: String },

    /// Serialization error
    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    /// Network error during distributed operations
    #[error("Network error: {message}")]
    NetworkError { message: String },

    /// Timeout during operation
    #[error("Operation timed out: {operation}")]
    Timeout { operation: String },

    /// Generic error
    #[error("Threshold signature error: {message}")]
    Generic { message: String },
}

impl From<serde_json::Error> for ThresholdError {
    fn from(err: serde_json::Error) -> Self {
        ThresholdError::SerializationError {
            message: err.to_string(),
        }
    }
}

impl From<k256::elliptic_curve::Error> for ThresholdError {
    fn from(err: k256::elliptic_curve::Error) -> Self {
        ThresholdError::CryptographicError {
            message: err.to_string(),
        }
    }
}

impl From<k256::ecdsa::Error> for ThresholdError {
    fn from(err: k256::ecdsa::Error) -> Self {
        ThresholdError::CryptographicError {
            message: err.to_string(),
        }
    }
}
