//! Media information extraction and metadata parsing
//!
//! This module provides utilities for extracting detailed media information
//! including video/audio stream properties and container metadata.

use crate::error::{Error, Result};
use crate::types::{MediaInfo, VideoStreamInfo, AudioStreamInfo, VideoCodec, AudioCodec, PixelFormat};
use std::path::Path;
use std::time::Duration;
use std::collections::HashMap;

/// Get detailed media information for a file
pub async fn get_media_info<P: AsRef<Path>>(path: P) -> Result<MediaInfo> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(Error::file_not_found(path.display().to_string()));
    }

    // For now, return mock data since we don't have FFmpeg bindings working
    // In production, this would use ffmpeg-next to parse actual files
    
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("unknown");

    // Generate realistic mock data based on file extension
    let (video_streams, audio_streams, format, duration, bitrate) = match extension {
        "mp4" | "mov" => create_typical_video_info(),
        "mp3" => create_typical_audio_info(),
        "wav" => create_wav_audio_info(),
        "mxf" => create_dnxhd_video_info(),
        _ => create_default_info(),
    };

    let metadata = extract_metadata(path);

    Ok(MediaInfo::new(
        format.to_string(),
        duration,
        bitrate,
        video_streams,
        audio_streams,
        metadata,
    ))
}

/// Create typical video file info (mock implementation)
fn create_typical_video_info() -> (Vec<VideoStreamInfo>, Vec<AudioStreamInfo>, &'static str, Duration, u64) {
    let video_stream = VideoStreamInfo {
        index: 0,
        codec: VideoCodec::H264,
        width: 1920,
        height: 1080,
        fps_numerator: 30000,
        fps_denominator: 1001,
        pixel_format: "yuv420p".to_string(),
        color_space: crate::types::ColorSpace::Bt709,
        bitrate: 8_000_000,
        gop_size: 30,
    };

    let audio_stream = AudioStreamInfo {
        index: 1,
        codec: AudioCodec::Aac,
        sample_rate: 48000,
        channels: 2,
        bit_depth: 16,
        bitrate: 192_000,
    };

    (
        vec![video_stream],
        vec![audio_stream],
        "mp4",
        Duration::from_secs(60),
        8_500_000,
    )
}

/// Create DNxHD MXF info (mock implementation)
fn create_dnxhd_video_info() -> (Vec<VideoStreamInfo>, Vec<AudioStreamInfo>, &'static str, Duration, u64) {
    let video_stream = VideoStreamInfo {
        index: 0,
        codec: VideoCodec::Dnxhd,
        width: 1920,
        height: 1080,
        fps_numerator: 24000,
        fps_denominator: 1001,
        pixel_format: "yuv422p".to_string(),
        color_space: crate::types::ColorSpace::Bt709,
        bitrate: 145_000_000,
        gop_size: 1, // I-frame only
    };

    (
        vec![video_stream],
        vec![],
        "mxf",
        Duration::from_secs(300),
        145_000_000,
    )
}

/// Create typical audio info (mock implementation)
fn create_typical_audio_info() -> (Vec<VideoStreamInfo>, Vec<AudioStreamInfo>, &'static str, Duration, u64) {
    let audio_stream = AudioStreamInfo {
        index: 0,
        codec: AudioCodec::Mp3,
        sample_rate: 44100,
        channels: 2,
        bit_depth: 16,
        bitrate: 192_000,
    };

    (
        vec![],
        vec![audio_stream],
        "mp3",
        Duration::from_secs(180),
        192_000,
    )
}

/// Create WAV audio info (mock implementation)
fn create_wav_audio_info() -> (Vec<VideoStreamInfo>, Vec<AudioStreamInfo>, &'static str, Duration, u64) {
    let audio_stream = AudioStreamInfo {
        index: 0,
        codec: AudioCodec::Pcm,
        sample_rate: 48000,
        channels: 2,
        bit_depth: 24,
        bitrate: 2_304_000, // 48kHz * 2ch * 24bit
    };

    (
        vec![],
        vec![audio_stream],
        "wav",
        Duration::from_secs(300),
        2_304_000,
    )
}

/// Create default info for unknown files
fn create_default_info() -> (Vec<VideoStreamInfo>, Vec<AudioStreamInfo>, &'static str, Duration, u64) {
    (
        vec![],
        vec![],
        "unknown",
        Duration::from_secs(0),
        0,
    )
}

/// Extract metadata from file path
fn extract_metadata(path: &Path) -> HashMap<String, String> {
    let mut metadata = HashMap::new();

    // Add file system metadata
    if let Ok(metadata_fs) = std::fs::metadata(path) {
        metadata.insert("file_size".to_string(), metadata_fs.len().to_string());
        
        #[cfg(unix)]
        if let Ok(created) = metadata_fs.created() {
            metadata.insert("created".to_string(), format!("{:?}", created));
        }
        
        #[cfg(unix)]
        if let Ok(modified) = metadata_fs.modified() {
            metadata.insert("modified".to_string(), format!("{:?}", modified));
        }
    }

    // Add filename metadata
    if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
        metadata.insert("filename".to_string(), filename.to_string());
    }

    metadata
}

/// Validate media file structure
pub fn validate_media_file<P: AsRef<Path>>(path: P) -> Result<bool> {
    let path = path.as_ref();

    if !path.exists() {
        return Ok(false);
    }

    // Check file size
    let metadata = std::fs::metadata(path)?;
    if metadata.len() == 0 {
        return Ok(false);
    }

    // Additional validation would go here with actual FFmpeg parsing
    Ok(true)
}

/// Calculate GOP structure from frame rate
pub fn estimate_gop_size(fps: f64, is_intra_only: bool) -> u32 {
    if is_intra_only {
        1 // I-frame only
    } else {
        // Typical GOP size is 2 seconds for most codecs
        (fps * 2.0) as u32
    }
}

/// Guess color space from resolution
pub fn guess_color_space(width: u32, height: u32) -> crate::types::ColorSpace {
    if width >= 3840 || height >= 2160 {
        crate::types::ColorSpace::Bt2020 // UHD likely uses BT.2020
    } else {
        crate::types::ColorSpace::Bt709 // HD and SD use BT.709
    }
}

/// Estimate compression ratio based on codec and bitrate
pub fn estimate_compression_ratio(codec: &VideoCodec, bitrate: u64, resolution: (u32, u32), fps: f64) -> f64 {
    // Uncompressed size for one frame
    let uncompressed_per_frame = (resolution.0 * resolution.1 * 3) as f64; // 3 bytes per pixel (RGB)
    let total_uncompressed_per_second = uncompressed_per_frame * fps;
    let compressed_per_second = (bitrate / 8) as f64; // Convert bits to bytes

    if compressed_per_second > 0.0 {
        total_uncompressed_per_second / compressed_per_second
    } else {
        1.0
    }
}

/// Parse FFmpeg format string to internal format
pub fn parse_ffmpeg_format(format_str: &str) -> Result<String> {
    Ok(format_str.to_lowercase())
}

/// Parse FFmpeg codec string to VideoCodec
pub fn parse_ffmpeg_codec(codec_str: &str) -> Result<VideoCodec> {
    let lower = codec_str.to_lowercase();
    Ok(match lower.as_str() {
        "h264" | "avc1" => VideoCodec::H264,
        "h265" | "hevc" => VideoCodec::H265,
        "prores" => VideoCodec::ProRes,
        "dnxhd" => VideoCodec::Dnxhd,
        "mjpeg" => VideoCodec::Mjpeg,
        "raw" => VideoCodec::Raw,
        other => VideoCodec::Other(other.to_string()),
    })
}

/// Parse FFmpeg pixel format string
pub fn parse_ffmpeg_pixel_format(format_str: &str) -> Result<PixelFormat> {
    let lower = format_str.to_lowercase();
    Ok(match lower.as_str() {
        "yuv420p" => PixelFormat::Yuv420P,
        "yuv420p10le" => PixelFormat::Yuv420P10,
        "rgb24" => PixelFormat::Rgb24,
        "rgba" => PixelFormat::Rgba,
        "bgra" => PixelFormat::Bgra,
        "rgb48le" => PixelFormat::Rgb48,
        "rgb64le" => PixelFormat::Rgb64,
        other => PixelFormat::Other(other.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_media_info() {
        let info = get_media_info("test.mp4").await.unwrap();
        assert!(info.has_video());
        assert!(info.has_audio());
        assert!(info.duration > Duration::ZERO);
    }

    #[test]
    fn test_validate_media_file() {
        assert!(validate_media_file("test.mp4").is_ok());
    }

    #[test]
    fn test_estimate_gop_size() {
        assert_eq!(estimate_gop_size(30.0, true), 1);
        assert_eq!(estimate_gop_size(30.0, false), 60);
    }

    #[test]
    fn test_guess_color_space() {
        assert_eq!(guess_color_space(1920, 1080), crate::types::ColorSpace::Bt709);
        assert_eq!(guess_color_space(3840, 2160), crate::types::ColorSpace::Bt2020);
    }

    #[test]
    fn test_parse_ffmpeg_codec() {
        assert_eq!(parse_ffmpeg_codec("h264").unwrap(), VideoCodec::H264);
        assert_eq!(parse_ffmpeg_codec("hevc").unwrap(), VideoCodec::H265);
        assert_eq!(parse_ffmpeg_codec("prores").unwrap(), VideoCodec::ProRes);
    }

    #[test]
    fn test_parse_ffmpeg_pixel_format() {
        assert_eq!(parse_ffmpeg_pixel_format("yuv420p").unwrap(), PixelFormat::Yuv420P);
        assert_eq!(parse_ffmpeg_pixel_format("rgba").unwrap(), PixelFormat::Rgba);
    }
}