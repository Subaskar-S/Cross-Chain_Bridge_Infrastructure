//! Type definitions for threshold signature operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
// Removed zeroize imports as they're not used in simplified version

/// Unique identifier for a validator
pub type ValidatorId = String;

/// Configuration for threshold signature scheme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    /// Number of signatures required (k in k-of-n)
    pub threshold: u32,
    /// Total number of validators (n in k-of-n)
    pub total_validators: u32,
    /// Key size in bits
    pub key_size: u32,
}

impl ThresholdConfig {
    /// Create a new threshold configuration
    pub fn new(threshold: u32, total_validators: u32, key_size: u32) -> crate::Result<Self> {
        if threshold > total_validators {
            return Err(crate::ThresholdError::InvalidThreshold {
                threshold,
                total: total_validators,
            });
        }
        if threshold == 0 {
            return Err(crate::ThresholdError::InvalidThreshold {
                threshold,
                total: total_validators,
            });
        }

        Ok(Self {
            threshold,
            total_validators,
            key_size,
        })
    }

    /// Validate the configuration
    pub fn validate(&self) -> crate::Result<()> {
        if self.threshold > self.total_validators || self.threshold == 0 {
            return Err(crate::ThresholdError::InvalidThreshold {
                threshold: self.threshold,
                total: self.total_validators,
            });
        }
        Ok(())
    }
}

/// Private key share for a validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyShare {
    /// Validator ID
    pub validator_id: ValidatorId,
    /// Private key share
    pub private_share: Vec<u8>,
    /// Public key share
    pub public_share: Vec<u8>,
    /// Polynomial coefficients for verification
    pub coefficients: Vec<Vec<u8>>,
    /// Threshold configuration
    pub config: ThresholdConfig,
}

/// Public key share for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyShare {
    /// Validator ID
    pub validator_id: ValidatorId,
    /// Public key share
    pub public_share: Vec<u8>,
    /// Verification key
    pub verification_key: Vec<u8>,
}

/// Partial signature from a single validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialSignature {
    /// Validator ID who created this signature
    pub validator_id: ValidatorId,
    /// Signature data
    pub signature: Vec<u8>,
    /// Nonce or commitment (scheme-specific)
    pub commitment: Option<Vec<u8>>,
    /// Timestamp when signature was created
    pub timestamp: SystemTime,
}

/// Aggregated signature from multiple validators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedSignature {
    /// Final signature
    pub signature: Vec<u8>,
    /// List of validator IDs that contributed
    pub signers: Vec<ValidatorId>,
    /// Aggregated public key
    pub public_key: Vec<u8>,
    /// Signature scheme used
    pub scheme: String,
    /// Timestamp when aggregation completed
    pub timestamp: SystemTime,
}

/// Generic signature type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    /// Signature bytes
    pub data: Vec<u8>,
    /// Recovery ID (for ECDSA)
    pub recovery_id: Option<u8>,
    /// Signature scheme
    pub scheme: String,
}

/// Signing session state
#[derive(Debug, Clone)]
pub struct SigningSession {
    /// Unique session identifier
    pub id: String,
    /// Message being signed
    pub message: Vec<u8>,
    /// Collected partial signatures
    pub partial_signatures: HashMap<ValidatorId, PartialSignature>,
    /// Required threshold
    pub threshold: u32,
    /// Total number of validators
    pub total_validators: u32,
    /// Session creation time
    pub created_at: SystemTime,
}

impl SigningSession {
    /// Check if session has expired
    pub fn is_expired(&self, timeout_secs: u64) -> bool {
        if let Ok(elapsed) = self.created_at.elapsed() {
            elapsed.as_secs() > timeout_secs
        } else {
            true // If we can't determine elapsed time, consider expired
        }
    }

    /// Get the number of signatures collected
    pub fn signature_count(&self) -> usize {
        self.partial_signatures.len()
    }

    /// Check if threshold is met
    pub fn is_threshold_met(&self) -> bool {
        self.signature_count() >= self.threshold as usize
    }

    /// Get list of signers
    pub fn get_signers(&self) -> Vec<ValidatorId> {
        self.partial_signatures.keys().cloned().collect()
    }
}

/// Distributed Key Generation (DKG) parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DkgParams {
    /// Threshold configuration
    pub config: ThresholdConfig,
    /// Participant validator IDs
    pub participants: Vec<ValidatorId>,
    /// Round number for the DKG protocol
    pub round: u32,
    /// Session identifier
    pub session_id: String,
}

/// DKG commitment for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DkgCommitment {
    /// Validator ID
    pub validator_id: ValidatorId,
    /// Commitment values
    pub commitments: Vec<Vec<u8>>,
    /// Proof of knowledge
    pub proof: Vec<u8>,
}

/// DKG share for distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DkgShare {
    /// Sender validator ID
    pub from: ValidatorId,
    /// Receiver validator ID
    pub to: ValidatorId,
    /// Encrypted share
    pub share: Vec<u8>,
    /// Verification data
    pub verification: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_config_validation() {
        // Valid configuration
        let config = ThresholdConfig::new(2, 3, 256).unwrap();
        assert_eq!(config.threshold, 2);
        assert_eq!(config.total_validators, 3);

        // Invalid: threshold > total
        assert!(ThresholdConfig::new(4, 3, 256).is_err());

        // Invalid: threshold = 0
        assert!(ThresholdConfig::new(0, 3, 256).is_err());
    }

    #[test]
    fn test_signing_session() {
        let mut session = SigningSession {
            id: "test".to_string(),
            message: b"test message".to_vec(),
            partial_signatures: HashMap::new(),
            threshold: 2,
            total_validators: 3,
            created_at: SystemTime::now(),
        };

        assert!(!session.is_threshold_met());
        assert_eq!(session.signature_count(), 0);

        // Add a partial signature
        let partial_sig = PartialSignature {
            validator_id: "validator1".to_string(),
            signature: vec![1, 2, 3],
            commitment: None,
            timestamp: SystemTime::now(),
        };

        session.partial_signatures.insert("validator1".to_string(), partial_sig);
        assert_eq!(session.signature_count(), 1);
        assert!(!session.is_threshold_met());

        // Add another partial signature
        let partial_sig2 = PartialSignature {
            validator_id: "validator2".to_string(),
            signature: vec![4, 5, 6],
            commitment: None,
            timestamp: SystemTime::now(),
        };

        session.partial_signatures.insert("validator2".to_string(), partial_sig2);
        assert_eq!(session.signature_count(), 2);
        assert!(session.is_threshold_met());
    }
}
