use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Platform enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
    Unknown,
}

impl Platform {
    /// Detect current platform
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOS
        } else if cfg!(target_os = "linux") {
            Platform::Linux
        } else {
            Platform::Unknown
        }
    }

    /// Get platform as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::Windows => "windows",
            Platform::MacOS => "macos", 
            Platform::Linux => "linux",
            Platform::Unknown => "unknown",
        }
    }
}

/// Camera device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraDeviceInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_available: bool,
    pub supports_formats: Vec<CameraFormat>,
    pub platform: Platform,
}

impl CameraDeviceInfo {
    /// Create new camera device info
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            description: None,
            is_available: true,
            supports_formats: Vec::new(),
            platform: Platform::current(),
        }
    }

    /// Set description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set supported formats
    pub fn with_formats(mut self, formats: Vec<CameraFormat>) -> Self {
        self.supports_formats = formats;
        self
    }

    /// Set availability
    pub fn with_availability(mut self, available: bool) -> Self {
        self.is_available = available;
        self
    }
}

/// Camera format specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CameraFormat {
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    pub format_type: String,
}

impl CameraFormat {
    /// Create new camera format
    pub fn new(width: u32, height: u32, fps: f32) -> Self {
        Self {
            width,
            height,
            fps,
            format_type: "RGB8".to_string(),
        }
    }

    /// Create high resolution format
    pub fn hd() -> Self {
        Self::new(1920, 1080, 30.0)
    }

    /// Create standard resolution format
    pub fn standard() -> Self {
        Self::new(1280, 720, 30.0)
    }

    /// Create low resolution format
    pub fn low() -> Self {
        Self::new(640, 480, 30.0)
    }

    /// Set format type
    pub fn with_format_type(mut self, format_type: String) -> Self {
        self.format_type = format_type;
        self
    }
}

/// Camera frame data with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraFrame {
    pub id: String,
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub timestamp: DateTime<Utc>,
    pub device_id: String,
    pub size_bytes: usize,
}

impl CameraFrame {
    /// Create new camera frame
    pub fn new(data: Vec<u8>, width: u32, height: u32, device_id: String) -> Self {
        let size_bytes = data.len();
        Self {
            id: Uuid::new_v4().to_string(),
            data,
            width,
            height,
            format: "RGB8".to_string(),
            timestamp: Utc::now(),
            device_id,
            size_bytes,
        }
    }

    /// Set format
    pub fn with_format(mut self, format: String) -> Self {
        self.format = format;
        self
    }

    /// Get frame aspect ratio
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    /// Check if frame is valid
    pub fn is_valid(&self) -> bool {
        !self.data.is_empty() && self.width > 0 && self.height > 0
    }
}

/// Camera initialization parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraInitParams {
    pub device_id: String,
    pub format: CameraFormat,
    pub auto_focus: bool,
    pub auto_exposure: bool,
}

impl CameraInitParams {
    /// Create new initialization parameters
    pub fn new(device_id: String) -> Self {
        Self {
            device_id,
            format: CameraFormat::standard(),
            auto_focus: true,
            auto_exposure: true,
        }
    }

    /// Set desired format
    pub fn with_format(mut self, format: CameraFormat) -> Self {
        self.format = format;
        self
    }

    /// Enable/disable auto focus
    pub fn with_auto_focus(mut self, enabled: bool) -> Self {
        self.auto_focus = enabled;
        self
    }

    /// Enable/disable auto exposure  
    pub fn with_auto_exposure(mut self, enabled: bool) -> Self {
        self.auto_exposure = enabled;
        self
    }
}