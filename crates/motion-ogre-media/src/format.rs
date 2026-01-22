//! Format detection and validation
//!
//! This module provides utilities for detecting media formats, validating file
//! formats, and managing format-specific encoding parameters.

use crate::error::{Error, Result};
use crate::types::{VideoCodec, AudioCodec, PixelFormat, ProxyQuality, EncodingQuality, ColorSpace};
use std::path::Path;

/// Detect video codec from file extension or format string
pub fn detect_video_codec(path: &Path) -> Result<VideoCodec> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "mov" | "mp4" => Ok(VideoCodec::H264), // Default assumption
        "m4v" => Ok(VideoCodec::H264),
        "mkv" => Ok(VideoCodec::H265), // Default assumption
        "avi" => Ok(VideoCodec::H264), // Default assumption
        "mxf" => Ok(VideoCodec::Dnxhd), // Common for DNxHD
        _ => Err(Error::invalid_format(format!("Unknown codec for extension: {}", extension)))
    }
}

/// Detect audio codec from file extension
pub fn detect_audio_codec(path: &Path) -> Result<AudioCodec> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "mp3" => Ok(AudioCodec::Mp3),
        "m4a" | "aac" => Ok(AudioCodec::Aac),
        "wav" => Ok(AudioCodec::Pcm),
        _ => Ok(AudioCodec::Other(extension.clone()))
    }
}

/// Get optimal pixel format for a given codec
pub fn get_optimal_pixel_format(codec: &VideoCodec) -> PixelFormat {
    match codec {
        VideoCodec::ProRes => PixelFormat::Yuv422P10,
        VideoCodec::Dnxhd => PixelFormat::Yuv422P,
        VideoCodec::H264 => PixelFormat::Yuv420P,
        VideoCodec::H265 => PixelFormat::Yuv420P10,
        _ => PixelFormat::Yuv420P
    }
}

/// Get recommended bitrate for proxy generation
pub fn get_proxy_bitrate(original_bitrate: u64, quality: ProxyQuality) -> u64 {
    let factor = quality.bitrate_scale();
    (original_bitrate as f64 * factor).max(500_000.0) as u64 // Minimum 500Kbps
}

/// Get encoder parameters for given quality and codec
pub fn get_encoder_parameters(
    codec: &VideoCodec,
    quality: EncodingQuality,
    resolution: (u32, u32),
    fps: f64,
) -> EncoderParameters {
    let base_bitrate = (resolution.0 * resolution.1 * fps as u32) / 1000; // pixels per second / 1000

    let adjusted_bitrate = match codec {
        VideoCodec::ProRes => (base_bitrate as f64 * 1.5 * quality.bitrate_multiplier() * 1000.0) as u64,
        VideoCodec::Dnxhd => (base_bitrate as f64 * 1.2 * quality.bitrate_multiplier() * 1000.0) as u64,
        VideoCodec::H264 => (base_bitrate as f64 * 0.8 * quality.bitrate_multiplier() * 1000.0) as u64,
        VideoCodec::H265 => (base_bitrate as f64 * 0.6 * quality.bitrate_multiplier() * 1000.0) as u64,
        _ => (base_bitrate as f64 * quality.bitrate_multiplier() * 1000.0) as u64,
    };

    EncoderParameters {
        bitrate: adjusted_bitrate,
        crf: quality.crf(),
        pixel_format: get_optimal_pixel_format(codec),
        keyframe_interval: get_keyframe_interval(fps),
    }
}

/// Encoder parameters
#[derive(Debug, Clone)]
pub struct EncoderParameters {
    /// Target bitrate in bits per second
    pub bitrate: u64,
    /// Constant Rate Factor (for H.264/H.265)
    pub crf: u8,
    /// Pixel format
    pub pixel_format: PixelFormat,
    /// Keyframe interval in frames
    pub keyframe_interval: u32,
}

/// Get keyframe interval based on frame rate
pub fn get_keyframe_interval(fps: f64) -> u32 {
    if fps > 0.0 {
        (fps * 2.0) as u32 // 2 seconds worth of frames
    } else {
        60 // Default fallback
    }
}

/// Validate that a codec is supported for given operation
pub fn validate_codec_support(codec: &VideoCodec, operation: CodecOperation) -> Result<()> {
    match operation {
        CodecOperation::Decode => {
            if !codec.is_supported_for_decode() {
                return Err(Error::unsupported_codec(format!("Codec not supported for decoding: {:?}", codec)));
            }
        }
        CodecOperation::Encode => {
            if !codec.is_supported_for_encode() {
                return Err(Error::unsupported_codec(format!("Codec not supported for encoding: {:?}", codec)));
            }
        }
    }
    Ok(())
}

/// Types of codec operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecOperation {
    /// Decode operation
    Decode,
    /// Encode operation
    Encode,
}

/// Container format information
#[derive(Debug, Clone)]
pub struct ContainerFormat {
    /// Format name
    pub name: String,
    /// Common extensions
    pub extensions: Vec<String>,
    /// Whether format supports video
    pub supports_video: bool,
    /// Whether format supports audio
    pub supports_audio: bool,
    /// Whether format supports metadata
    pub supports_metadata: bool,
}

impl ContainerFormat {
    /// Create a new container format info
    pub fn new(name: &str, extensions: Vec<&str>, supports_video: bool, supports_audio: bool, supports_metadata: bool) -> Self {
        Self {
            name: name.to_string(),
            extensions: extensions.iter().map(|s| s.to_string()).collect(),
            supports_video,
            supports_audio,
            supports_metadata,
        }
    }

    /// Check if this format supports the given extension
    pub fn supports_extension(&self, extension: &str) -> bool {
        self.extensions.iter().any(|ext| ext.eq_ignore_ascii_case(extension))
    }
}

/// Get common container formats
pub fn get_common_formats() -> Vec<ContainerFormat> {
    vec![
        ContainerFormat::new("QuickTime", vec!["mov", "qt"], true, true, true),
        ContainerFormat::new("MP4", vec!["mp4", "m4v", "m4a"], true, true, true),
        ContainerFormat::new("Matroska", vec!["mkv", "mka"], true, true, true),
        ContainerFormat::new("AVI", vec!["avi"], true, true, true),
        ContainerFormat::new("MXF", vec!["mxf"], true, true, true),
        ContainerFormat::new("WAV", vec!["wav"], false, true, true),
        ContainerFormat::new("MP3", vec!["mp3"], false, true, true),
    ]
}

/// Detect container format from file path
pub fn detect_container_format(path: &Path) -> Result<ContainerFormat> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| Error::invalid_format("No file extension"))?;

    let formats = get_common_formats();
    
    for format in formats {
        if format.supports_extension(extension) {
            return Ok(format);
        }
    }

    Err(Error::invalid_format(format!("Unknown container format for extension: {}", extension)))
}

/// Validate file extension for given codecs
pub fn validate_file_extension_for_codec(path: &Path, video_codec: &VideoCodec, audio_codec: Option<&AudioCodec>) -> Result<()> {
    let format = detect_container_format(path)?;
    
    if !format.supports_video && video_codec != &VideoCodec::Raw {
        return Err(Error::invalid_format(format!("Container format {} does not support video", format.name)));
    }

    if let Some(audio_codec) = audio_codec {
        if !format.supports_audio && audio_codec != &AudioCodec::Raw {
            return Err(Error::invalid_format(format!("Container format {} does not support audio", format.name)));
        }
    }

    Ok(())
}

/// ProRes profile settings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProResProfile {
    /// ProRes 422 Proxy
    Proxy = 0,
    /// ProRes 422 LT
    Lt = 1,
    /// ProRes 422 Standard
    Standard = 2,
    /// ProRes 422 HQ
    Hq = 3,
    /// ProRes 4444
    _4444 = 4,
    /// ProRes 4444 XQ
    Xq = 5,
}

impl ProResProfile {
    /// Get bitrate target for profile (in megabits per second)
    pub fn bitrate_target(&self, resolution: (u32, u32), fps: f64) -> u64 {
        let pixel_rate = (resolution.0 * resolution.1) as f64 * fps;
        let base_rate = pixel_rate * self.bitrate_factor();
        (base_rate / 1000000.0) as u64
    }

    /// Get bitrate factor for profile
    fn bitrate_factor(&self) -> f64 {
        match self {
            ProResProfile::Proxy => 0.1,
            ProResProfile::Lt => 0.5,
            ProResProfile::Standard => 1.0,
            ProResProfile::Hq => 1.5,
            ProResProfile::_4444 => 2.0,
            ProResProfile::Xq => 2.5,
        }
    }
}

/// DNxHD profile settings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnxhdProfile {
    /// DNxHD 36
    _36 = 36,
    /// DNxHD 145
    _145 = 145,
    /// DNxHD 220
    _220 = 220,
    /// DNxHD 444
    _444 = 0,
}

impl DnxhdProfile {
    /// Get bitrate for profile (in megabits per second)
    pub fn bitrate_mbps(&self) -> u64 {
        match self {
            DnxhdProfile::_36 => 36,
            DnxhdProfile::_145 => 145,
            DnxhdProfile::_220 => 220,
            DnxhdProfile::_444 => 0, // Variable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_video_codec() {
        assert_eq!(
            detect_video_codec(Path::new("test.mp4")).unwrap(),
            VideoCodec::H264
        );
        assert_eq!(
            detect_video_codec(Path::new("test.mov")).unwrap(),
            VideoCodec::H264
        );
        assert_eq!(
            detect_video_codec(Path::new("test.mxf")).unwrap(),
            VideoCodec::Dnxhd
        );
    }

    #[test]
    fn test_optimal_pixel_format() {
        assert_eq!(get_optimal_pixel_format(&VideoCodec::ProRes), PixelFormat::Yuv422P10);
        assert_eq!(get_optimal_pixel_format(&VideoCodec::Dnxhd), PixelFormat::Yuv422P);
        assert_eq!(get_optimal_pixel_format(&VideoCodec::H264), PixelFormat::Yuv420P);
    }

    #[test]
    fn test_encoder_parameters() {
        let params = get_encoder_parameters(
            &VideoCodec::H264,
            EncodingQuality::Medium,
            (1920, 1080),
            30.0,
        );
        
        assert!(params.bitrate > 0);
        assert_eq!(params.crf, 23);
    }

    #[test]
    fn test_container_format_detection() {
        let format = detect_container_format(Path::new("test.mp4")).unwrap();
        assert_eq!(format.name, "MP4");
        assert!(format.supports_video);
        assert!(format.supports_audio);
    }

    #[test]
    fn test_prores_profile_bitrate() {
        let profile = ProResProfile::Hq;
        let bitrate = profile.bitrate_target((1920, 1080), 30.0);
        assert!(bitrate > 0);
    }
}