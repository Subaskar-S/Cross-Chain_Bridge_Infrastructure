//! Main bridge coordinator that orchestrates cross-chain operations

use crate::{
    config::RelayerConfig,
    error::{RelayerError, Result},
    ethereum::EthereumClient,
    polkadot::PolkadotClient,
    event_monitor::EventMonitor,
    signature_coordinator::SignatureCoordinator,
    database::Database,
};
use threshold::{SimpleThresholdManager, ThresholdConfig};
use tokio::sync::mpsc;
use tracing::{info, warn, error, debug};
use std::sync::Arc;

/// Events that can occur in the bridge
#[derive(Debug, Clone)]
pub enum BridgeEvent {
    /// Token lock event from Ethereum
    EthereumLock {
        user: String,
        token: String,
        amount: String,
        polkadot_address: String,
        tx_hash: String,
        block_number: u64,
    },
    /// Token burn event from Polkadot
    PolkadotBurn {
        user: String,
        asset_id: u32,
        amount: String,
        ethereum_recipient: String,
        tx_hash: String,
        block_number: u32,
    },
}

/// Main bridge coordinator
pub struct BridgeCoordinator {
    config: RelayerConfig,
    ethereum_client: Arc<EthereumClient>,
    polkadot_client: Arc<PolkadotClient>,
    threshold_manager: Arc<SimpleThresholdManager>,
    signature_coordinator: Arc<SignatureCoordinator>,
    database: Arc<Database>,
    event_monitor: Arc<EventMonitor>,
    event_sender: mpsc::UnboundedSender<BridgeEvent>,
    event_receiver: mpsc::UnboundedReceiver<BridgeEvent>,
}

impl BridgeCoordinator {
    /// Create a new bridge coordinator
    pub async fn new(config: RelayerConfig) -> Result<Self> {
        info!("Initializing bridge coordinator");

        // Initialize threshold manager
        let threshold_config = ThresholdConfig::new(
            config.threshold.threshold,
            config.threshold.total_validators,
            config.threshold.key_size,
        ).map_err(|e| RelayerError::ThresholdSignature(e))?;

        let threshold_manager = Arc::new(
            SimpleThresholdManager::new(threshold_config)
                .map_err(|e| RelayerError::ThresholdSignature(e))?
        );

        // Initialize database
        let database = Arc::new(Database::new(&config.database).await?);

        // Initialize clients
        let ethereum_client = Arc::new(EthereumClient::new(&config.ethereum).await?);
        let polkadot_client = Arc::new(PolkadotClient::new(&config.polkadot).await?);

        // Initialize signature coordinator
        let signature_coordinator = Arc::new(
            SignatureCoordinator::new(
                config.validator.clone(),
                threshold_manager.clone(),
                database.clone(),
            ).await?
        );

        // Initialize event monitor
        let event_monitor = Arc::new(
            EventMonitor::new(
                ethereum_client.clone(),
                polkadot_client.clone(),
                database.clone(),
            ).await?
        );

        // Create event channel
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Ok(Self {
            config,
            ethereum_client,
            polkadot_client,
            threshold_manager,
            signature_coordinator,
            database,
            event_monitor,
            event_sender,
            event_receiver,
        })
    }

    /// Start the bridge coordinator
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting bridge coordinator");

        // Start event monitoring
        let event_sender = self.event_sender.clone();
        let event_monitor = self.event_monitor.clone();
        let monitoring_config = self.config.monitoring.clone();

        tokio::spawn(async move {
            if let Err(e) = event_monitor.start_monitoring(event_sender, monitoring_config).await {
                error!("Event monitoring failed: {}", e);
            }
        });

        // Start signature coordination if validator mode is enabled
        if self.config.validator.enabled {
            info!("Starting validator mode");
            let signature_coordinator = self.signature_coordinator.clone();
            tokio::spawn(async move {
                if let Err(e) = signature_coordinator.start().await {
                    error!("Signature coordination failed: {}", e);
                }
            });
        }

        // Main event processing loop
        self.process_events().await
    }

    /// Process bridge events
    async fn process_events(&mut self) -> Result<()> {
        info!("Starting event processing loop");

        while let Some(event) = self.event_receiver.recv().await {
            if let Err(e) = self.handle_event(event).await {
                error!("Failed to handle event: {}", e);
                // Continue processing other events
            }
        }

        warn!("Event processing loop ended");
        Ok(())
    }

    /// Handle a single bridge event
    async fn handle_event(&self, event: BridgeEvent) -> Result<()> {
        debug!("Handling bridge event: {:?}", event);

        match event {
            BridgeEvent::EthereumLock {
                user,
                token,
                amount,
                polkadot_address,
                tx_hash,
                block_number,
            } => {
                self.handle_ethereum_lock(
                    user,
                    token,
                    amount,
                    polkadot_address,
                    tx_hash,
                    block_number,
                ).await
            }
            BridgeEvent::PolkadotBurn {
                user,
                asset_id,
                amount,
                ethereum_recipient,
                tx_hash,
                block_number,
            } => {
                self.handle_polkadot_burn(
                    user,
                    asset_id,
                    amount,
                    ethereum_recipient,
                    tx_hash,
                    block_number,
                ).await
            }
        }
    }

    /// Handle Ethereum lock event (mint on Polkadot)
    async fn handle_ethereum_lock(
        &self,
        user: String,
        token: String,
        amount: String,
        polkadot_address: String,
        tx_hash: String,
        block_number: u64,
    ) -> Result<()> {
        info!(
            "Processing Ethereum lock: user={}, token={}, amount={}, tx_hash={}",
            user, token, amount, tx_hash
        );

        // Check if already processed
        if self.database.is_ethereum_tx_processed(&tx_hash).await? {
            debug!("Transaction {} already processed", tx_hash);
            return Ok(());
        }

        // Store the lock request
        self.database.store_ethereum_lock(
            &user,
            &token,
            &amount,
            &polkadot_address,
            &tx_hash,
            block_number,
        ).await?;

        // If validator mode is enabled, participate in signature generation
        if self.config.validator.enabled {
            self.signature_coordinator.request_mint_signature(
                &polkadot_address,
                &token,
                &amount,
                &tx_hash,
            ).await?;
        }

        // If we have enough signatures, submit to Polkadot
        if let Some(signatures) = self.signature_coordinator.get_mint_signatures(&tx_hash).await? {
            self.polkadot_client.mint_tokens(
                &polkadot_address,
                &token,
                &amount,
                &tx_hash,
                signatures,
            ).await?;

            // Mark as processed
            self.database.mark_ethereum_tx_processed(&tx_hash).await?;
            info!("Successfully minted tokens on Polkadot for tx {}", tx_hash);
        }

        Ok(())
    }

    /// Handle Polkadot burn event (unlock on Ethereum)
    async fn handle_polkadot_burn(
        &self,
        user: String,
        asset_id: u32,
        amount: String,
        ethereum_recipient: String,
        tx_hash: String,
        block_number: u32,
    ) -> Result<()> {
        info!(
            "Processing Polkadot burn: user={}, asset_id={}, amount={}, tx_hash={}",
            user, asset_id, amount, tx_hash
        );

        // Check if already processed
        if self.database.is_polkadot_tx_processed(&tx_hash).await? {
            debug!("Transaction {} already processed", tx_hash);
            return Ok(());
        }

        // Store the burn request
        self.database.store_polkadot_burn(
            &user,
            asset_id,
            &amount,
            &ethereum_recipient,
            &tx_hash,
            block_number,
        ).await?;

        // If validator mode is enabled, participate in signature generation
        if self.config.validator.enabled {
            self.signature_coordinator.request_unlock_signature(
                &ethereum_recipient,
                asset_id,
                &amount,
                &tx_hash,
            ).await?;
        }

        // If we have enough signatures, submit to Ethereum
        if let Some(signatures) = self.signature_coordinator.get_unlock_signatures(&tx_hash).await? {
            // Get token address from asset_id
            let token_address = self.database.get_token_address_by_asset_id(asset_id).await?;
            
            self.ethereum_client.unlock_tokens(
                &ethereum_recipient,
                &token_address,
                &amount,
                &tx_hash,
                signatures,
            ).await?;

            // Mark as processed
            self.database.mark_polkadot_tx_processed(&tx_hash).await?;
            info!("Successfully unlocked tokens on Ethereum for tx {}", tx_hash);
        }

        Ok(())
    }

    /// Get bridge statistics
    pub async fn get_stats(&self) -> Result<BridgeStats> {
        let stats = BridgeStats {
            ethereum_processed_txs: self.database.count_ethereum_processed_txs().await?,
            polkadot_processed_txs: self.database.count_polkadot_processed_txs().await?,
            pending_signatures: self.signature_coordinator.count_pending_signatures().await?,
            active_validators: self.signature_coordinator.count_active_validators().await?,
        };

        Ok(stats)
    }

    /// Shutdown the coordinator gracefully
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down bridge coordinator");
        
        // Close database connections
        self.database.close().await?;
        
        info!("Bridge coordinator shutdown complete");
        Ok(())
    }
}

/// Bridge statistics
#[derive(Debug, Clone)]
pub struct BridgeStats {
    pub ethereum_processed_txs: u64,
    pub polkadot_processed_txs: u64,
    pub pending_signatures: u64,
    pub active_validators: u64,
}
