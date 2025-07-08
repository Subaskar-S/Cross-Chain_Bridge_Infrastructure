//! Test assertion utilities

use super::TestResult;
use serde_json::Value;

/// Assert that a value is within a range
pub fn assert_in_range<T>(value: T, min: T, max: T, message: &str) -> TestResult<()>
where
    T: PartialOrd + std::fmt::Debug,
{
    if value >= min && value <= max {
        Ok(())
    } else {
        Err(format!("{}: {:?} not in range [{:?}, {:?}]", message, value, min, max).into())
    }
}

/// Assert that a JSON value contains expected fields
pub fn assert_json_contains(actual: &Value, expected_fields: &[&str]) -> TestResult<()> {
    for field in expected_fields {
        if !actual.get(field).is_some() {
            return Err(format!("Missing field: {}", field).into());
        }
    }
    Ok(())
}

/// Assert that a transaction hash is valid format
pub fn assert_valid_tx_hash(tx_hash: &str) -> TestResult<()> {
    if tx_hash.starts_with("0x") && tx_hash.len() == 66 {
        Ok(())
    } else {
        Err(format!("Invalid transaction hash format: {}", tx_hash).into())
    }
}

/// Assert that an address is valid Ethereum format
pub fn assert_valid_ethereum_address(address: &str) -> TestResult<()> {
    if address.starts_with("0x") && address.len() == 42 {
        Ok(())
    } else {
        Err(format!("Invalid Ethereum address format: {}", address).into())
    }
}

/// Assert that an address is valid Polkadot format
pub fn assert_valid_polkadot_address(address: &str) -> TestResult<()> {
    if address.len() == 48 {
        Ok(())
    } else {
        Err(format!("Invalid Polkadot address format: {}", address).into())
    }
}

/// Assert that a signature is valid format
pub fn assert_valid_signature(signature: &[u8]) -> TestResult<()> {
    if signature.len() == 64 || signature.len() == 65 {
        Ok(())
    } else {
        Err(format!("Invalid signature length: {}", signature.len()).into())
    }
}

/// Assert that bridge statistics are reasonable
pub fn assert_valid_bridge_stats(stats: &Value) -> TestResult<()> {
    let required_fields = [
        "ethereum_processed_txs",
        "polkadot_processed_txs", 
        "pending_signatures",
        "active_validators"
    ];
    
    assert_json_contains(stats, &required_fields)?;
    
    // Check that values are non-negative
    if let Some(eth_txs) = stats["ethereum_processed_txs"].as_u64() {
        assert_in_range(eth_txs, 0, u64::MAX, "ethereum_processed_txs")?;
    }
    
    if let Some(dot_txs) = stats["polkadot_processed_txs"].as_u64() {
        assert_in_range(dot_txs, 0, u64::MAX, "polkadot_processed_txs")?;
    }
    
    if let Some(pending) = stats["pending_signatures"].as_u64() {
        assert_in_range(pending, 0, 1000, "pending_signatures")?;
    }
    
    if let Some(validators) = stats["active_validators"].as_u64() {
        assert_in_range(validators, 1, 100, "active_validators")?;
    }
    
    Ok(())
}

/// Assert that two values are approximately equal (for floating point comparisons)
pub fn assert_approx_eq(a: f64, b: f64, tolerance: f64, message: &str) -> TestResult<()> {
    if (a - b).abs() <= tolerance {
        Ok(())
    } else {
        Err(format!("{}: {} != {} (tolerance: {})", message, a, b, tolerance).into())
    }
}

/// Assert that an operation completes within a time limit
pub async fn assert_completes_within<F, T>(
    operation: F,
    timeout: std::time::Duration,
    message: &str,
) -> TestResult<T>
where
    F: std::future::Future<Output = TestResult<T>>,
{
    match tokio::time::timeout(timeout, operation).await {
        Ok(result) => result,
        Err(_) => Err(format!("{}: operation timed out after {:?}", message, timeout).into()),
    }
}
