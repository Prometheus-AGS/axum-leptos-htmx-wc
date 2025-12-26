//! Error types for the SDK.

use thiserror::Error;

/// SDK error type.
#[derive(Error, Debug)]
pub enum Error {
    /// HTTP request failed.
    #[error("HTTP error: {0}")]
    #[cfg(feature = "http-client")]
    Http(#[from] reqwest::Error),

    /// Invalid URL.
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// API returned an error response.
    #[error("API error ({status}): {message}")]
    Api {
        /// HTTP status code.
        status: u16,
        /// Error message from the API.
        message: String,
    },

    /// Configuration error (embedded mode).
    #[error("Configuration error: {0}")]
    Config(String),

    /// Runtime error (embedded mode).
    #[error("Runtime error: {0}")]
    Runtime(String),

    /// Stream ended unexpectedly.
    #[error("Stream ended unexpectedly")]
    StreamEnded,
}

/// Result type alias for SDK operations.
pub type Result<T> = std::result::Result<T, Error>;
