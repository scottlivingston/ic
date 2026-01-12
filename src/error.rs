//! Unified error types for the IC application.

use thiserror::Error;

/// Application-wide error type
#[derive(Error, Debug)]
pub enum IcError {
    #[error("Audio error: {0}")]
    Audio(String),

    #[error("Config error: {0}")]
    Config(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("WiFi error: {0}")]
    Wifi(#[from] crate::wifi::WifiError),
}
