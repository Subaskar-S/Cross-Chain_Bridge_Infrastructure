//! Error types for the API service

use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Result type for API operations
pub type Result<T> = std::result::Result<T, ApiError>;

/// Errors that can occur in the API service
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Internal server error: {message}")]
    Internal { message: String },

    #[error("Relayer error: {0}")]
    Relayer(#[from] relayer::RelayerError),

    #[error("Threshold signature error: {0}")]
    ThresholdSignature(#[from] threshold::ThresholdError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::Config { message } => (StatusCode::INTERNAL_SERVER_ERROR, message),
            ApiError::Validation { message } => (StatusCode::BAD_REQUEST, message),
            ApiError::NotFound { resource } => (StatusCode::NOT_FOUND, format!("Not found: {}", resource)),
            ApiError::Internal { message } => (StatusCode::INTERNAL_SERVER_ERROR, message),
            ApiError::Relayer(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ApiError::ThresholdSignature(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

        let body = Json(json!({
            "error": status.canonical_reason().unwrap_or("Unknown"),
            "message": error_message,
        }));

        (status, body).into_response()
    }
}
