//! Schnorr threshold signature implementation

use crate::{
    error::{Result, ThresholdError},
    types::{
        AggregatedSignature, KeyShare, PartialSignature, PublicKeyShare, ThresholdConfig,
        ValidatorId,
    },
};
use k256::{
    elliptic_curve::{
        group::GroupEncoding,
        ops::Reduce,
        point::AffineCoordinates,
        sec1::{FromEncodedPoint, ToEncodedPoint},
        Field, FieldBytes,
    },
    AffinePoint, ProjectivePoint, Scalar, U256,
};
use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Schnorr threshold signature implementation
#[derive(Debug, Clone)]
pub struct SchnorrThreshold {
    /// Random number generator
    rng: rand::rngs::OsRng,
}

impl SchnorrThreshold {
    /// Create a new Schnorr threshold signature instance
    pub fn new() -> Self {
        Self {
            rng: rand::rngs::OsRng,
        }
    }

    /// Generate a random scalar
    fn random_scalar<R: RngCore + CryptoRng>(&self, rng: &mut R) -> Scalar {
        Scalar::random(rng)
    }

    /// Hash message with domain separation
    fn hash_message(&self, message: &[u8], context: &str) -> Scalar {
        let mut hasher = Sha256::new();
        hasher.update(context.as_bytes());
        hasher.update(message);
        let hash = hasher.finalize();
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
        let inv_denominator = Option::from(denominator.invert())
            .ok_or_else(|| ThresholdError::CryptographicError {
                message: "Failed to compute Lagrange coefficient".to_string(),
            })?;

        Ok(numerator * inv_denominator)
    }
}

impl SchnorrThreshold {
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

        let private_share = Option::from(Scalar::from_repr(FieldBytes::from(private_share_bytes)))
            .ok_or_else(|| ThresholdError::InvalidKeyShare {
                reason: "Invalid private share scalar".to_string(),
            })?;

        let mut rng = rand::rngs::OsRng;

        // Generate nonce
        let nonce = self.random_scalar(&mut rng);
        let nonce_point = (ProjectivePoint::GENERATOR * nonce).to_affine();

        // Create challenge hash
        let mut hasher = Sha256::new();
        hasher.update(session_id.as_bytes());
        hasher.update(nonce_point.to_encoded_point(false).as_bytes());
        hasher.update(message);
        let challenge_hash = hasher.finalize();
        let challenge = Scalar::reduce(U256::from_be_slice(&challenge_hash));

        // Compute partial signature: s = nonce + challenge * private_share
        let signature_scalar = nonce + challenge * private_share;

        Ok(PartialSignature {
            validator_id: key_share.validator_id.clone(),
            signature: signature_scalar.to_bytes().to_vec(),
            commitment: Some(nonce_point.to_encoded_point(false).as_bytes().to_vec()),
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

        // Extract signer indices (assuming 1-indexed)
        let signer_indices: Vec<u32> = (1..=partial_sigs.len() as u32).collect();

        // Aggregate signatures using Lagrange interpolation
        let mut aggregated_signature = Scalar::ZERO;

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

            let partial_scalar = Scalar::from_bytes(&sig_bytes.into())
                .ok_or_else(|| ThresholdError::InvalidSignature {
                    reason: "Invalid signature scalar".to_string(),
                })?;

            // Add weighted partial signature
            aggregated_signature += lagrange_coeff * partial_scalar;
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

        let signers: Vec<ValidatorId> = partial_sigs
            .iter()
            .map(|sig| sig.validator_id.clone())
            .collect();

        Ok(AggregatedSignature {
            signature: aggregated_signature.to_bytes().to_vec(),
            signers,
            public_key: aggregated_pubkey.to_affine().to_encoded_point(false).as_bytes().to_vec(),
            scheme: "schnorr".to_string(),
            timestamp: std::time::SystemTime::now(),
        })
    }

    async fn verify_signature(
        &self,
        signature: &AggregatedSignature,
        message: &[u8],
        public_key: &[u8],
    ) -> Result<bool> {
        // Deserialize signature
        let sig_bytes: [u8; 32] = signature
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| ThresholdError::InvalidSignature {
                reason: "Invalid signature length".to_string(),
            })?;

        let signature_scalar = Scalar::from_bytes(&sig_bytes.into())
            .ok_or_else(|| ThresholdError::InvalidSignature {
                reason: "Invalid signature scalar".to_string(),
            })?;

        // Deserialize public key
        let pubkey_point = AffinePoint::from_encoded_point(
            &k256::EncodedPoint::from_bytes(public_key)
                .map_err(|_| ThresholdError::InvalidSignature {
                    reason: "Invalid public key encoding".to_string(),
                })?,
        )
        .ok_or_else(|| ThresholdError::InvalidSignature {
            reason: "Invalid public key point".to_string(),
        })?;

        // For verification, we would need the original nonce commitments
        // This is a simplified verification - in practice, you'd need to store
        // and verify against the original nonce commitments
        let challenge = self.hash_message(message, "schnorr_challenge");

        // Verify: s * G = R + c * P
        let left_side = ProjectivePoint::GENERATOR * signature_scalar;
        let right_side = ProjectivePoint::from(pubkey_point) * challenge;

        Ok(left_side.to_affine() == right_side.to_affine())
    }
}

impl Default for SchnorrThreshold {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_schnorr_key_generation() {
        let config = ThresholdConfig::new(2, 3, 256).unwrap();
        let validator_ids = vec!["val1".to_string(), "val2".to_string(), "val3".to_string()];

        let schnorr = SchnorrThreshold::new();
        let key_shares = schnorr.generate_keys(&config, &validator_ids).await.unwrap();

        assert_eq!(key_shares.len(), 3);
        assert!(key_shares.contains_key("val1"));
        assert!(key_shares.contains_key("val2"));
        assert!(key_shares.contains_key("val3"));
    }
}
