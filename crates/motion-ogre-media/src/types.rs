//! Common types and data structures for media operations

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Video codec types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VideoCodec {
    /// H.264/AVC
    H264,
    /// H.265/HEVC
    H265,
    /// Apple ProRes
    ProRes,
    /// Avid DNxHD
    Dnxhd,
    /// MJPEG
    Mjpeg,
    /// Uncompressed
    Raw,
    /// Other codec
    Other(String),
}

impl VideoCodec {
    /// Check if this codec is supported for decoding
    pub fn is_supported_for_decode(&self) -> bool {
        matches!(
            self,
            VideoCodec::H264 | VideoCodec::H265 | VideoCodec::ProRes | VideoCodec::Dnxhd | VideoCodec::Mjpeg | VideoCodec::Raw
        )
    }

    /// Check if this codec is supported for encoding
    pub fn is_supported_for_encode(&self) -> bool {
        matches!(
            self,
            VideoCodec::H264 | VideoCodec::H265 | VideoCodec::ProRes | VideoCodec::Dnxhd | VideoCodec::Mjpeg
        )
    }

    /// Get codec name as string
    pub fn as_str(&self) -> &str {
        match self {
            VideoCodec::H264 => "h264",
            VideoCodec::H265 => "h265",
            VideoCodec::ProRes => "prores",
            VideoCodec::Dnxhd => "dnxhd",
            VideoCodec::Mjpeg => "mjpeg",
            VideoCodec::Raw => "raw",
            VideoCodec::Other(name) => name.as_str(),
        }
    }

    /// Create from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "h264" | "avc" => VideoCodec::H264,
            "h265" | "hevc" => VideoCodec::H265,
            "prores" => VideoCodec::ProRes,
            "dnxhd" => VideoCodec::Dnxhd,
            "mjpeg" => VideoCodec::Mjpeg,
            "raw" => VideoCodec::Raw,
            other => VideoCodec::Other(other.to_string()),
        }
    }
}

/// Audio codec types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AudioCodec {
    /// AAC
    Aac,
    /// MP3
    Mp3,
    /// PCM
    Pcm,
    /// AC-3
    Ac3,
    /// Uncompressed
    Raw,
    /// Other codec
    Other(String),
}

impl AudioCodec {
    /// Get codec name as string
    pub fn as_str(&self) -> &str {
        match self {
            AudioCodec::Aac => "aac",
            AudioCodec::Mp3 => "mp3",
            AudioCodec::Pcm => "pcm",
            AudioCodec::Ac3 => "ac3",
            AudioCodec::Raw => "raw",
            AudioCodec::Other(name) => name.as_str(),
        }
    }

    /// Create from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "aac" => AudioCodec::Aac,
            "mp3" => AudioCodec::Mp3,
            "pcm" => AudioCodec::Pcm,
            "ac3" => AudioCodec::Ac3,
            "raw" => AudioCodec::Raw,
            other => AudioCodec::Other(other.to_string()),
        }
    }
}

/// Pixel format for video frames
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PixelFormat {
    /// 8-bit YUV 4:2:0
    Yuv420P,
    /// 10-bit YUV 4:2:0
    Yuv420P10,
    /// 8-bit RGB
    Rgb24,
    /// 8-bit RGBA
    Rgba,
    /// 8-bit BGRA (GPU-friendly)
    Bgra,
    /// 10-bit RGB
    Rgb48,
    /// 16-bit RGB
    Rgb64,
    /// Other format
    Other(String),
}

impl PixelFormat {
    /// Get bytes per pixel
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            PixelFormat::Yuv420P => 1,
            PixelFormat::Yuv420P10 => 2,
            PixelFormat::Rgb24 => 3,
            PixelFormat::Rgba => 4,
            PixelFormat::Bgra => 4,
            PixelFormat::Rgb48 => 6,
            PixelFormat::Rgb64 => 8,
            PixelFormat::Other(_) => 4, // Default assumption
        }
    }

    /// Check if format is GPU-friendly
    pub fn is_gpu_friendly(&self) -> bool {
        matches!(self, PixelFormat::Rgba | PixelFormat::Bgra | PixelFormat::Rgb24)
    }
}

/// Color space for video
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColorSpace {
    /// BT.601 (SD)
    Bt601,
    /// BT.709 (HD)
    Bt709,
    /// BT.2020 (UHD)
    Bt2020,
    /// BT.2100 (HDR)
    Bt2100,
    /// Unknown
    Unknown,
}

impl ColorSpace {
    /// Convert to string
    pub fn as_str(&self) -> &str {
        match self {
            ColorSpace::Bt601 => "bt601",
            ColorSpace::Bt709 => "bt709",
            ColorSpace::Bt2020 => "bt2020",
            ColorSpace::Bt2100 => "bt2100",
            ColorSpace::Unknown => "unknown",
        }
    }
}

/// Video frame data
#[derive(Debug, Clone)]
pub struct VideoFrame {
    /// Frame width
    pub width: u32,
    /// Frame height
    pub height: u32,
    /// Pixel format
    pub format: PixelFormat,
    /// Frame data
    pub data: Vec<u8>,
    /// Presentation timestamp
    pub pts: Duration,
    /// Decode timestamp
    pub dts: Option<Duration>,
    /// Duration of frame
    pub duration: Duration,
    /// Keyframe flag
    pub key_frame: bool,
}

impl VideoFrame {
    /// Create a new video frame
    pub fn new(
        width: u32,
        height: u32,
        format: PixelFormat,
        data: Vec<u8>,
        pts: Duration,
        dts: Option<Duration>,
        duration: Duration,
        key_frame: bool,
    ) -> Self {
        Self {
            width,
            height,
            format,
            data,
            pts,
            dts,
            duration,
            key_frame,
        }
    }

    /// Get frame size in bytes
    pub fn size_in_bytes(&self) -> usize {
        self.data.len()
    }

    /// Check if frame is valid
    pub fn is_valid(&self) -> bool {
        !self.data.is_empty() && self.width > 0 && self.height > 0
    }
}

/// Audio frame data
#[derive(Debug, Clone)]
pub struct AudioFrame {
    /// Sample rate
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
    /// Bit depth
    pub bit_depth: u16,
    /// Frame data
    pub data: Vec<i16>,
    /// Presentation timestamp
    pub pts: Duration,
    /// Duration of frame
    pub duration: Duration,
}

impl AudioFrame {
    /// Create a new audio frame
    pub fn new(
        sample_rate: u32,
        channels: u16,
        bit_depth: u16,
        data: Vec<i16>,
        pts: Duration,
        duration: Duration,
    ) -> Self {
        Self {
            sample_rate,
            channels,
            bit_depth,
            data,
            pts,
            duration,
        }
    }

    /// Get number of samples
    pub fn sample_count(&self) -> usize {
        self.data.len() / self.channels as usize
    }

    /// Check if frame is valid
    pub fn is_valid(&self) -> bool {
        !self.data.is_empty() && self.sample_rate > 0 && self.channels > 0
    }
}

/// Media information for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    /// Container format
    pub format: String,
    /// Duration
    pub duration: Duration,
    /// Bitrate
    pub bitrate: u64,
    /// Video stream information
    pub video_streams: Vec<VideoStreamInfo>,
    /// Audio stream information
    pub audio_streams: Vec<AudioStreamInfo>,
    /// Metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl MediaInfo {
    /// Create new media info
    pub fn new(
        format: String,
        duration: Duration,
        bitrate: u64,
        video_streams: Vec<VideoStreamInfo>,
        audio_streams: Vec<AudioStreamInfo>,
        metadata: std::collections::HashMap<String, String>,
    ) -> Self {
        Self {
            format,
            duration,
            bitrate,
            video_streams,
            audio_streams,
            metadata,
        }
    }

    /// Check if this is a video file
    pub fn has_video(&self) -> bool {
        !self.video_streams.is_empty()
    }

    /// Check if this is an audio file
    pub fn has_audio(&self) -> bool {
        !self.audio_streams.is_empty()
    }

    /// Get primary video stream
    pub fn primary_video_stream(&self) -> Option<&VideoStreamInfo> {
        self.video_streams.first()
    }

    /// Get primary audio stream
    pub fn primary_audio_stream(&self) -> Option<&AudioStreamInfo> {
        self.audio_streams.first()
    }
}

/// Video stream information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoStreamInfo {
    /// Stream index
    pub index: usize,
    /// Video codec
    pub codec: VideoCodec,
    /// Resolution width
    pub width: u32,
    /// Resolution height
    pub height: u32,
    /// Frame rate numerator
    pub fps_numerator: u32,
    /// Frame rate denominator
    pub fps_denominator: u32,
    /// Pixel format
    pub pixel_format: String,
    /// Color space
    pub color_space: ColorSpace,
    /// Bitrate
    pub bitrate: u64,
    /// GOP size
    pub gop_size: u32,
}

impl VideoStreamInfo {
    /// Calculate frame rate
    pub fn frame_rate(&self) -> f64 {
        if self.fps_denominator > 0 {
            self.fps_numerator as f64 / self.fps_denominator as f64
        } else {
            0.0
        }
    }

    /// Get frame duration
    pub fn frame_duration(&self) -> Duration {
        if self.fps_numerator > 0 {
            Duration::from_secs_f64(1.0 / self.frame_rate())
        } else {
            Duration::ZERO
        }
    }
}

/// Audio stream information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStreamInfo {
    /// Stream index
    pub index: usize,
    /// Audio codec
    pub codec: AudioCodec,
    /// Sample rate
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
    /// Bit depth
    pub bit_depth: u16,
    /// Bitrate
    pub bitrate: u64,
}

impl AudioStreamInfo {
    /// Calculate bytes per frame
    pub fn bytes_per_frame(&self) -> usize {
        (self.bit_depth as usize / 8) * self.channels as usize
    }
}

/// Proxy quality levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProxyQuality {
    /// Lowest quality for very fast preview
    Draft,
    /// Low quality for fast scrubbing
    Low,
    /// Medium quality for reasonable preview
    Medium,
    /// High quality for detailed preview
    High,
}

impl ProxyQuality {
    /// Get resolution scale factor for this quality
    pub fn resolution_scale(&self) -> f32 {
        match self {
            ProxyQuality::Draft => 0.125, // 1/8 resolution
            ProxyQuality::Low => 0.25,    // 1/4 resolution
            ProxyQuality::Medium => 0.5,    // 1/2 resolution
            ProxyQuality::High => 0.75,     // 3/4 resolution
        }
    }

    /// Get bitrate scale factor for this quality
    pub fn bitrate_scale(&self) -> f32 {
        match self {
            ProxyQuality::Draft => 0.1,
            ProxyQuality::Low => 0.25,
            ProxyQuality::Medium => 0.5,
            ProxyQuality::High => 0.75,
        }
    }
}

/// Encoding quality levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncodingQuality {
    /// Very low quality for fast encoding
    VeryLow,
    /// Low quality for faster encoding
    Low,
    /// Medium quality for balanced encoding
    Medium,
    /// High quality for professional results
    High,
    /// Very high quality for archival
    VeryHigh,
}

impl EncodingQuality {
    /// Get CRF value for H.264/H.265 encoding
    pub fn crf(&self) -> u8 {
        match self {
            EncodingQuality::VeryLow => 30,
            EncodingQuality::Low => 28,
            EncodingQuality::Medium => 23,
            EncodingQuality::High => 18,
            EncodingQuality::VeryHigh => 15,
        }
    }

    /// Get bitrate multiplier for VBR encoding
    pub fn bitrate_multiplier(&self) -> f32 {
        match self {
            EncodingQuality::VeryLow => 0.5,
            EncodingQuality::Low => 0.75,
            EncodingQuality::Medium => 1.0,
            EncodingQuality::High => 1.5,
            EncodingQuality::VeryHigh => 2.0,
        }
    }
}