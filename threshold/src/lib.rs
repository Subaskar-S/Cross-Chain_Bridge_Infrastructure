//! Threshold Signature Library for Cross-Chain Bridge
//!
//! This library implements threshold signature schemes (k-of-n) for validator consensus
//! in the cross-chain bridge. It supports both Schnorr and ECDSA signature schemes.

pub mod error;
pub mod types;
pub mod utils;
pub mod simple;

pub use error::{ThresholdError, Result};
pub use types::{
    ValidatorId, KeyShare, PublicKeyShare, Signature, ThresholdConfig,
    SigningSession, PartialSignature, AggregatedSignature
};
pub use simple::SimpleThresholdManager;

// Re-export the simple threshold manager as the main interface
pub use simple::SimpleThresholdManager as ThresholdManager;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_threshold_manager_creation() {
        let config = ThresholdConfig {
            threshold: 2,
            total_validators: 3,
            key_size: 256,
        };

        let manager = ThresholdManager::new(config).unwrap();
        assert_eq!(manager.config().threshold, 2);
        assert_eq!(manager.config().total_validators, 3);
    }

    #[tokio::test]
    async fn test_signing_session_creation() {
        let config = ThresholdConfig {
            threshold: 2,
            total_validators: 3,
            key_size: 256,
        };

        let manager = ThresholdManager::new(config).unwrap();
        let message = b"test message";
        let session_id = "test_session".to_string();

        let session = manager
            .create_signing_session(message, session_id.clone())
            .await
            .unwrap();

        assert_eq!(session.id, session_id);
        assert_eq!(session.message, message);
        assert_eq!(session.threshold, 2);
        assert_eq!(session.total_validators, 3);
    }
}
