use crate::types::{CameraDeviceInfo, CameraFormat};

pub type DeviceInfo = CameraDeviceInfo;
pub type FormatInfo = CameraFormat;

#[derive(Debug, Clone)]
pub enum BufferPolicy {
    DropOldest { capacity: usize },
}

#[derive(Debug, Clone)]
pub enum AudioMode {
    Disabled,
    Enabled,
}

#[derive(Debug, Clone)]
pub struct CaptureConfig {
    pub device_id: String,
    pub format: CameraFormat,
    pub buffer_policy: BufferPolicy,
    pub audio_mode: AudioMode,
    pub audio_device_id: Option<String>,
}

impl CaptureConfig {
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct Frame {
    pub sequence: u64,
    pub timestamp_us: u64,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub device_id: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct AudioPacket {
    pub sequence: u64,
    pub timestamp_us: u64,
    pub sample_rate: u32,
    pub channels: u16,
    pub format: String,
    pub data: Vec<u8>,
}
