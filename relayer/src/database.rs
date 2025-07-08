//! Database operations for the relayer service

use crate::{
    config::DatabaseConfig,
    error::{RelayerError, Result},
};
use sqlx::{PgPool, Row};
use tracing::{info, debug, error};

/// Database client for storing bridge state
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new database client
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        info!("Connecting to database at {}", config.url);

        let pool = PgPool::connect(&config.url)
            .await
            .map_err(|e| RelayerError::Database {
                message: format!("Failed to connect to database: {}", e),
            })?;

        // Run migrations
        let db = Self { pool };
        db.migrate().await?;

        Ok(db)
    }

    /// Run database migrations
    async fn migrate(&self) -> Result<()> {
        info!("Running database migrations");

        // Create tables if they don't exist
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS ethereum_locks (
                id SERIAL PRIMARY KEY,
                user_address VARCHAR(42) NOT NULL,
                token_address VARCHAR(42) NOT NULL,
                amount VARCHAR(78) NOT NULL,
                polkadot_address VARCHAR(66) NOT NULL,
                tx_hash VARCHAR(66) NOT NULL UNIQUE,
                block_number BIGINT NOT NULL,
                processed BOOLEAN DEFAULT FALSE,
                created_at TIMESTAMP DEFAULT NOW()
            )
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| RelayerError::Database {
            message: format!("Failed to create ethereum_locks table: {}", e),
        })?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS polkadot_burns (
                id SERIAL PRIMARY KEY,
                user_account VARCHAR(48) NOT NULL,
                asset_id INTEGER NOT NULL,
                amount VARCHAR(78) NOT NULL,
                ethereum_recipient VARCHAR(42) NOT NULL,
                tx_hash VARCHAR(66) NOT NULL UNIQUE,
                block_number INTEGER NOT NULL,
                processed BOOLEAN DEFAULT FALSE,
                created_at TIMESTAMP DEFAULT NOW()
            )
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| RelayerError::Database {
            message: format!("Failed to create polkadot_burns table: {}", e),
        })?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS processed_transactions (
                id SERIAL PRIMARY KEY,
                tx_hash VARCHAR(66) NOT NULL UNIQUE,
                chain VARCHAR(20) NOT NULL,
                processed_at TIMESTAMP DEFAULT NOW()
            )
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| RelayerError::Database {
            message: format!("Failed to create processed_transactions table: {}", e),
        })?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS bridge_state (
                id SERIAL PRIMARY KEY,
                key VARCHAR(50) NOT NULL UNIQUE,
                value VARCHAR(100) NOT NULL,
                updated_at TIMESTAMP DEFAULT NOW()
            )
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| RelayerError::Database {
            message: format!("Failed to create bridge_state table: {}", e),
        })?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS token_mappings (
                id SERIAL PRIMARY KEY,
                ethereum_address VARCHAR(42) NOT NULL UNIQUE,
                polkadot_asset_id INTEGER NOT NULL UNIQUE,
                created_at TIMESTAMP DEFAULT NOW()
            )
        "#)
        .execute(&self.pool)
        .await
        .map_err(|e| RelayerError::Database {
            message: format!("Failed to create token_mappings table: {}", e),
        })?;

        info!("Database migrations completed");
        Ok(())
    }

    /// Store an Ethereum lock event
    pub async fn store_ethereum_lock(
        &self,
        user: &str,
        token: &str,
        amount: &str,
        polkadot_address: &str,
        tx_hash: &str,
        block_number: u64,
    ) -> Result<()> {
        sqlx::query(r#"
            INSERT INTO ethereum_locks (user_address, token_address, amount, polkadot_address, tx_hash, block_number)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (tx_hash) DO NOTHING
        "#)
        .bind(user)
        .bind(token)
        .bind(amount)
        .bind(polkadot_address)
        .bind(tx_hash)
        .bind(block_number as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| RelayerError::Database {
            message: format!("Failed to store Ethereum lock: {}", e),
        })?;

        debug!("Stored Ethereum lock: {}", tx_hash);
        Ok(())
    }

    /// Store a Polkadot burn event
    pub async fn store_polkadot_burn(
        &self,
        user: &str,
        asset_id: u32,
        amount: &str,
        ethereum_recipient: &str,
        tx_hash: &str,
        block_number: u32,
    ) -> Result<()> {
        sqlx::query(r#"
            INSERT INTO polkadot_burns (user_account, asset_id, amount, ethereum_recipient, tx_hash, block_number)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (tx_hash) DO NOTHING
        "#)
        .bind(user)
        .bind(asset_id as i32)
        .bind(amount)
        .bind(ethereum_recipient)
        .bind(tx_hash)
        .bind(block_number as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| RelayerError::Database {
            message: format!("Failed to store Polkadot burn: {}", e),
        })?;

        debug!("Stored Polkadot burn: {}", tx_hash);
        Ok(())
    }

    /// Check if an Ethereum transaction is processed
    pub async fn is_ethereum_tx_processed(&self, tx_hash: &str) -> Result<bool> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM processed_transactions WHERE tx_hash = $1 AND chain = 'ethereum'")
            .bind(tx_hash)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RelayerError::Database {
                message: format!("Failed to check Ethereum tx status: {}", e),
            })?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    /// Check if a Polkadot transaction is processed
    pub async fn is_polkadot_tx_processed(&self, tx_hash: &str) -> Result<bool> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM processed_transactions WHERE tx_hash = $1 AND chain = 'polkadot'")
            .bind(tx_hash)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RelayerError::Database {
                message: format!("Failed to check Polkadot tx status: {}", e),
            })?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    /// Mark an Ethereum transaction as processed
    pub async fn mark_ethereum_tx_processed(&self, tx_hash: &str) -> Result<()> {
        sqlx::query("INSERT INTO processed_transactions (tx_hash, chain) VALUES ($1, 'ethereum') ON CONFLICT (tx_hash) DO NOTHING")
            .bind(tx_hash)
            .execute(&self.pool)
            .await
            .map_err(|e| RelayerError::Database {
                message: format!("Failed to mark Ethereum tx as processed: {}", e),
            })?;

        debug!("Marked Ethereum tx as processed: {}", tx_hash);
        Ok(())
    }

    /// Mark a Polkadot transaction as processed
    pub async fn mark_polkadot_tx_processed(&self, tx_hash: &str) -> Result<()> {
        sqlx::query("INSERT INTO processed_transactions (tx_hash, chain) VALUES ($1, 'polkadot') ON CONFLICT (tx_hash) DO NOTHING")
            .bind(tx_hash)
            .execute(&self.pool)
            .await
            .map_err(|e| RelayerError::Database {
                message: format!("Failed to mark Polkadot tx as processed: {}", e),
            })?;

        debug!("Marked Polkadot tx as processed: {}", tx_hash);
        Ok(())
    }

    /// Get last processed Ethereum block
    pub async fn get_last_processed_ethereum_block(&self) -> Result<Option<u64>> {
        let row = sqlx::query("SELECT value FROM bridge_state WHERE key = 'last_ethereum_block'")
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| RelayerError::Database {
                message: format!("Failed to get last Ethereum block: {}", e),
            })?;

        if let Some(row) = row {
            let value: String = row.get("value");
            Ok(Some(value.parse().unwrap_or(0)))
        } else {
            Ok(None)
        }
    }

    /// Set last processed Ethereum block
    pub async fn set_last_processed_ethereum_block(&self, block_number: u64) -> Result<()> {
        sqlx::query(r#"
            INSERT INTO bridge_state (key, value) VALUES ('last_ethereum_block', $1)
            ON CONFLICT (key) DO UPDATE SET value = $1, updated_at = NOW()
        "#)
        .bind(block_number.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| RelayerError::Database {
            message: format!("Failed to set last Ethereum block: {}", e),
        })?;

        Ok(())
    }

    /// Get last processed Polkadot block
    pub async fn get_last_processed_polkadot_block(&self) -> Result<Option<u32>> {
        let row = sqlx::query("SELECT value FROM bridge_state WHERE key = 'last_polkadot_block'")
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| RelayerError::Database {
                message: format!("Failed to get last Polkadot block: {}", e),
            })?;

        if let Some(row) = row {
            let value: String = row.get("value");
            Ok(Some(value.parse().unwrap_or(0)))
        } else {
            Ok(None)
        }
    }

    /// Set last processed Polkadot block
    pub async fn set_last_processed_polkadot_block(&self, block_number: u32) -> Result<()> {
        sqlx::query(r#"
            INSERT INTO bridge_state (key, value) VALUES ('last_polkadot_block', $1)
            ON CONFLICT (key) DO UPDATE SET value = $1, updated_at = NOW()
        "#)
        .bind(block_number.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| RelayerError::Database {
            message: format!("Failed to set last Polkadot block: {}", e),
        })?;

        Ok(())
    }

    /// Get token address by asset ID
    pub async fn get_token_address_by_asset_id(&self, asset_id: u32) -> Result<String> {
        let row = sqlx::query("SELECT ethereum_address FROM token_mappings WHERE polkadot_asset_id = $1")
            .bind(asset_id as i32)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RelayerError::Database {
                message: format!("Failed to get token address for asset {}: {}", asset_id, e),
            })?;

        Ok(row.get("ethereum_address"))
    }

    /// Count processed Ethereum transactions
    pub async fn count_ethereum_processed_txs(&self) -> Result<u64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM processed_transactions WHERE chain = 'ethereum'")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RelayerError::Database {
                message: format!("Failed to count Ethereum transactions: {}", e),
            })?;

        Ok(row.get::<i64, _>("count") as u64)
    }

    /// Count processed Polkadot transactions
    pub async fn count_polkadot_processed_txs(&self) -> Result<u64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM processed_transactions WHERE chain = 'polkadot'")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RelayerError::Database {
                message: format!("Failed to count Polkadot transactions: {}", e),
            })?;

        Ok(row.get::<i64, _>("count") as u64)
    }

    /// Close database connections
    pub async fn close(&self) -> Result<()> {
        self.pool.close().await;
        info!("Database connections closed");
        Ok(())
    }
}
