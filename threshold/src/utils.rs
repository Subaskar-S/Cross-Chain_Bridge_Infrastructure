//! Utility functions for threshold signatures

use crate::{
    error::{Result, ThresholdError},
    types::{KeyShare, PublicKeyShare, ThresholdConfig, ValidatorId},
};
use rand::{CryptoRng, RngCore};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Generate a cryptographically secure random session ID
pub fn generate_session_id() -> String {
    use rand::Rng;
    let mut rng = rand::rngs::OsRng;
    let random_bytes: [u8; 16] = rng.gen();
    hex::encode(random_bytes)
}

/// Validate threshold configuration
pub fn validate_threshold_config(config: &ThresholdConfig) -> Result<()> {
    if config.threshold == 0 {
        return Err(ThresholdError::InvalidThreshold {
            threshold: config.threshold,
            total: config.total_validators,
        });
    }

    if config.threshold > config.total_validators {
        return Err(ThresholdError::InvalidThreshold {
            threshold: config.threshold,
            total: config.total_validators,
        });
    }

    if config.total_validators == 0 {
        return Err(ThresholdError::InvalidThreshold {
            threshold: config.threshold,
            total: config.total_validators,
        });
    }

    Ok(())
}

/// Extract public key shares from key shares
pub fn extract_public_key_shares(
    key_shares: &HashMap<ValidatorId, KeyShare>,
) -> Result<Vec<PublicKeyShare>> {
    let mut public_shares = Vec::new();

    for (validator_id, key_share) in key_shares {
        // For verification, we need to compute the verification key
        // This is typically the public key corresponding to the private share
        let verification_key = key_share.public_share.clone();

        let public_share = PublicKeyShare {
            validator_id: validator_id.clone(),
            public_share: key_share.public_share.clone(),
            verification_key,
        };

        public_shares.push(public_share);
    }

    Ok(public_shares)
}

/// Compute the combined public key from key shares
pub fn compute_combined_public_key(key_shares: &[PublicKeyShare]) -> Result<Vec<u8>> {
    if key_shares.is_empty() {
        return Err(ThresholdError::InvalidKeyShare {
            reason: "No key shares provided".to_string(),
        });
    }

    // For threshold schemes, the combined public key is typically the
    // evaluation of the public polynomial at x=0, which is the first coefficient
    // In our case, we'll use the first public key share as a placeholder
    // In a real implementation, this would be computed during DKG
    Ok(key_shares[0].public_share.clone())
}

/// Verify that a key share is valid (simplified version)
pub fn verify_key_share(key_share: &KeyShare) -> Result<()> {
    // Check that private share length is correct
    if key_share.private_share.len() != 32 {
        return Err(ThresholdError::InvalidKeyShare {
            reason: "Invalid private share length".to_string(),
        });
    }

    // Check that public share length is correct
    if key_share.public_share.len() != 65 && key_share.public_share.len() != 33 {
        return Err(ThresholdError::InvalidKeyShare {
            reason: "Invalid public share length".to_string(),
        });
    }

    // For the simplified version, we just check the basic format
    // In a full implementation, we would verify the cryptographic relationship
    Ok(())
}

/// Generate deterministic validator IDs for testing
pub fn generate_test_validator_ids(count: usize) -> Vec<ValidatorId> {
    (0..count)
        .map(|i| format!("validator_{:02}", i))
        .collect()
}

/// Create a test threshold configuration
pub fn create_test_config(threshold: u32, total: u32) -> Result<ThresholdConfig> {
    ThresholdConfig::new(threshold, total, 256)
}

/// Hash data with a domain separator
pub fn hash_with_domain(domain: &str, data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(domain.as_bytes());
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Secure random number generation
pub fn secure_random_bytes<R: RngCore + CryptoRng>(rng: &mut R, len: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; len];
    rng.fill_bytes(&mut bytes);
    bytes
}

/// Convert bytes to hex string
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Convert hex string to bytes
pub fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>> {
    hex::decode(hex_str).map_err(|e| ThresholdError::SerializationError {
        message: format!("Invalid hex string: {}", e),
    })
}

/// Timing-safe comparison of byte arrays
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }

    result == 0
}

/// Zeroize sensitive data
pub fn zeroize_vec(data: &mut Vec<u8>) {
    use zeroize::Zeroize;
    data.zeroize();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_session_id() {
        let id1 = generate_session_id();
        let id2 = generate_session_id();

        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 32); // 16 bytes * 2 hex chars
    }

    #[test]
    fn test_validate_threshold_config() {
        // Valid config
        let config = ThresholdConfig::new(2, 3, 256).unwrap();
        assert!(validate_threshold_config(&config).is_ok());

        // Invalid: threshold = 0
        let config = ThresholdConfig {
            threshold: 0,
            total_validators: 3,
            key_size: 256,
        };
        assert!(validate_threshold_config(&config).is_err());

        // Invalid: threshold > total
        let config = ThresholdConfig {
            threshold: 4,
            total_validators: 3,
            key_size: 256,
        };
        assert!(validate_threshold_config(&config).is_err());
    }

    #[test]
    fn test_generate_test_validator_ids() {
        let ids = generate_test_validator_ids(3);
        assert_eq!(ids.len(), 3);
        assert_eq!(ids[0], "validator_00");
        assert_eq!(ids[1], "validator_01");
        assert_eq!(ids[2], "validator_02");
    }

    #[test]
    fn test_hex_conversion() {
        let data = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
        let hex_str = bytes_to_hex(&data);
        assert_eq!(hex_str, "0123456789abcdef");

        let decoded = hex_to_bytes(&hex_str).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_constant_time_eq() {
        let a = vec![1, 2, 3, 4];
        let b = vec![1, 2, 3, 4];
        let c = vec![1, 2, 3, 5];
        let d = vec![1, 2, 3];

        assert!(constant_time_eq(&a, &b));
        assert!(!constant_time_eq(&a, &c));
        assert!(!constant_time_eq(&a, &d));
    }

    #[test]
    fn test_hash_with_domain() {
        let data = b"test data";
        let hash1 = hash_with_domain("domain1", data);
        let hash2 = hash_with_domain("domain2", data);
        let hash3 = hash_with_domain("domain1", data);

        assert_ne!(hash1, hash2);
        assert_eq!(hash1, hash3);
        assert_eq!(hash1.len(), 32); // SHA256 output
    }
}
