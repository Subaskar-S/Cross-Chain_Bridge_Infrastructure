//! Event monitoring service for cross-chain events

use crate::{
    coordinator::BridgeEvent,
    config::MonitoringConfig,
    database::Database,
    ethereum::EthereumClient,
    polkadot::PolkadotClient,
    error::{RelayerError, Result},
};
use tokio::sync::mpsc;
use tracing::{info, debug, error, warn};
use std::sync::Arc;
use std::time::Duration;

/// Event monitor that watches both chains for bridge events
pub struct EventMonitor {
    ethereum_client: Arc<EthereumClient>,
    polkadot_client: Arc<PolkadotClient>,
    database: Arc<Database>,
}

impl EventMonitor {
    /// Create a new event monitor
    pub async fn new(
        ethereum_client: Arc<EthereumClient>,
        polkadot_client: Arc<PolkadotClient>,
        database: Arc<Database>,
    ) -> Result<Self> {
        Ok(Self {
            ethereum_client,
            polkadot_client,
            database,
        })
    }

    /// Start monitoring events on both chains
    pub async fn start_monitoring(
        &self,
        event_sender: mpsc::UnboundedSender<BridgeEvent>,
        config: MonitoringConfig,
    ) -> Result<()> {
        info!("Starting event monitoring");

        // Start Ethereum monitoring
        let ethereum_client = self.ethereum_client.clone();
        let database = self.database.clone();
        let event_sender_eth = event_sender.clone();
        let poll_interval = config.poll_interval;

        tokio::spawn(async move {
            if let Err(e) = Self::monitor_ethereum_events(
                ethereum_client,
                database,
                event_sender_eth,
                poll_interval,
            ).await {
                error!("Ethereum event monitoring failed: {}", e);
            }
        });

        // Start Polkadot monitoring
        let polkadot_client = self.polkadot_client.clone();
        let database = self.database.clone();
        let event_sender_dot = event_sender;

        tokio::spawn(async move {
            if let Err(e) = Self::monitor_polkadot_events(
                polkadot_client,
                database,
                event_sender_dot,
                poll_interval,
            ).await {
                error!("Polkadot event monitoring failed: {}", e);
            }
        });

        Ok(())
    }

    /// Monitor Ethereum events
    async fn monitor_ethereum_events(
        ethereum_client: Arc<EthereumClient>,
        database: Arc<Database>,
        event_sender: mpsc::UnboundedSender<BridgeEvent>,
        poll_interval: u64,
    ) -> Result<()> {
        info!("Starting Ethereum event monitoring");

        let mut last_processed_block = database.get_last_processed_ethereum_block().await?
            .unwrap_or(0);

        loop {
            match Self::process_ethereum_events(
                &ethereum_client,
                &database,
                &event_sender,
                last_processed_block,
            ).await {
                Ok(new_block) => {
                    if new_block > last_processed_block {
                        last_processed_block = new_block;
                        database.set_last_processed_ethereum_block(new_block).await?;
                    }
                }
                Err(e) => {
                    error!("Error processing Ethereum events: {}", e);
                }
            }

            tokio::time::sleep(Duration::from_secs(poll_interval)).await;
        }
    }

    /// Process Ethereum events for a block range
    async fn process_ethereum_events(
        ethereum_client: &EthereumClient,
        database: &Database,
        event_sender: &mpsc::UnboundedSender<BridgeEvent>,
        from_block: u64,
    ) -> Result<u64> {
        let current_block = ethereum_client.get_block_number().await?;
        
        if current_block <= from_block {
            return Ok(from_block);
        }

        debug!("Processing Ethereum blocks {} to {}", from_block + 1, current_block);

        // Get past lock events
        let lock_events = ethereum_client.get_past_lock_events(from_block + 1).await?;

        for event in lock_events {
            let bridge_event = BridgeEvent::EthereumLock {
                user: format!("{:?}", event.user),
                token: format!("{:?}", event.token),
                amount: event.amount.to_string(),
                polkadot_address: format!("{:?}", event.polkadot_address),
                tx_hash: "mock_tx_hash".to_string(), // Would get from event metadata
                block_number: current_block,
            };

            if let Err(e) = event_sender.send(bridge_event) {
                error!("Failed to send Ethereum event: {}", e);
            }
        }

        Ok(current_block)
    }

    /// Monitor Polkadot events
    async fn monitor_polkadot_events(
        polkadot_client: Arc<PolkadotClient>,
        database: Arc<Database>,
        event_sender: mpsc::UnboundedSender<BridgeEvent>,
        poll_interval: u64,
    ) -> Result<()> {
        info!("Starting Polkadot event monitoring");

        let mut last_processed_block = database.get_last_processed_polkadot_block().await?
            .unwrap_or(0);

        loop {
            match Self::process_polkadot_events(
                &polkadot_client,
                &database,
                &event_sender,
                last_processed_block,
            ).await {
                Ok(new_block) => {
                    if new_block > last_processed_block {
                        last_processed_block = new_block;
                        database.set_last_processed_polkadot_block(new_block).await?;
                    }
                }
                Err(e) => {
                    error!("Error processing Polkadot events: {}", e);
                }
            }

            tokio::time::sleep(Duration::from_secs(poll_interval)).await;
        }
    }

    /// Process Polkadot events for a block range
    async fn process_polkadot_events(
        polkadot_client: &PolkadotClient,
        database: &Database,
        event_sender: &mpsc::UnboundedSender<BridgeEvent>,
        from_block: u32,
    ) -> Result<u32> {
        let current_block = polkadot_client.get_block_number().await?;
        
        if current_block <= from_block {
            return Ok(from_block);
        }

        debug!("Processing Polkadot blocks {} to {}", from_block + 1, current_block);

        // Get past burn events
        let burn_events = polkadot_client.get_past_burn_events(from_block + 1).await?;

        for event in burn_events {
            let bridge_event = BridgeEvent::PolkadotBurn {
                user: event.burner,
                asset_id: event.asset_id,
                amount: event.amount,
                ethereum_recipient: event.ethereum_recipient,
                tx_hash: event.tx_hash,
                block_number: event.block_number,
            };

            if let Err(e) = event_sender.send(bridge_event) {
                error!("Failed to send Polkadot event: {}", e);
            }
        }

        Ok(current_block)
    }
}
