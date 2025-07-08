//! Cross-chain bridge relayer service
//!
//! This service monitors events on both Ethereum and Polkadot chains,
//! coordinates threshold signature generation, and submits proofs.

pub mod error;
pub mod ethereum;
pub mod polkadot;
pub mod coordinator;
pub mod config;
pub mod event_monitor;
pub mod signature_coordinator;
pub mod database;

pub use error::{RelayerError, Result};
pub use coordinator::BridgeCoordinator;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
