//! WebSocket handlers

use crate::{error::Result, server::ApiState};
use axum::{extract::Extension, response::Response};

pub async fn websocket_handler(
    Extension(_state): Extension<ApiState>,
) -> Result<Response> {
    // Placeholder for WebSocket upgrade
    Ok(Response::builder()
        .status(501)
        .body("WebSocket not implemented yet".into())
        .unwrap())
}

pub async fn events_websocket(
    Extension(_state): Extension<ApiState>,
) -> Result<Response> {
    Ok(Response::builder()
        .status(501)
        .body("Events WebSocket not implemented yet".into())
        .unwrap())
}

pub async fn stats_websocket(
    Extension(_state): Extension<ApiState>,
) -> Result<Response> {
    Ok(Response::builder()
        .status(501)
        .body("Stats WebSocket not implemented yet".into())
        .unwrap())
}
