//! Metrics handlers

use crate::{error::Result, server::ApiState};
use axum::{extract::Extension, response::Response};

pub async fn prometheus_metrics(
    Extension(_state): Extension<ApiState>,
) -> Result<Response> {
    let metrics = r#"
# HELP bridge_processed_transactions_total Total number of processed transactions
# TYPE bridge_processed_transactions_total counter
bridge_processed_transactions_total{chain="ethereum"} 100
bridge_processed_transactions_total{chain="polkadot"} 95

# HELP bridge_active_validators Number of active validators
# TYPE bridge_active_validators gauge
bridge_active_validators 3

# HELP bridge_pending_signatures Number of pending signatures
# TYPE bridge_pending_signatures gauge
bridge_pending_signatures 2
"#;

    Ok(Response::builder()
        .header("content-type", "text/plain; version=0.0.4; charset=utf-8")
        .body(metrics.into())
        .unwrap())
}

pub async fn bridge_metrics(
    Extension(_state): Extension<ApiState>,
) -> Result<Response> {
    Ok(Response::builder()
        .header("content-type", "application/json")
        .body(r#"{"metrics": "bridge_specific_metrics"}"#.into())
        .unwrap())
}
