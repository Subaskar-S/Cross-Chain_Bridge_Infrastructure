//! API service for the cross-chain bridge
//!
//! Provides REST and WebSocket endpoints for monitoring bridge status,
//! validator information, and transaction history.

pub mod handlers;
pub mod routes;
pub mod error;
pub mod server;
pub mod websocket;
pub mod middleware;

pub use error::{ApiError, Result};
pub use server::ApiServer;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
