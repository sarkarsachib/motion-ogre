//! # Motion Ogre Media Library
//!
//! A comprehensive media handling library providing codec support, proxy generation,
//! and frame serving for real-time video editing workflows.
//!
//! ## Features
//!
//! - **FFmpeg Integration**: Safe bindings to FFmpeg for codec support
//! - **Multiple Codecs**: H.264, H.265, ProRes, DNxHD support
//! - **Proxy Generation**: Automatic low-res proxy creation
//! - **Frame Serving**: Efficient frame extraction with exact seeking
//! - **Audio Processing**: Extract and resample audio tracks
//! - **Hardware Acceleration**: GPU decoding where available
//!
//! ## Modules
//!
//! - `decode`: Video/audio decoding and frame extraction
//! - `encode`: Video/audio encoding with quality control
//! - `proxy`: Automatic proxy generation pipeline
//! - `info`: Media metadata extraction
//! - `audio`: Audio processing and waveform generation
//! - `error`: Error types and handling
//! - `types`: Common types and data structures

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![forbid(unsafe_code)]

pub mod decode;
pub mod encode;
pub mod proxy;
pub mod info;
pub mod audio;
pub mod error;
pub mod types;
pub mod cache;
pub mod format;

pub use error::{Error, Result};
pub use types::*;

use std::path::Path;
use tokio::sync::RwLock;
use std::sync::Arc;

/// Media context for file operations
#[derive(Debug, Clone)]
pub struct MediaContext {
    inner: Arc<RwLock<MediaContextInner>>,
}

#[derive(Debug)]
struct MediaContextInner {
    frame_cache: cache::FrameCache,
    proxy_cache: proxy::ProxyCache,
    hardware_acceleration: bool,
    max_threads: usize,
}

impl MediaContext {
    /// Create a new media context
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(MediaContextInner {
                frame_cache: cache::FrameCache::new(100),
                proxy_cache: proxy::ProxyCache::new(10),
                hardware_acceleration: true,
                max_threads: num_cpus::get(),
            })),
        }
    }

    /// Get media information for a file
    pub async fn get_media_info<P: AsRef<Path>>(&self, path: P) -> Result<MediaInfo> {
        info::get_media_info(path).await
    }

    /// Create a decoder for a media file
    pub async fn create_decoder<P: AsRef<Path>>(&self, path: P) -> Result<decode::MediaDecoder> {
        decode::MediaDecoder::new(path).await
    }

    /// Create a proxy generator for a media file
    pub async fn create_proxy_generator<P: AsRef<Path>>(&self, path: P) -> Result<proxy::ProxyGenerator> {
        proxy::ProxyGenerator::new(path, self.clone()).await
    }

    /// Create an encoder
    pub fn create_encoder(&self) -> Result<encode::MediaEncoder> {
        encode::MediaEncoder::new()
    }
}