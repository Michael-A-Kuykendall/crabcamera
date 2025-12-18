//! Recording configuration types

use serde::{Deserialize, Serialize};

/// Audio configuration for recording
/// Per #RecorderIntegrateAudio: ! supports_audio_optional
#[cfg(feature = "audio")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Audio device ID (None = default device)
    pub device_id: Option<String>,
    /// Sample rate (must be 48000 for Opus)
    pub sample_rate: u32,
    /// Number of channels (1 or 2)
    pub channels: u16,
    /// Opus bitrate in bits per second
    pub bitrate: u32,
}

#[cfg(feature = "audio")]
impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            device_id: None,
            sample_rate: 48000, // Opus requirement
            channels: 2,
            bitrate: 128_000,
        }
    }
}

#[cfg(feature = "audio")]
impl AudioConfig {
    /// Create audio config for a specific device
    pub fn with_device(device_id: impl Into<String>) -> Self {
        Self {
            device_id: Some(device_id.into()),
            ..Default::default()
        }
    }

    /// Set mono audio
    pub fn mono(mut self) -> Self {
        self.channels = 1;
        self
    }

    /// Set stereo audio
    pub fn stereo(mut self) -> Self {
        self.channels = 2;
        self
    }

    /// Set bitrate
    pub fn with_bitrate(mut self, bitrate: u32) -> Self {
        self.bitrate = bitrate;
        self
    }
}

/// Quality presets for video recording
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RecordingQuality {
    /// 720p at 30fps, lower bitrate - good for previews/streaming
    Low,
    /// 1080p at 30fps, standard bitrate - balanced quality
    Medium,
    /// 1080p at 60fps or 4K at 30fps - high quality
    #[default]
    High,
    /// Custom settings
    Custom,
}

impl RecordingQuality {
    /// Get recommended bitrate in bits per second
    pub fn bitrate(&self) -> u32 {
        match self {
            RecordingQuality::Low => 2_500_000,      // 2.5 Mbps for 720p
            RecordingQuality::Medium => 5_000_000,   // 5 Mbps for 1080p
            RecordingQuality::High => 10_000_000,    // 10 Mbps for high quality
            RecordingQuality::Custom => 5_000_000,   // Default to medium
        }
    }

    /// Get recommended resolution (width, height)
    pub fn resolution(&self) -> (u32, u32) {
        match self {
            RecordingQuality::Low => (1280, 720),
            RecordingQuality::Medium => (1920, 1080),
            RecordingQuality::High => (1920, 1080),
            RecordingQuality::Custom => (1920, 1080),
        }
    }

    /// Get recommended framerate
    pub fn fps(&self) -> f64 {
        match self {
            RecordingQuality::Low => 30.0,
            RecordingQuality::Medium => 30.0,
            RecordingQuality::High => 30.0,  // Can be overridden
            RecordingQuality::Custom => 30.0,
        }
    }
}



/// Configuration for video recording
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingConfig {
    /// Video width in pixels
    pub width: u32,
    /// Video height in pixels
    pub height: u32,
    /// Frames per second
    pub fps: f64,
    /// Target bitrate in bits per second
    pub bitrate: u32,
    /// Quality preset used
    pub quality: RecordingQuality,
    /// Enable fast-start for web streaming (moov before mdat)
    pub fast_start: bool,
    /// Optional title metadata
    pub title: Option<String>,
    /// Audio configuration (None = video only)
    /// Per #RecorderIntegrateAudio: ! supports_audio_optional
    #[cfg(feature = "audio")]
    pub audio: Option<AudioConfig>,
}

impl RecordingConfig {
    /// Create a new recording configuration with explicit dimensions
    pub fn new(width: u32, height: u32, fps: f64) -> Self {
        Self {
            width,
            height,
            fps,
            bitrate: 5_000_000,
            quality: RecordingQuality::Custom,
            fast_start: true,
            title: None,
            #[cfg(feature = "audio")]
            audio: None,
        }
    }

    /// Create configuration from a quality preset
    pub fn from_quality(quality: RecordingQuality) -> Self {
        let (width, height) = quality.resolution();
        Self {
            width,
            height,
            fps: quality.fps(),
            bitrate: quality.bitrate(),
            quality,
            fast_start: true,
            title: None,
            #[cfg(feature = "audio")]
            audio: None,
        }
    }

    /// Create configuration from a quality preset with custom fps
    pub fn from_quality_with_fps(quality: RecordingQuality, fps: f64) -> Self {
        let (width, height) = quality.resolution();
        Self {
            width,
            height,
            fps,
            bitrate: quality.bitrate(),
            quality,
            fast_start: true,
            title: None,
            #[cfg(feature = "audio")]
            audio: None,
        }
    }

    /// Set the title metadata
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set fast-start mode
    pub fn with_fast_start(mut self, enabled: bool) -> Self {
        self.fast_start = enabled;
        self
    }

    /// Set custom bitrate
    pub fn with_bitrate(mut self, bitrate: u32) -> Self {
        self.bitrate = bitrate;
        self
    }

    /// Enable audio recording with the given configuration
    /// Per #RecorderIntegrateAudio: ! supports_audio_optional
    #[cfg(feature = "audio")]
    pub fn with_audio(mut self, audio_config: AudioConfig) -> Self {
        self.audio = Some(audio_config);
        self
    }

    /// Enable audio recording with default configuration (default device, stereo, 128kbps)
    #[cfg(feature = "audio")]
    pub fn with_default_audio(mut self) -> Self {
        self.audio = Some(AudioConfig::default());
        self
    }
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self::from_quality(RecordingQuality::High)
    }
}

/// Statistics returned after finishing a recording
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingStats {
    /// Total number of video frames written
    pub video_frames: u64,
    /// Total number of audio frames written
    pub audio_frames: u64,
    /// Duration in seconds
    pub duration_secs: f64,
    /// Total bytes written to file
    pub bytes_written: u64,
    /// Average frames per second achieved
    pub actual_fps: f64,
    /// Number of dropped frames (if any)
    pub dropped_frames: u64,
    /// Output file path
    pub output_path: String,
}

impl RecordingStats {
    /// Calculate the average bitrate achieved
    pub fn avg_bitrate(&self) -> f64 {
        if self.duration_secs > 0.0 {
            (self.bytes_written as f64 * 8.0) / self.duration_secs
        } else {
            0.0
        }
    }
}
