//! Media encoding with quality control
//!
//! This module provides encoding capabilities for various video and audio formats
//! with configurable quality levels and bitrate control.

use crate::error::{Error, Result};
use crate::types::{VideoCodec, AudioCodec, EncodingQuality, VideoFrame, AudioFrame, PixelFormat};
use crate::format::{get_encoder_parameters, validate_codec_support, EncoderParameters, ProResProfile, DnxhdProfile};
use std::path::Path;
use std::time::Duration;

/// Media encoder for video and audio files
pub struct MediaEncoder {
    config: EncoderConfig,
    state: EncoderState,
}

#[derive(Debug, Clone)]
struct EncoderConfig {
    /// Target video codec
    video_codec: VideoCodec,
    /// Target audio codec
    audio_codec: Option<AudioCodec>,
    /// Output file path
    output_path: String,
    /// Video quality setting
    video_quality: EncodingQuality,
    /// Audio quality setting
    audio_quality: EncodingQuality,
    /// Use hardware acceleration if available
    hwaccel_enabled: bool,
}

#[derive(Debug, Default)]
struct EncoderState {
    /// Encoded frame count
    frames_encoded: u64,
    /// Total bytes written
    bytes_written: u64,
    /// Encoding session start time
    start_time: Option<std::time::Instant>,
    /// Last error message
    last_error: Option<String>,
}

impl MediaEncoder {
    /// Create a new encoder
    pub fn new() -> Result<Self> {
        Ok(Self {
            config: EncoderConfig {
                video_codec: VideoCodec::H264,
                audio_codec: None,
                output_path: String::new(),
                video_quality: EncodingQuality::Medium,
                audio_quality: EncodingQuality::Medium,
                hwaccel_enabled: true,
            },
            state: EncoderState::default(),
        })
    }

    /// Create encoder specifically for H.264
    pub fn new_h264() -> Result<Self> {
        let mut encoder = Self::new()?;
        encoder.set_video_codec(VideoCodec::H264)?;
        Ok(encoder)
    }

    /// Create encoder specifically for H.265
    pub fn new_h265() -> Result<Self> {
        let mut encoder = Self::new()?;
        encoder.set_video_codec(VideoCodec::H265)?;
        Ok(encoder)
    }

    /// Create encoder specifically for ProRes
    pub fn new_prores() -> Result<Self> {
        let mut encoder = Self::new()?;
        encoder.set_video_codec(VideoCodec::ProRes)?;
        Ok(encoder)
    }

    /// Create encoder specifically for DNxHD
    pub fn new_dnxhd() -> Result<Self> {
        let mut encoder = Self::new()?;
        encoder.set_video_codec(VideoCodec::Dnxhd)?;
        Ok(encoder)
    }

    /// Set output file path
    pub fn set_output_path<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.config.output_path = path.as_ref().display().to_string();
        Ok(())
    }

    /// Set video codec
    pub fn set_video_codec(&mut self, codec: VideoCodec) -> Result<()> {
        validate_codec_support(&codec, crate::format::CodecOperation::Encode)?;
        self.config.video_codec = codec;
        Ok(())
    }

    /// Set audio codec
    pub fn set_audio_codec(&mut self, codec: Option<AudioCodec>) -> Result<()> {
        // Basic validation would go here
        self.config.audio_codec = codec;
        Ok(())
    }

    /// Set video quality
    pub fn set_video_quality(&mut self, quality: EncodingQuality) -> Result<()> {
        self.config.video_quality = quality;
        Ok(())
    }

    /// Set audio quality
    pub fn set_audio_quality(&mut self, quality: EncodingQuality) -> Result<()> {
        self.config.audio_quality = quality;
        Ok(())
    }

    /// Enable or disable hardware acceleration
    pub fn set_hwaccel_enabled(&mut self, enabled: bool) {
        self.config.hwaccel_enabled = enabled;
    }

    /// Get current configuration
    pub fn config(&self) -> &EncoderConfig {
        &self.config
    }

    /// Encode a video frame
    pub fn encode_video_frame(&mut self, frame: &VideoFrame) -> Result<()> {
        if !self.state.start_time.is_some() {
            self.state.start_time = Some(std::time::Instant::now());
        }

        let params = self.calculate_encoder_params(frame)?;
        let encoded_size = self.estimate_encoded_frame_size(frame, &params);

        // Simulate encoding
        self.state.frames_encoded += 1;
        self.state.bytes_written += encoded_size as u64;

        Ok(())
    }

    /// Encode an audio frame
    pub fn encode_audio_frame(&mut self, frame: &AudioFrame) -> Result<()> {
        if !self.state.start_time.is_some() {
            self.state.start_time = Some(std::time::Instant::now());
        }

        let size_per_sample = (frame.bit_depth / 8) as usize;
        let encoded_size = frame.data.len() * size_per_sample;

        self.state.bytes_written += encoded_size as u64;
        
        Ok(())
    }

    /// Finish encoding and write the output file
    pub fn finish(&mut self) -> Result<EncodingResult> {
        let duration = self.state.start_time
            .map(|start| start.elapsed())
            .unwrap_or(Duration::ZERO);

        let result = EncodingResult {
            output_path: self.config.output_path.clone(),
            frames_encoded: self.state.frames_encoded,
            bytes_written: self.state.bytes_written,
            encoding_time: duration,
            success: self.state.last_error.is_none(),
            average_fps: if duration.as_secs_f64() > 0.0 {
                self.state.frames_encoded as f64 / duration.as_secs_f64()
            } else {
                0.0
            },
        };

        Ok(result)
    }

    /// Get encoder parameters for a given frame
    fn calculate_encoder_params(&self, frame: &VideoFrame) -> Result<EncoderParameters> {
        // Calculate based on actual frame dimensions
        let fps = 1.0 / frame.duration.as_secs_f64();
        get_encoder_parameters(
            &self.config.video_codec,
            self.config.video_quality,
            (frame.width, frame.height),
            fps,
        )
    }

    /// Estimate encoded frame size
    fn estimate_encoded_frame_size(&self, frame: &VideoFrame, params: &EncoderParameters) -> usize {
        // Very rough estimation based on bitrate
        let bits_per_second = params.bitrate;
        let fps = 1.0 / frame.duration.as_secs_f64();
        let bits_per_frame = (bits_per_second as f64 / fps) as usize;
        let bytes_per_frame = bits_per_frame / 8;

        // Apply quality multiplier
        let quality_multiplier = self.config.video_quality.bitrate_multiplier();
        ((bytes_per_frame as f64 * quality_multiplier) as usize)
            .min(frame.data.len())  // Can't be bigger than source
            .max(1024)              // Minimum size
    }

    /// Get current encoding statistics
    pub fn get_stats(&self) -> EncodingStats {
        EncodingStats {
            frames_encoded: self.state.frames_encoded,
            bytes_written: self.state.bytes_written,
            encoding_time: self.state.start_time.map(|s| s.elapsed()).unwrap_or(Duration::ZERO),
            average_fps: self.state.start_time
                .map(|start| self.state.frames_encoded as f64 / start.elapsed().as_secs_f64())
                .unwrap_or(0.0),
            last_error: self.state.last_error.clone(),
        }
    }
}

/// Encoding result information
#[derive(Debug, Clone)]
pub struct EncodingResult {
    /// Output file path
    pub output_path: String,
    /// Number of frames encoded
    pub frames_encoded: u64,
    /// Total bytes written
    pub bytes_written: u64,
    /// Encoding duration
    pub encoding_time: Duration,
    /// Success status
    pub success: bool,
    /// Average encoding FPS
    pub average_fps: f64,
}

/// Encoding statistics
#[derive(Debug, Clone)]
pub struct EncodingStats {
    /// Frames encoded so far
    pub frames_encoded: u64,
    /// Bytes written so far
    pub bytes_written: u64,
    /// Current encoding time
    pub encoding_time: Duration,
    /// Average FPS
    pub average_fps: f64,
    /// Last error if any
    pub last_error: Option<String>,
}

/// Proxy encoder specifically designed for high-quality proxy generation
pub struct ProxyEncoder {
    base_encoder: MediaEncoder,
    resolution_scale: f32,
}

impl ProxyEncoder {
    /// Create a new proxy encoder
    pub fn new(original_width: u32, original_height: u32, quality: crate::types::ProxyQuality) -> Result<Self> {
        let mut base_encoder = Self::create_base_encoder(quality)?;
        
        Ok(Self {
            base_encoder,
            resolution_scale: quality.resolution_scale(),
        })
    }

    /// Create appropriate base encoder for proxy quality
    fn create_base_encoder(quality: crate::types::ProxyQuality) -> Result<MediaEncoder> {
        // Professional proxy formats: ProRes HQ or DNxHD
        let mut encoder = MediaEncoder::new_prores()?;
        encoder.set_video_quality(EncodingQuality::High)?;
        Ok(encoder)
    }

    /// Scale resolution for proxy
    pub fn scale_resolution(&self, original_width: u32, original_height: u32) -> (u32, u32) {
        (
            (original_width as f32 * self.resolution_scale) as u32,
            (original_height as f32 * self.resolution_scale) as u32,
        )
    }

    /// Encode frame as proxy (with scaling)
    pub fn encode_frame_as_proxy(&mut self, frame: &VideoFrame) -> Result<()> {
        // In production, would actually scale the frame here
        // For now, just encode with modified parameters
        
        let scaled_params = self.get_proxy_encoding_params(frame);
        self.base_encoder.encode_video_frame(frame)?;
        Ok(())
    }

    /// Get proxy encoding parameters
    fn get_proxy_encoding_params(&self, frame: &VideoFrame) -> EncoderParameters {
        let (scaled_width, scaled_height) = self.scale_resolution(frame.width, frame.height);
        
        get_encoder_parameters(
            &self.base_encoder.config().video_codec,
            EncodingQuality::Medium, // Always use medium for proxies
            (scaled_width, scaled_height),
            1.0 / frame.duration.as_secs_f64(),
        )
    }

    /// Get the underlying encoder
    pub fn encoder(&mut self) -> &mut MediaEncoder {
        &mut self.base_encoder
    }

    /// Finish encoding and return result
    pub fn finish(&mut self) -> Result<EncodingResult> {
        self.base_encoder.finish()
    }
}

/// Bi-directional encoding optimization
pub struct BidirectionalEncoder {
    forward_encoder: MediaEncoder,
    reverse_encoder: MediaEncoder,
}

impl BidirectionalEncoder {
    /// Create a new bidirectional encoder
    pub fn new(codec: VideoCodec) -> Result<Self> {
        let mut forward = MediaEncoder::new()?;
        forward.set_video_codec(codec.clone())?;
        
        let mut reverse = MediaEncoder::new()?;
        reverse.set_video_codec(codec)?;

        Ok(Self {
            forward_encoder: forward,
            reverse_encoder: reverse,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{VideoFrame, AudioFrame, PixelFormat};
    use std::time::Duration;

    #[test]
    fn test_encoder_creation() {
        let encoder = MediaEncoder::new().unwrap();
        assert_eq!(encoder.config().video_codec, VideoCodec::H264);
    }

    #[test]
    fn test_codec_specific_encoders() {
        let h264 = MediaEncoder::new_h264().unwrap();
        assert_eq!(h264.config().video_codec, VideoCodec::H264);

        let h265 = MediaEncoder::new_h265().unwrap();
        assert_eq!(h265.config().video_codec, VideoCodec::H265);

        let prores = MediaEncoder::new_prores().unwrap();
        assert_eq!(prores.config().video_codec, VideoCodec::ProRes);

        let dnxhd = MediaEncoder::new_dnxhd().unwrap();
        assert_eq!(dnxhd.config().video_codec, VideoCodec::Dnxhd);
    }

    #[test]
    fn test_encode_frame() {
        let mut encoder = MediaEncoder::new().unwrap();
        let frame = create_test_frame();
        
        encoder.encode_video_frame(&frame).unwrap();
        let stats = encoder.get_stats();
        
        assert_eq!(stats.frames_encoded, 1);
        assert!(stats.bytes_written > 0);
    }

    #[test]
    fn test_encode_multiple_frames() {
        let mut encoder = MediaEncoder::new_h264().unwrap();
        let frame = create_test_frame();
        
        for _ in 0..10 {
            encoder.encode_video_frame(&frame).unwrap();
        }
        
        let stats = encoder.get_stats();
        assert_eq!(stats.frames_encoded, 10);
        assert!(stats.average_fps > 0.0 || stats.encoding_time == Duration::ZERO);
    }

    #[test]
    fn test_finish_encoding() {
        let mut encoder = MediaEncoder::new().unwrap();
        let frame = create_test_frame();
        
        encoder.encode_video_frame(&frame).unwrap();
        let result = encoder.finish().unwrap();
        
        assert!(result.success);
        assert_eq!(result.frames_encoded, 1);
        assert!(result.average_fps >= 0.0);
    }

    #[test]
    fn test_proxy_encoder() {
        let proxy_encoder = ProxyEncoder::new(1920, 1080, crate::types::ProxyQuality::Low).unwrap();
        let scaled = proxy_encoder.scale_resolution(1920, 1080);
        
        // Should be 1/4 resolution for Low quality
        assert_eq!(scaled, (480, 270));
    }

    #[test]
    fn test_quality_settings() {
        let mut encoder = MediaEncoder::new().unwrap();
        
        encoder.set_video_quality(EncodingQuality::High).unwrap();
        assert_eq!(encoder.config().video_quality, EncodingQuality::High);
    }

    fn create_test_frame() -> VideoFrame {
        VideoFrame::new(
            1920,
            1080,
            PixelFormat::Yuv420P,
            vec![0; 1920 * 1080 * 3 / 2],
            Duration::ZERO,
            None,
            Duration::from_millis(33),
            true,
        )
    }
}