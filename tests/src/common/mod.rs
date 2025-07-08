//! Common testing utilities

pub mod setup;
pub mod mock_data;
pub mod assertions;

use std::time::Duration;
use tokio::time::timeout;

/// Test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub ethereum_rpc_url: String,
    pub polkadot_ws_url: String,
    pub database_url: String,
    pub test_timeout: Duration,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            ethereum_rpc_url: "http://localhost:8545".to_string(),
            polkadot_ws_url: "ws://localhost:9944".to_string(),
            database_url: "postgresql://bridge_user:bridge_pass@localhost:5432/bridge_test".to_string(),
            test_timeout: Duration::from_secs(30),
        }
    }
}

/// Test result type
pub type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Run a test with timeout
pub async fn with_timeout<F, T>(test_fn: F) -> TestResult<T>
where
    F: std::future::Future<Output = TestResult<T>>,
{
    timeout(Duration::from_secs(30), test_fn)
        .await
        .map_err(|_| -> Box<dyn std::error::Error + Send + Sync> { "Test timed out".into() })?
}

/// Wait for a condition to be true
pub async fn wait_for_condition<F>(mut condition: F, max_wait: Duration) -> TestResult<()>
where
    F: FnMut() -> bool,
{
    let start = std::time::Instant::now();
    
    while start.elapsed() < max_wait {
        if condition() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    Err("Condition not met within timeout".into())
}

/// Generate a random test identifier
pub fn generate_test_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("test_{}", rng.gen::<u32>())
}

/// Test environment setup and cleanup
pub struct TestEnvironment {
    pub config: TestConfig,
    _cleanup: Vec<Box<dyn FnOnce() + Send>>,
}

impl TestEnvironment {
    pub fn new() -> Self {
        Self {
            config: TestConfig::default(),
            _cleanup: Vec::new(),
        }
    }

    pub async fn setup(&mut self) -> TestResult<()> {
        // Setup test environment
        // This would include:
        // - Starting local blockchain nodes
        // - Setting up test database
        // - Deploying test contracts
        // - Initializing test data
        
        tracing::info!("Setting up test environment");
        Ok(())
    }

    pub async fn cleanup(&mut self) -> TestResult<()> {
        // Cleanup test environment
        tracing::info!("Cleaning up test environment");
        Ok(())
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        // Ensure cleanup happens even if test panics
        for cleanup_fn in self._cleanup.drain(..) {
            cleanup_fn();
        }
    }
}
