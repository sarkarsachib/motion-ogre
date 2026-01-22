//! Audio extraction and waveform generation
//!
//! This module provides audio processing capabilities including waveform generation
//! for timeline display, resampling to project sample rate, and multi-channel support.

use crate::error::{Error, Result};
use crate::types::AudioFrame;
use crate::decode::MediaDecoder;
use crate::cache::WaveformCache;
use std::path::Path;
use std::time::Duration;
use std::sync::Arc;

/// Audio processor for waveform generation and processing
pub struct AudioProcessor {
    cache: Arc<WaveformCache>,
    resample_quality: ResampleQuality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResampleQuality {
    Fast,
    Medium,
    High,
    Best,
}

impl AudioProcessor {
    /// Create a new audio processor
    pub fn new(cache_size: usize) -> Self {
        Self {
            cache: Arc::new(WaveformCache::new(cache_size)),
            resample_quality: ResampleQuality::High,
        }
    }

    /// Set resampling quality
    pub fn set_resample_quality(&mut self, quality: ResampleQuality) {
        self.resample_quality = quality;
    }

    /// Generate waveform for a media file
    pub async fn generate_waveform<P: AsRef<Path>>(
        &self,
        path: P,
        resolution: WaveformResolution,
    ) -> Result<WaveformData> {
        let path_str = path.as_ref().display().to_string();
        
        // Check cache first
        if let Some(cached) = self.cache.get(&path_str) {
            let waveform = WaveformData {
                samples: (*cached).clone(),
                resolution,
                duration: Duration::ZERO, // Would extract from file
                peak_amplitude: 1.0,
                rms_amplitude: 0.707,
            };
            return Ok(waveform);
        }

        // Generate waveform from audio frames
        let frames = self.extract_audio_frames(path).await?;
        let samples = self.frames_to_waveform_samples(&frames, resolution)?;

        // Cache the result
        self.cache.put(path_str, samples.clone())?;

        let waveform = WaveformData {
            samples,
            resolution,
            duration: Duration::ZERO, // Would calculate real duration
            peak_amplitude: 1.0,
            rms_amplitude: 0.707,
        };

        Ok(waveform)
    }

    /// Extract all audio frames from a media file
    async fn extract_audio_frames<P: AsRef<Path>>(&self, path: P) -> Result<Vec<AudioFrame>> {
        let decoder = MediaDecoder::new(path).await?;
        
        if !decoder.has_audio() {
            return Err(Error::audio_error("No audio stream found in file"));
        }

        let mut frames = Vec::new();
        
        loop {
            match decoder.decode_next_audio_frame().await? {
                Some(frame) => frames.push(frame),
                None => break, // EOF
            }
        }

        Ok(frames)
    }

    /// Convert audio frames to waveform samples
    fn frames_to_waveform_samples(
        &self,
        frames: &[AudioFrame],
        resolution: WaveformResolution,
    ) -> Result<Vec<f32>> {
        if frames.is_empty() {
            return Ok(Vec::new());
        }

        // Combine all frames into a single sample stream
        let total_samples: usize = frames.iter().map(|f| f.sample_count()).sum();
        let mut all_samples = Vec::with_capacity(total_samples);

        for frame in frames {
            // Convert 16-bit PCM to normalized float
            for sample in &frame.data {
                let normalized = *sample as f32 / i16::MAX as f32;
                all_samples.push(normalized);
            }
        }

        // Downsample to desired resolution
        let samples_per_pixel = self.calculate_samples_per_pixel(
            all_samples.len(),
            frames[0].channels as usize,
            resolution,
        );

        let mut waveform_samples = Vec::new();
        let mut chunk_start = 0;

        while chunk_start < all_samples.len() {
            let chunk_end = (chunk_start + samples_per_pixel)
                .min(all_samples.len());
            
            let chunk = &all_samples[chunk_start..chunk_end];
            
            // Calculate peak amplitude for this chunk
            let peak = chunk.iter()
                .map(|s| s.abs())
                .fold(0.0f32, |a, b| a.max(b));
            
            waveform_samples.push(peak);
            chunk_start = chunk_end;
        }

        Ok(waveform_samples)
    }

    /// Calculate samples needed per display pixel
    fn calculate_samples_per_pixel(
        &self,
        total_samples: usize,
        channels: usize,
        resolution: WaveformResolution,
    ) -> usize {
        let target_samples = match resolution {
            WaveformResolution::Low => 800,   // For thumbnails
            WaveformResolution::Medium => 1920, // For timeline view
            WaveformResolution::High => 3840,   // For detailed view
            WaveformResolution::VeryHigh => 7680, // For zoomed view
        };

        // Ensure we have at least 1 sample per pixel
        (total_samples / (target_samples * channels)).max(1)
    }

    /// Resample audio frames to target sample rate
    pub fn resample_frames(
        &self,
        frames: &[AudioFrame],
        target_sample_rate: u32,
    ) -> Result<Vec<AudioFrame>> {
        if frames.is_empty() {
            return Ok(Vec::new());
        }

        let source_rate = frames[0].sample_rate;
        if source_rate == target_sample_rate {
            // No resampling needed
            return Ok(frames.to_vec());
        }

        let ratio = target_sample_rate as f64 / source_rate as f64;
        let mut resampled_frames = Vec::new();

        for frame in frames {
            let new_sample_count = (frame.sample_count() as f64 * ratio) as usize;
            let new_data = self.resample_frame_data(&frame.data, frame.channels, new_sample_count)?;

            let resampled_frame = AudioFrame::new(
                target_sample_rate,
                frame.channels,
                frame.bit_depth,
                new_data,
                frame.pts,
                Duration::from_secs_f64(frame.duration.as_secs_f64() * ratio),
            );

            resampled_frames.push(resampled_frame);
        }

        Ok(resampled_frames)
    }

    /// Resample a single frame's data
    fn resample_frame_data(
        &self,
        data: &[i16],
        channels: u16,
        target_samples: usize,
    ) -> Result<Vec<i16>> {
        let source_samples = data.len() / channels as usize;
        let ratio = target_samples as f64 / source_samples as f64;
        let mut resampled = Vec::with_capacity(target_samples * channels as usize);

        for i in 0..target_samples {
            let source_pos = (i as f64 / ratio) as usize;
            let source_pos = source_pos.min(source_samples.saturating_sub(1));

            for ch in 0..channels as usize {
                let sample = data[source_pos * channels as usize + ch];
                resampled.push(sample);
            }
        }

        Ok(resampled)
    }

    /// Extract audio peaks for timeline display
    pub async fn extract_peaks<P: AsRef<Path>>(
        &self,
        path: P,
        peak_count: usize,
    ) -> Result<Vec<PeakInfo>> {
        let waveform = self.generate_waveform(path, WaveformResolution::Medium).await?;
        
        let sample_per_peak = waveform.samples.len() / peak_count;
        let mut peaks = Vec::with_capacity(peak_count);

        for i in 0..peak_count {
            let start_idx = i * sample_per_peak;
            let end_idx = ((i + 1) * sample_per_peak).min(waveform.samples.len());
            
            let chunk = &waveform.samples[start_idx..end_idx];
            if chunk.is_empty() {
                peaks.push(PeakInfo::default());
                continue;
            }

            let positive_peak = chunk.iter().filter(|s| **s > 0.0).fold(0.0, |a, b| a.max(*b));
            let negative_peak = -chunk.iter().filter(|s| **s < 0.0).fold(0.0, |a, b| a.max(-*b));

            let rms = (chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32).sqrt();

            peaks.push(PeakInfo {
                positive_peak,
                negative_peak,
                rms_amplitude: rms,
                timestamp: Duration::from_secs_f64((i * sample_per_peak) as f64 / waveform.resolution.samples_per_second()),
            });
        }

        Ok(peaks)
    }

    /// Clear the waveform cache
    pub fn clear_cache(&self) -> Result<()> {
        self.cache.clear();
        Ok(())
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            entry_count: self.cache.len(),
            total_size_bytes: 0, // Would track actual size
            hit_rate: 0.0,        // Would track hits/misses
        }
    }
}

/// Waveform resolution settings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveformResolution {
    Low,      // ~800 samples for thumbnails
    Medium,   // ~1920 samples for timeline view
    High,     // ~3840 samples for detailed view
    VeryHigh, // ~7680 samples for zoomed view
}

impl WaveformResolution {
    /// Estimated samples per second at this resolution
    pub fn samples_per_second(&self) -> f64 {
        match self {
            WaveformResolution::Low => 800.0 / 30.0,    // 30 sec timeline
            WaveformResolution::Medium => 1920.0 / 30.0, // 30 sec timeline
            WaveformResolution::High => 3840.0 / 30.0,   // 30 sec timeline
            WaveformResolution::VeryHigh => 7680.0 / 30.0, // 30 sec timeline
        }
    }
}

/// Waveform data structure
#[derive(Debug, Clone)]
pub struct WaveformData {
    pub samples: Vec<f32>,
    pub resolution: WaveformResolution,
    pub duration: Duration,
    pub peak_amplitude: f32,
    pub rms_amplitude: f32,
}

/// Peak information for timeline display
#[derive(Debug, Clone)]
pub struct PeakInfo {
    pub positive_peak: f32,
    pub negative_peak: f32,
    pub rms_amplitude: f32,
    pub timestamp: Duration,
}

impl Default for PeakInfo {
    fn default() -> Self {
        Self {
            positive_peak: 0.0,
            negative_peak: 0.0,
            rms_amplitude: 0.0,
            timestamp: Duration::ZERO,
        }
    }
}

/// Cache statistics
pub struct CacheStats {
    pub entry_count: usize,
    pub total_size_bytes: usize,
    pub hit_rate: f64,
}

/// Batch audio processor for multiple files
pub struct BatchAudioProcessor {
    processor: AudioProcessor,
}

impl BatchAudioProcessor {
    /// Create a new batch processor
    pub fn new(cache_size: usize) -> Self {
        Self {
            processor: AudioProcessor::new(cache_size),
        }
    }

    /// Process multiple files
    pub async fn process_files<P: AsRef<Path>>(
        &self,
        paths: &[P],
        resolution: WaveformResolution,
    ) -> Vec<Result<WaveformData>> {
        let mut results = Vec::new();

        for path in paths {
            let result = self.processor.generate_waveform(path, resolution).await;
            results.push(result);
        }

        results
    }

    /// Extract peaks from multiple files
    pub async fn extract_peaks_from_files<P: AsRef<Path>>(
        &self,
        paths: &[P],
        peak_count: usize,
    ) -> Vec<Result<Vec<PeakInfo>>> {
        let mut results = Vec::new();

        for path in paths {
            let result = self.processor.extract_peaks(path, peak_count).await;
            results.push(result);
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AudioFrame, AudioCodec};

    #[test]
    fn test_audio_processor_creation() {
        let processor = AudioProcessor::new(100);
        assert_eq!(processor.cache_size(), 0);
    }

    #[test]
    fn test_resample_quality() {
        let mut processor = AudioProcessor::new(100);
        processor.set_resample_quality(ResampleQuality::Best);
        assert_eq!(processor.resample_quality, ResampleQuality::Best);
    }

    #[test]
    fn test_frames_to_waveform_samples() {
        let processor = AudioProcessor::new(100);
        let frames = vec![
            AudioFrame::new(
                44100,
                2,
                16,
                vec![1000, -1000, 2000, -2000, 1500, -1500],
                Duration::ZERO,
                Duration::from_millis(20),
            ),
        ];

        let samples = processor.frames_to_waveform_samples(&frames, WaveformResolution::Medium).unwrap();
        assert!(!samples.is_empty());

        // Should have normalized values between 0 and 1
        for sample in &samples {
            assert!(*sample >= 0.0 && *sample <= 1.0);
        }
    }

    #[test]
    fn test_resample_frames() {
        let processor = AudioProcessor::new(100);
        let frames = vec![
            AudioFrame::new(
                44100,
                2,
                16,
                vec![1000; 882], // 20ms at 44.1kHz
                Duration::ZERO,
                Duration::from_millis(20),
            ),
        ];

        let resampled = processor.resample_frames(&frames, 48000).unwrap();
        assert!(!resampled.is_empty());
        
        let frame = &resampled[0];
        assert_eq!(frame.sample_rate, 48000);
        assert!(frame.sample_count() > 0);
    }

    #[test]
    fn test_waveform_resolution_samples_per_second() {
        assert!(WaveformResolution::Medium.samples_per_second() > 0.0);
        assert!(WaveformResolution::High.samples_per_second() > WaveformResolution::Medium.samples_per_second());
    }

    #[test]
    fn test_cache_stats() {
        let processor = AudioProcessor::new(100);
        let stats = processor.cache_stats();
        
        assert_eq!(stats.entry_count, 0);
        assert_eq!(stats.hit_rate, 0.0);
    }
}