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
    pub metadata: FrameMetadata,
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
            metadata: FrameMetadata::default(),
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

/// Advanced camera controls for professional photography
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CameraControls {
    pub auto_focus: Option<bool>,
    pub focus_distance: Option<f32>,          // 0.0 = infinity, 1.0 = closest
    pub auto_exposure: Option<bool>,
    pub exposure_time: Option<f32>,           // Seconds
    pub iso_sensitivity: Option<u32>,         // ISO value
    pub white_balance: Option<WhiteBalance>,
    pub aperture: Option<f32>,                // f-stop value
    pub zoom: Option<f32>,                    // Digital zoom factor
    pub brightness: Option<f32>,              // -1.0 to 1.0
    pub contrast: Option<f32>,                // -1.0 to 1.0
    pub saturation: Option<f32>,              // -1.0 to 1.0
    pub sharpness: Option<f32>,               // -1.0 to 1.0
    pub noise_reduction: Option<bool>,
    pub image_stabilization: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WhiteBalance {
    Auto,
    Daylight,
    Fluorescent,
    Incandescent,
    Flash,
    Cloudy,
    Shade,
    Custom(u32), // Color temperature in Kelvin
}

impl Default for CameraControls {
    fn default() -> Self {
        Self {
            auto_focus: Some(true),
            focus_distance: None,
            auto_exposure: Some(true),
            exposure_time: None,
            iso_sensitivity: Some(400),
            white_balance: Some(WhiteBalance::Auto),
            aperture: None,
            zoom: Some(1.0),
            brightness: Some(0.0),
            contrast: Some(0.0),
            saturation: Some(0.0),
            sharpness: Some(0.0),
            noise_reduction: Some(true),
            image_stabilization: Some(true),
        }
    }
}

impl CameraControls {
    pub fn professional() -> Self {
        Self {
            auto_focus: Some(false),
            focus_distance: Some(0.5),
            auto_exposure: Some(false),
            exposure_time: Some(1.0/60.0),
            iso_sensitivity: Some(100),
            white_balance: Some(WhiteBalance::Daylight),
            aperture: Some(8.0),
            zoom: Some(1.0),
            brightness: Some(0.0),
            contrast: Some(0.3),
            saturation: Some(0.4),
            sharpness: Some(0.5),
            noise_reduction: Some(true),
            image_stabilization: Some(true),
        }
    }
}

/// Burst capture configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurstConfig {
    pub count: u32,                    // Number of photos
    pub interval_ms: u32,              // Time between shots
    pub bracketing: Option<ExposureBracketing>,
    pub focus_stacking: bool,          // Vary focus for each shot
    pub auto_save: bool,               // Automatically save all frames
    pub save_directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureBracketing {
    pub stops: Vec<f32>,              // EV adjustments: [-2.0, 0.0, +2.0]
    pub base_exposure: f32,           // Base exposure time in seconds
}

impl BurstConfig {
    pub fn hdr_burst() -> Self {
        Self {
            count: 3,
            interval_ms: 200,
            bracketing: Some(ExposureBracketing {
                stops: vec![-1.0, 0.0, 1.0],
                base_exposure: 1.0/125.0,
            }),
            focus_stacking: false,
            auto_save: true,
            save_directory: Some("hdr_captures".to_string()),
        }
    }
}

/// Camera hardware capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraCapabilities {
    pub supports_auto_focus: bool,
    pub supports_manual_focus: bool,
    pub supports_auto_exposure: bool,
    pub supports_manual_exposure: bool,
    pub supports_white_balance: bool,
    pub supports_zoom: bool,
    pub supports_flash: bool,
    pub supports_burst_mode: bool,
    pub supports_hdr: bool,
    pub max_resolution: (u32, u32),
    pub max_fps: f32,
    pub exposure_range: Option<(f32, f32)>, // min, max exposure time
    pub iso_range: Option<(u32, u32)>,      // min, max ISO
    pub focus_range: Option<(f32, f32)>,    // min, max focus distance
}

impl Default for CameraCapabilities {
    fn default() -> Self {
        Self {
            supports_auto_focus: true,
            supports_manual_focus: false,
            supports_auto_exposure: true,
            supports_manual_exposure: false,
            supports_white_balance: true,
            supports_zoom: false,
            supports_flash: false,
            supports_burst_mode: true,
            supports_hdr: false,
            max_resolution: (1920, 1080),
            max_fps: 30.0,
            exposure_range: None,
            iso_range: None,
            focus_range: None,
        }
    }
}

/// Extended metadata for camera frames
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrameMetadata {
    pub exposure_time: Option<f32>,
    pub iso_sensitivity: Option<u32>,
    pub white_balance: Option<WhiteBalance>,
    pub focus_distance: Option<f32>,
    pub aperture: Option<f32>,
    pub flash_fired: Option<bool>,
    pub scene_mode: Option<String>,
    pub capture_settings: Option<CameraControls>,
}


/// Performance metrics for camera operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraPerformanceMetrics {
    pub capture_latency_ms: f32,
    pub processing_time_ms: f32,
    pub memory_usage_mb: f32,
    pub fps_actual: f32,
    pub dropped_frames: u32,
    pub buffer_overruns: u32,
    pub quality_score: f32,
}

impl Default for CameraPerformanceMetrics {
    fn default() -> Self {
        Self {
            capture_latency_ms: 0.0,
            processing_time_ms: 0.0,
            memory_usage_mb: 0.0,
            fps_actual: 0.0,
            dropped_frames: 0,
            buffer_overruns: 0,
            quality_score: 0.0,
        }
    }
}

/// Camera initialization parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraInitParams {
    pub device_id: String,
    pub format: CameraFormat,
    pub controls: CameraControls,
}

impl CameraInitParams {
    /// Create new initialization parameters
    pub fn new(device_id: String) -> Self {
        Self {
            device_id,
            format: CameraFormat::standard(),
            controls: CameraControls::default(),
        }
    }

    /// Set desired format
    pub fn with_format(mut self, format: CameraFormat) -> Self {
        self.format = format;
        self
    }

    /// Set camera controls
    pub fn with_controls(mut self, controls: CameraControls) -> Self {
        self.controls = controls;
        self
    }

    /// Enable/disable auto focus
    pub fn with_auto_focus(mut self, enabled: bool) -> Self {
        self.controls.auto_focus = Some(enabled);
        self
    }

    /// Enable/disable auto exposure  
    pub fn with_auto_exposure(mut self, enabled: bool) -> Self {
        self.controls.auto_exposure = Some(enabled);
        self
    }
    
    /// Create parameters optimized for professional photography
    pub fn professional(device_id: String) -> Self {
        Self {
            device_id,
            format: CameraFormat::new(2592, 1944, 15.0), // 5MP high quality
            controls: CameraControls::professional(),
        }
    }
}