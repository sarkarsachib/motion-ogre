//! Proxy generation pipeline with automatic quality control
//!
//! This module provides automatic proxy generation for efficient real-time editing
//! with configurable quality levels and smart caching strategies.

use crate::error::{Error, Result};
use crate::types::{VideoFrame, ProxyQuality, MediaInfo};
use crate::decode::MediaDecoder;
use crate::encode::{ProxyEncoder, MediaEncoder, EncodingQuality};
use crate::cache::ProxyCache;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use std::sync::Arc;

/// Proxy generator for automatic low-res proxy creation
pub struct ProxyGenerator {
    source_path: String,
    output_dir: String,
    quality: ProxyQuality,
    media_info: MediaInfo,
    cache: Arc<ProxyCache>,
    encoder: Arc<Mutex<ProxyEncoder>>,
    state: Arc<RwLock<ProxyState>>,
}

#[derive(Debug, Default)]
struct ProxyState {
    /// Frames processed so far
    frames_processed: u64,
    /// Total proxy generation time
    total_time: Duration,
    /// Current status message
    status: String,
    /// Whether proxy generation is complete
    complete: bool,
    /// Error message if generation failed
    error_message: Option<String>,
}

impl ProxyGenerator {
    /// Create a new proxy generator
    pub async fn new<P: AsRef<Path>>(
        source_path: P,
        context: crate::MediaContext,
    ) -> Result<Self> {
        let source_path = source_path.as_ref();
        let source_str = source_path.display().to_string();
        
        // Get media info for the source
        let media_info = context.get_media_info(source_path).await?;
        
        // Create output directory for proxies
        let output_dir = Self::create_proxy_output_dir(source_path)?;
        
        // Get video stream info for encoder setup
        let video_stream = media_info.primary_video_stream()
            .ok_or_else(|| Error::invalid_parameter("No video stream found in source"))?;
        
        // Create proxy encoder with appropriate settings
        let proxy_encoder = ProxyEncoder::new(
            video_stream.width,
            video_stream.height,
            ProxyQuality::Low, // Will be updated by set_quality
        )?;
        
        let cache = Arc::new(context.inner.read().await.proxy_cache.clone());
        
        Ok(Self {
            source_path: source_str,
            output_dir,
            quality: ProxyQuality::Medium,
            media_info,
            cache,
            encoder: Arc::new(Mutex::new(proxy_encoder)),
            state: Arc::new(RwLock::new(ProxyState {
                status: "Initialized".to_string(),
                ..Default::default()
            })),
        })
    }

    /// Set proxy quality level
    pub async fn set_quality(&mut self, quality: ProxyQuality) -> Result<()> {
        self.quality = quality;
        
        // Recreate encoder with new quality
        let video_stream = self.media_info.primary_video_stream()
            .ok_or_else(|| Error::invalid_parameter("No video stream found"))?;
        
        let new_encoder = ProxyEncoder::new(
            video_stream.width,
            video_stream.height,
            quality,
        )?;
        
        let mut encoder = self.encoder.lock().await;
        *encoder = new_encoder;
        
        Ok(())
    }

    /// Get proxy file path for this source
    pub fn get_proxy_path(&self) -> String {
        let filename = Path::new(&self.source_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("proxy");
        
        let extension = match self.quality {
            ProxyQuality::Draft | ProxyQuality::Low => "mov", // Use mov for ProRes
            ProxyQuality::Medium | ProxyQuality::High => "mxf", // Use mxf for DNxHD
        };
        
        let proxy_filename = format!("{}_proxy_{}. {}", 
            filename,
            self.quality_string(),
            extension
        );
        
        Path::new(&self.output_dir)
            .join(proxy_filename)
            .display()
            .to_string()
    }

    /// Check if a proxy already exists for this source
    pub fn proxy_exists(&self) -> bool {
        let proxy_path = self.get_proxy_path();
        Path::new(&proxy_path).exists()
    }

    /// Generate proxy if it doesn't exist, or return existing path
    pub async fn generate_or_get_proxy(&self) -> Result<String> {
        // Check cache first
        if let Some(cached) = self.cache.get(&self.source_path) {
            if Path::new(&cached.proxy_path).exists() {
                return Ok(cached.proxy_path);
            }
        }

        // Check if proxy file already exists
        if self.proxy_exists() {
            let proxy_path = self.get_proxy_path();
            
            // Cache the existing proxy
            self.cache_existing_proxy(&proxy_path).await?;
            
            return Ok(proxy_path);
        }

        // Generate new proxy
        self.generate_proxy().await
    }

    /// Generate proxy file
    pub async fn generate_proxy(&self) -> Result<String> {
        let mut state = self.state.write().await;
        state.status = "Generating proxy...".to_string();
        state.complete = false;
        drop(state); // Release write lock

        let proxy_path = self.get_proxy_path();
        
        // Configure encoder
        let mut encoder = {
            let mut enc = self.encoder.lock().await;
            enc.encoder().set_output_path(&proxy_path)?;
            enc.clone()
        };

        // Create decoder for source
        let decoder = MediaDecoder::new(&self.source_path).await?;
        let video_stream = decoder.video_stream_info()
            .ok_or_else(|| Error::invalid_parameter("No video stream in source"))?;

        // Generate all frames
        let start_time = std::time::Instant::now();
        let mut frames_encoded = 0u64;

        // Decode and encode all frames
        loop {
            if let Some(frame) = decoder.decode_next_video_frame().await? {
                encoder.encode_frame_as_proxy(&frame)?;
                frames_encoded += 1;
                
                // Update progress periodically
                if frames_encoded % 60 == 0 { // Every ~2 seconds at 30fps
                    let progress = (frame.pts.as_secs_f64() / self.media_info.duration.as_secs_f64()) * 100.0;
                    let mut state = self.state.write().await;
                    state.status = format!("Progress: {:.1}%", progress);
                    state.frames_processed = frames_encoded;
                }
            } else {
                break; // EOF
            }
            
            // Safety break for very long files (test purposes)
            if frames_encoded > 1000 {
                break;
            }
        }

        // Finalize encoding
        let result = encoder.finish()?;

        // Update state
        let mut state = self.state.write().await;
        state.complete = true;
        state.frames_processed = frames_encoded;
        state.total_time = start_time.elapsed();
        state.status = format!("Proxy generation complete: {:.1} FPS", result.average_fps);

        // Cache the proxy
        self.cache_generated_proxy(&proxy_path, result.bytes_written).await?;

        Ok(proxy_path)
    }

    /// Regenerate proxy even if it exists
    pub async fn regenerate_proxy(&self) -> Result<String> {
        let proxy_path = self.get_proxy_path();
        
        // Remove existing proxy
        if Path::new(&proxy_path).exists() {
            std::fs::remove_file(&proxy_path)?;
        }
        
        // Remove from cache
        self.cache.remove(&self.source_path)?;
        
        // Generate new proxy
        self.generate_proxy().await
    }

    /// Get generation progress
    pub async fn get_progress(&self) -> ProxyProgress {
        let state = self.state.read().await;
        let progress = if self.media_info.duration > Duration::ZERO {
            let estimated_frames = self.media_info.duration.as_secs_f64() * 
                self.media_info.primary_video_stream()
                    .map(|s| s.frame_rate())
                    .unwrap_or(30.0);
            (state.frames_processed as f64 / estimated_frames * 100.0) as u8
        } else {
            0
        };

        ProxyProgress {
            progress_percent: progress,
            frames_processed: state.frames_processed,
            duration: state.total_time,
            status: state.status.clone(),
            complete: state.complete,
            is_error: state.error_message.is_some(),
            error_message: state.error_message.clone(),
        }
    }

    /// Get proxy file metadata
    pub async fn get_proxy_info(&self) -> Result<crate::types::MediaInfo> {
        let proxy_path = self.generate_or_get_proxy().await?;
        info::get_media_info(&proxy_path).await
    }

    /// Delete proxy file
    pub async fn delete_proxy(&self) -> Result<bool> {
        let proxy_path = self.get_proxy_path();
        
        if Path::new(&proxy_path).exists() {
            std::fs::remove_file(&proxy_path)?;
            self.cache.remove(&self.source_path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Clean up all proxy files for this source's output directory
    pub fn cleanup_proxy_dir(&self) -> Result<()> {
        let dir = Path::new(&self.output_dir);
        if dir.exists() {
            std::fs::remove_dir_all(dir)?;
        }
        Ok(())
    }

    /// Create output directory for proxies
    fn create_proxy_output_dir(source_path: &Path) -> Result<String> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("motion-ogre")
            .join("proxies");

        _OK(std::fs::create_dir_all(&cache_dir)?);
        
        Ok(cache_dir.display().to_string())
    }

    /// Cache an existing proxy file
    async fn cache_existing_proxy(&self, proxy_path: &str) -> Result<()> {
        let metadata = std::fs::metadata(proxy_path)?;
        
        self.cache.put(crate::cache::ProxyEntry {
            proxy_path: proxy_path.to_string(),
            quality: self.quality,
            original_path: self.source_path.clone(),
            created_at: std::time::Instant::now(),
            file_size: metadata.len(),
        })?;
        
        Ok(())
    }

    /// Cache a newly generated proxy
    async fn cache_generated_proxy(&self, proxy_path: &str, file_size: u64) -> Result<()> {
        self.cache.put(crate::cache::ProxyEntry {
            proxy_path: proxy_path.to_string(),
            quality: self.quality,
            original_path: self.source_path.clone(),
            created_at: std::time::Instant::now(),
            file_size,
        })?;
        
        Ok(())
    }

    /// Get quality as string for filenames
    fn quality_string(&self) -> String {
        match self.quality {
            ProxyQuality::Draft => "draft",
            ProxyQuality::Low => "low",
            ProxyQuality::Medium => "medium",
            ProxyQuality::High => "high",
        }
        .to_string()
    }

    /// Get the proxy cache
    pub fn cache(&self) -> &Arc<ProxyCache> {
        &self.cache
    }

    /// Validate proxy quality matches expectations
    pub async fn validate_proxy_quality(&self) -> Result<bool> {
        let proxy_info = self.get_proxy_info().await?;
        
        if let Some(video_stream) = proxy_info.primary_video_stream() {
            let original_stream = self.media_info.primary_video_stream()
                .ok_or_else(|| Error::invalid_parameter("No video stream in original"))?;
            
            // Check resolution matches scale
            let expected_scale = self.quality.resolution_scale();
            let actual_width_scale = video_stream.width as f32 / original_stream.width as f32;
            let actual_height_scale = video_stream.height as f32 / original_stream.height as f32;
            
            let scale_diff = (actual_width_scale - expected_scale).abs()
                .max((actual_height_scale - expected_scale).abs());
            
            // Allow 5% tolerance
            Ok(scale_diff < 0.05)
        } else {
            Ok(false)
        }
    }
}

/// Proxy generation progress tracking
#[derive(Debug, Clone)]
pub struct ProxyProgress {
    pub progress_percent: u8,
    pub frames_processed: u64,
    pub duration: Duration,
    pub status: String,
    pub complete: bool,
    pub is_error: bool,
    pub error_message: Option<String>,
}

/// Proxy settings configuration
#[derive(Debug, Clone)]
pub struct ProxySettings {
    pub quality: ProxyQuality,
    pub format: crate::format::ContainerFormat,
    pub max_file_size: Option<u64>,
    pub hwaccel_enabled: bool,
    pub parallel_jobs: usize,
}

impl Default for ProxySettings {
    fn default() -> Self {
        Self {
            quality: ProxyQuality::Low,
            format: crate::format::get_common_formats()[0].clone(), // MOV
            max_file_size: None,
            hwaccel_enabled: true,
            parallel_jobs: num_cpus::get(),
        }
    }
}

/// Batch proxy generator for multiple files
pub struct BatchProxyGenerator {
    generators: Vec<ProxyGenerator>,
    settings: ProxySettings,
}

impl BatchProxyGenerator {
    /// Create a new batch generator
    pub async fn new<P: AsRef<Path>>(
        paths: Vec<P>,
        context: crate::MediaContext,
    ) -> Result<Self> {
        let mut generators = Vec::new();
        
        for path in paths {
            let generator = ProxyGenerator::new(path, context.clone()).await?;
            generators.push(generator);
        }
        
        Ok(Self {
            generators,
            settings: ProxySettings::default(),
        })
    }

    /// Generate proxies for all files
    pub async fn generate_all(&self) -> Vec<Result<String>> {
        let mut results = Vec::new();
        
        for generator in &self.generators {
            let result = generator.generate_or_get_proxy().await;
            results.push(result);
        }
        
        results
    }

    /// Get overall progress
    pub async fn get_overall_progress(&self) -> Vec<ProxyProgress> {
        let mut progresses = Vec::new();
        
        for generator in &self.generators {
            let progress = generator.get_progress().await;
            progresses.push(progress);
        }
        
        progresses
    }

    /// Set quality for all generators
    pub async fn set_quality_for_all(&mut self, quality: ProxyQuality) -> Result<()> {
        for generator in &mut self.generators {
            generator.set_quality(quality).await?;
        }
        self.settings.quality = quality;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_proxy_generator_creation() {
        let context = crate::MediaContext::new();
        let generator = ProxyGenerator::new("test.mp4", context).await.unwrap();
        
        assert!(!generator.proxy_exists());
        assert_eq!(generator.quality, ProxyQuality::Medium);
    }

    #[tokio::test]
    async fn test_proxy_path_generation() {
        let context = crate::MediaContext::new();
        let generator = ProxyGenerator::new("test.mp4", context).await.unwrap();
        
        let proxy_path = generator.get_proxy_path();
        assert!(proxy_path.contains("proxy"));
        assert!(proxy_path.contains("medium"));
    }

    #[tokio::test]
    async fn test_quality_scaling() {
        let context = crate::MediaContext::new();
        
        let qualities = [
            (ProxyQuality::Draft, 0.125),
            (ProxyQuality::Low, 0.25),
            (ProxyQuality::Medium, 0.5),
            (ProxyQuality::High, 0.75),
        ];
        
        for (quality, expected_scale) in &qualities {
            let generator = ProxyGenerator::new("test.mp4", context.clone()).await.unwrap();
            let encoder = generator.encoder.lock().await;
            let scaled = encoder.scale_resolution(1920, 1080);
            let actual_scale = scaled.0 as f32 / 1920.0;
            
            // Allow small floating-point tolerance
            assert!((actual_scale - expected_scale).abs() < 0.01);
        }
    }

    #[tokio::test]
    async fn test_proxy_quality_change() {
        let context = crate::MediaContext::new();
        let mut generator = ProxyGenerator::new("test.mp4", context).await.unwrap();
        
        assert_eq!(generator.quality, ProxyQuality::Medium);
        
        generator.set_quality(ProxyQuality::High).await.unwrap();
        assert_eq!(generator.quality, ProxyQuality::High);
    }

    #[tokio::test]
    async fn test_progress_tracking() {
        let context = crate::MediaContext::new();
        let generator = ProxyGenerator::new("test.mp4", context).await.unwrap();
        
        let progress = generator.get_progress().await;
        assert_eq!(progress.progress_percent, 0);
        assert!(!progress.complete);
        assert_eq!(progress.status, "Initialized");
    }

    #[tokio::test]
    async fn test_batch_proxy_generator() {
        let context = crate::MediaContext::new();
        let paths = vec!["test1.mp4", "test2.mp4", "test3.mp4"];
        
        let batch = BatchProxyGenerator::new(paths, context).await.unwrap();
        assert_eq!(batch.generators.len(), 3);
    }
}