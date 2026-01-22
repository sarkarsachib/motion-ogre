//! Error types and result handling for media operations

use std::fmt;
use thiserror::Error;

/// Result type for media operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for media operations
#[derive(Error, Debug)]
pub enum Error {
    /// File not found or inaccessible
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Unsupported codec
    #[error("Unsupported codec: {0}")]
    UnsupportedCodec(String),

    /// Invalid media format
    #[error("Invalid media format: {0}")]
    InvalidFormat(String),

    /// Decoding failed
    #[error("Decoding failed: {0}")]
    DecodeError(String),

    /// Encoding failed
    #[error("Encoding failed: {0}")]
    EncodeError(String),

    /// Seek operation failed
    #[error("Seek failed: {0}")]
    SeekError(String),

    /// Hardware acceleration not available
    #[error("Hardware acceleration not available: {0}")]
    HardwareAccelerationError(String),

    /// Invalid frame
    #[error("Invalid frame: {0}")]
    InvalidFrame(String),

    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Proxy generation failed
    #[error("Proxy generation failed: {0}")]
    ProxyError(String),

    /// Cache error
    #[error("Cache error: {0}")]
    CacheError(String),

    /// Audio processing error
    #[error("Audio processing error: {0}")]
    AudioError(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

impl Error {
    /// Create a file not found error
    pub fn file_not_found(path: impl AsRef<str>) -> Self {
        Error::FileNotFound(path.as_ref().to_string())
    }

    /// Create an unsupported codec error
    pub fn unsupported_codec(codec: impl AsRef<str>) -> Self {
        Error::UnsupportedCodec(codec.as_ref().to_string())
    }

    /// Create an invalid format error
    pub fn invalid_format(details: impl AsRef<str>) -> Self {
        Error::InvalidFormat(details.as_ref().to_string())
    }

    /// Create a decode error
    pub fn decode_error(details: impl AsRef<str>) -> Self {
        Error::DecodeError(details.as_ref().to_string())
    }

    /// Create an encode error
    pub fn encode_error(details: impl AsRef<str>) -> Self {
        Error::EncodeError(details.as_ref().to_string())
    }

    /// Create a seek error
    pub fn seek_error(details: impl AsRef<str>) -> Self {
        Error::SeekError(details.as_ref().to_string())
    }

    /// Create a hardware acceleration error
    pub fn hwaccel_error(details: impl AsRef<str>) -> Self {
        Error::HardwareAccelerationError(details.as_ref().to_string())
    }

    /// Create an invalid frame error
    pub fn invalid_frame(details: impl AsRef<str>) -> Self {
        Error::InvalidFrame(details.as_ref().to_string())
    }

    /// Create an invalid parameter error
    pub fn invalid_parameter(details: impl AsRef<str>) -> Self {
        Error::InvalidParameter(details.as_ref().to_string())
    }

    /// Create a proxy error
    pub fn proxy_error(details: impl AsRef<str>) -> Self {
        Error::ProxyError(details.as_ref().to_string())
    }

    /// Create a cache error
    pub fn cache_error(details: impl AsRef<str>) -> Self {
        Error::CacheError(details.as_ref().to_string())
    }

    /// Create an audio error
    pub fn audio_error(details: impl AsRef<str>) -> Self {
        Error::AudioError(details.as_ref().to_string())
    }

    /// Convert to other error
    pub fn other<T: fmt::Display>(msg: T) -> Self {
        Error::Other(msg.to_string())
    }
}