use crate::types::{CameraDeviceInfo, CameraFormat};

/// Device information alias
pub type DeviceInfo = CameraDeviceInfo;
/// Format information alias
pub type FormatInfo = CameraFormat;

/// Buffer management policy for captured frames
#[derive(Debug, Clone)]
pub enum BufferPolicy {
    /// Drop the oldest frame when buffer is full
    DropOldest {
        /// Maximum number of frames to hold
        capacity: usize
    },
}

/// Audio capture mode configuration
#[derive(Debug, Clone)]
pub enum AudioMode {
    /// Audio capture disabled
    Disabled,
    /// Audio capture enabled
    Enabled,
}

/// Configuration for headless capture session
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    /// Camera device ID to capture from
    pub device_id: String,
    /// Desired camera format (resolution, fps)
    pub format: CameraFormat,
    /// Buffer management policy
    pub buffer_policy: BufferPolicy,
    /// Audio capture mode
    pub audio_mode: AudioMode,
    /// Optional specific audio device ID
    pub audio_device_id: Option<String>,
}

impl CaptureConfig {
    /// Create new capture configuration with defaults
    pub fn new(device_id: String, format: CameraFormat) -> Self {
        Self {
            device_id,
            format,
            buffer_policy: BufferPolicy::DropOldest { capacity: 2 },
            audio_mode: AudioMode::Disabled,
            audio_device_id: None,
        }
    }
}

/// A captured video frame in headless mode
#[derive(Debug, Clone, serde::Serialize)]
pub struct Frame {
    /// Monotonically increasing sequence number
    pub sequence: u64,
    /// Timestamp in microseconds
    pub timestamp_us: u64,
    /// Frame width in pixels
    pub width: u32,
    /// Frame height in pixels
    pub height: u32,
    /// Pixel format string (e.g. "RGB8", "YUYV")
    pub format: String,
    /// Source device ID
    pub device_id: String,
    /// Raw frame data
    pub data: Vec<u8>,
}

/// A captured audio packet
#[derive(Debug, Clone)]
pub struct AudioPacket {
    /// Monotonically increasing sequence number
    pub sequence: u64,
    /// Timestamp in microseconds
    pub timestamp_us: u64,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of audio channels
    pub channels: u16,
    /// Audio format string (e.g., "F32", "I16")
    pub format: String,
    /// Raw audio data
    pub data: Vec<u8>,
}
