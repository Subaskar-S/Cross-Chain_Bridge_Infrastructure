//! ECDSA threshold signature implementation

use crate::{
    error::{Result, ThresholdError},
    types::{
        AggregatedSignature, KeyShare, PartialSignature, PublicKeyShare, ThresholdConfig,
        ValidatorId,
    },
    ThresholdScheme,
};
use k256::{
    ecdsa::{RecoveryId, Signature as EcdsaSignature, VerifyingKey},
    elliptic_curve::{
        group::GroupEncoding,
        ops::{Reduce, ReduceNonZero},
        point::AffineCoordinates,
        Field, Group,
    },
    AffinePoint, ProjectivePoint, Scalar, U256,
};
use rand::{CryptoRng, RngCore};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// ECDSA threshold signature implementation
#[derive(Debug, Clone)]
pub struct EcdsaThreshold {
    /// Random number generator
    rng: rand::rngs::OsRng,
}

impl EcdsaThreshold {
    /// Create a new ECDSA threshold signature instance
    pub fn new() -> Self {
        Self {
            rng: rand::rngs::OsRng,
        }
    }

    /// Generate a random scalar
    fn random_scalar<R: RngCore + CryptoRng>(&self, rng: &mut R) -> Scalar {
        Scalar::random(rng)
    }

    /// Hash message for ECDSA signing
    fn hash_message(&self, message: &[u8]) -> Scalar {
        let hash = Sha256::digest(message);
        Scalar::reduce(U256::from_be_slice(&hash))
    }

    /// Generate Lagrange coefficient for interpolation
    fn lagrange_coefficient(&self, i: u32, signers: &[u32]) -> Result<Scalar> {
        let mut numerator = Scalar::ONE;
        let mut denominator = Scalar::ONE;

        for &j in signers {
            if i != j {
                numerator *= Scalar::from(j);
                let diff = if j > i {
                    Scalar::from(j - i)
                } else {
                    -Scalar::from(i - j)
                };
                denominator *= diff;
            }
        }

        // Compute modular inverse
        let inv_denominator = denominator
            .invert()
            .ok_or_else(|| ThresholdError::CryptographicError {
                message: "Failed to compute Lagrange coefficient".to_string(),
            })?;

        Ok(numerator * inv_denominator)
    }
}

#[async_trait::async_trait]
impl ThresholdScheme for EcdsaThreshold {
    async fn generate_keys(
        &self,
        config: &ThresholdConfig,
        validator_ids: &[ValidatorId],
    ) -> Result<HashMap<ValidatorId, KeyShare>> {
        if validator_ids.len() != config.total_validators as usize {
            return Err(ThresholdError::InvalidThreshold {
                threshold: config.threshold,
                total: config.total_validators,
            });
        }

        let mut rng = rand::rngs::OsRng;
        let mut key_shares = HashMap::new();

        // Generate polynomial coefficients
        let mut coefficients = Vec::new();
        for _ in 0..config.threshold {
            coefficients.push(self.random_scalar(&mut rng));
        }

        // Generate key shares for each validator
        for (index, validator_id) in validator_ids.iter().enumerate() {
            let x = Scalar::from((index + 1) as u32); // x-coordinate (1-indexed)

            // Evaluate polynomial at x
            let mut private_share = coefficients[0]; // a_0
            let mut x_power = x;

            for coeff in coefficients.iter().skip(1) {
                private_share += *coeff * x_power;
                x_power *= x;
            }

            // Compute public share
            let public_share = (ProjectivePoint::GENERATOR * private_share).to_affine();

            // Serialize coefficients for verification
            let serialized_coeffs: Vec<Vec<u8>> = coefficients
                .iter()
                .map(|c| c.to_bytes().to_vec())
                .collect();

            let key_share = KeyShare {
                validator_id: validator_id.clone(),
                private_share: private_share.to_bytes().to_vec(),
                public_share: public_share.to_encoded_point(false).as_bytes().to_vec(),
                coefficients: serialized_coeffs,
                config: config.clone(),
            };

            key_shares.insert(validator_id.clone(), key_share);
        }

        Ok(key_shares)
    }

    async fn partial_sign(
        &self,
        key_share: &KeyShare,
        message: &[u8],
        session_id: &str,
    ) -> Result<PartialSignature> {
        // Deserialize private share
        let private_share_bytes: [u8; 32] = key_share
            .private_share
            .as_slice()
            .try_into()
            .map_err(|_| ThresholdError::InvalidKeyShare {
                reason: "Invalid private share length".to_string(),
            })?;

        let private_share = Scalar::from_bytes(&private_share_bytes.into())
            .ok_or_else(|| ThresholdError::InvalidKeyShare {
                reason: "Invalid private share scalar".to_string(),
            })?;

        let mut rng = rand::rngs::OsRng;

        // Generate nonce k
        let k = self.random_scalar(&mut rng);
        let k_point = (ProjectivePoint::GENERATOR * k).to_affine();

        // Get r coordinate
        let r = Scalar::reduce_nonzero(U256::from_be_slice(
            &k_point.x().to_bytes(),
        ))
        .ok_or_else(|| ThresholdError::CryptographicError {
            message: "Failed to compute r coordinate".to_string(),
        })?;

        // Hash the message
        let message_hash = self.hash_message(message);

        // Compute k^(-1)
        let k_inv = k.invert().ok_or_else(|| ThresholdError::CryptographicError {
            message: "Failed to invert k".to_string(),
        })?;

        // Compute partial signature: s_i = k^(-1) * (H(m) + r * x_i)
        let s_partial = k_inv * (message_hash + r.as_ref() * private_share);

        // Store r as commitment
        let commitment = r.to_bytes().to_vec();

        Ok(PartialSignature {
            validator_id: key_share.validator_id.clone(),
            signature: s_partial.to_bytes().to_vec(),
            commitment: Some(commitment),
            timestamp: std::time::SystemTime::now(),
        })
    }

    async fn aggregate_signatures(
        &self,
        partial_sigs: &[PartialSignature],
        public_key_shares: &[PublicKeyShare],
        message: &[u8],
        config: &ThresholdConfig,
    ) -> Result<AggregatedSignature> {
        if partial_sigs.len() < config.threshold as usize {
            return Err(ThresholdError::InsufficientSignatures {
                required: config.threshold,
                received: partial_sigs.len() as u32,
            });
        }

        // All partial signatures should have the same r value
        let r_bytes = partial_sigs[0]
            .commitment
            .as_ref()
            .ok_or_else(|| ThresholdError::InvalidSignature {
                reason: "Missing r commitment".to_string(),
            })?;

        let r_array: [u8; 32] = r_bytes
            .as_slice()
            .try_into()
            .map_err(|_| ThresholdError::InvalidSignature {
                reason: "Invalid r length".to_string(),
            })?;

        let r = Scalar::from_bytes(&r_array.into())
            .ok_or_else(|| ThresholdError::InvalidSignature {
                reason: "Invalid r scalar".to_string(),
            })?;

        // Extract signer indices (assuming 1-indexed)
        let signer_indices: Vec<u32> = (1..=partial_sigs.len() as u32).collect();

        // Aggregate signatures using Lagrange interpolation
        let mut aggregated_s = Scalar::ZERO;

        for (i, partial_sig) in partial_sigs.iter().enumerate() {
            let signer_index = signer_indices[i];

            // Compute Lagrange coefficient
            let lagrange_coeff = self.lagrange_coefficient(signer_index, &signer_indices)?;

            // Deserialize partial signature
            let sig_bytes: [u8; 32] = partial_sig
                .signature
                .as_slice()
                .try_into()
                .map_err(|_| ThresholdError::InvalidSignature {
                    reason: "Invalid signature length".to_string(),
                })?;

            let partial_s = Scalar::from_bytes(&sig_bytes.into())
                .ok_or_else(|| ThresholdError::InvalidSignature {
                    reason: "Invalid signature scalar".to_string(),
                })?;

            // Add weighted partial signature
            aggregated_s += lagrange_coeff * partial_s;
        }

        // Compute aggregated public key
        let mut aggregated_pubkey = ProjectivePoint::IDENTITY;
        for (i, pubkey_share) in public_key_shares.iter().take(partial_sigs.len()).enumerate() {
            let signer_index = signer_indices[i];
            let lagrange_coeff = self.lagrange_coefficient(signer_index, &signer_indices)?;

            // Deserialize public key share
            let pubkey_point = AffinePoint::from_encoded_point(
                &k256::EncodedPoint::from_bytes(&pubkey_share.public_share)
                    .map_err(|_| ThresholdError::InvalidSignature {
                        reason: "Invalid public key encoding".to_string(),
                    })?,
            )
            .ok_or_else(|| ThresholdError::InvalidSignature {
                reason: "Invalid public key point".to_string(),
            })?;

            aggregated_pubkey += ProjectivePoint::from(pubkey_point) * lagrange_coeff;
        }

        // Create ECDSA signature (r, s)
        let mut signature_bytes = Vec::with_capacity(64);
        signature_bytes.extend_from_slice(&r.to_bytes());
        signature_bytes.extend_from_slice(&aggregated_s.to_bytes());

        let signers: Vec<ValidatorId> = partial_sigs
            .iter()
            .map(|sig| sig.validator_id.clone())
            .collect();

        Ok(AggregatedSignature {
            signature: signature_bytes,
            signers,
            public_key: aggregated_pubkey.to_affine().to_encoded_point(false).as_bytes().to_vec(),
            scheme: "ecdsa".to_string(),
            timestamp: std::time::SystemTime::now(),
        })
    }

    async fn verify_signature(
        &self,
        signature: &AggregatedSignature,
        message: &[u8],
        public_key: &[u8],
    ) -> Result<bool> {
        if signature.signature.len() != 64 {
            return Err(ThresholdError::InvalidSignature {
                reason: "Invalid ECDSA signature length".to_string(),
            });
        }

        // Extract r and s from signature
        let r_bytes: [u8; 32] = signature.signature[0..32]
            .try_into()
            .map_err(|_| ThresholdError::InvalidSignature {
                reason: "Invalid r length".to_string(),
            })?;

        let s_bytes: [u8; 32] = signature.signature[32..64]
            .try_into()
            .map_err(|_| ThresholdError::InvalidSignature {
                reason: "Invalid s length".to_string(),
            })?;

        // Create ECDSA signature
        let ecdsa_sig = EcdsaSignature::from_scalars(r_bytes, s_bytes)
            .map_err(|e| ThresholdError::InvalidSignature {
                reason: format!("Invalid ECDSA signature: {}", e),
            })?;

        // Create verifying key
        let verifying_key = VerifyingKey::from_encoded_point(
            &k256::EncodedPoint::from_bytes(public_key)
                .map_err(|_| ThresholdError::InvalidSignature {
                    reason: "Invalid public key encoding".to_string(),
                })?,
        )
        .map_err(|e| ThresholdError::InvalidSignature {
            reason: format!("Invalid verifying key: {}", e),
        })?;

        // Verify signature
        use k256::ecdsa::signature::Verifier;
        Ok(verifying_key.verify(message, &ecdsa_sig).is_ok())
    }
}

impl Default for EcdsaThreshold {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_ecdsa_key_generation() {
        let config = ThresholdConfig::new(2, 3, 256).unwrap();
        let validator_ids = vec!["val1".to_string(), "val2".to_string(), "val3".to_string()];

        let ecdsa = EcdsaThreshold::new();
        let key_shares = ecdsa.generate_keys(&config, &validator_ids).await.unwrap();

        assert_eq!(key_shares.len(), 3);
        assert!(key_shares.contains_key("val1"));
        assert!(key_shares.contains_key("val2"));
        assert!(key_shares.contains_key("val3"));
    }
}
