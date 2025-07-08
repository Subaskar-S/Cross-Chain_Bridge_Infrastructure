//! Mock data for testing

use serde_json::json;

/// Mock Ethereum transaction data
pub fn mock_ethereum_lock_event() -> serde_json::Value {
    json!({
        "user": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        "token": "0xA0b86a33E6441e6e80D0c4C34F4F6cA4C7C7B0c8",
        "amount": "1000000000000000000",
        "polkadot_address": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "tx_hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        "block_number": 12345,
        "nonce": 1
    })
}

/// Mock Polkadot burn event data
pub fn mock_polkadot_burn_event() -> serde_json::Value {
    json!({
        "burner": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "asset_id": 1,
        "amount": "1000000000000000000",
        "ethereum_recipient": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
        "tx_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        "block_number": 6789
    })
}

/// Mock validator data
pub fn mock_validators() -> Vec<serde_json::Value> {
    vec![
        json!({
            "id": "validator_0",
            "ethereum_address": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
            "polkadot_address": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
            "stake": "1000",
            "active": true
        }),
        json!({
            "id": "validator_1",
            "ethereum_address": "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC",
            "polkadot_address": "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
            "stake": "1000",
            "active": true
        }),
        json!({
            "id": "validator_2",
            "ethereum_address": "0x90F79bf6EB2c4f870365E785982E1f101E93b906",
            "polkadot_address": "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y",
            "stake": "1000",
            "active": true
        }),
    ]
}

/// Mock token data
pub fn mock_token_mapping() -> serde_json::Value {
    json!({
        "ethereum_address": "0xA0b86a33E6441e6e80D0c4C34F4F6cA4C7C7B0c8",
        "polkadot_asset_id": 1,
        "name": "Test Token",
        "symbol": "TEST",
        "decimals": 18
    })
}

/// Mock signature data
pub fn mock_threshold_signatures() -> Vec<Vec<u8>> {
    vec![
        vec![1u8; 65], // Mock signature 1
        vec![2u8; 65], // Mock signature 2
        vec![3u8; 65], // Mock signature 3
    ]
}

/// Mock bridge statistics
pub fn mock_bridge_stats() -> serde_json::Value {
    json!({
        "ethereum_processed_txs": 100,
        "polkadot_processed_txs": 95,
        "pending_signatures": 2,
        "active_validators": 3,
        "total_volume": "1000000000000000000000",
        "uptime": 99.5
    })
}

/// Generate mock private keys for testing
pub fn mock_private_keys() -> Vec<String> {
    vec![
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        "1123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        "2123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
    ]
}

/// Generate mock transaction hashes
pub fn mock_transaction_hashes() -> Vec<String> {
    vec![
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        "0x2234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        "0x3234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
    ]
}
