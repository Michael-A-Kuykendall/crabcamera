use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::constants::*;

/// Platform enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    /// Windows OS.
    Windows,
    /// Apple macOS.
    MacOS,
    /// Linux OS.
    Linux,
    /// Unknown or unsupported platform.
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
    /// Unique identifier for the camera device.
    pub id: String,
    /// Human-readable name of the camera.
    pub name: String,
    /// Optional description of the camera.
    pub description: Option<String>,
    /// Whether the camera is currently available for use.
    pub is_available: bool,
    /// List of supported capture formats.
    pub supports_formats: Vec<CameraFormat>,
    /// The platform this camera belongs to.
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
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Frames per second.
    pub fps: f32,
    /// Format identifier (e.g. "MJPEG").
    pub format_type: String,
}

impl CameraFormat {
    /// Create new camera format
    pub fn new(width: u32, height: u32, fps: f32) -> Self {
        Self {
            width,
            height,
            fps,
            format_type: FORMAT_RGB.to_string(),
        }
    }

    /// Create high resolution format
    pub fn hd() -> Self {
        Self::new(DEFAULT_RESOLUTION_WIDTH, DEFAULT_RESOLUTION_HEIGHT, DEFAULT_FPS)
    }

    /// Create standard resolution format
    pub fn standard() -> Self {
        Self::new(FALLBACK_RESOLUTION_WIDTH, FALLBACK_RESOLUTION_HEIGHT, DEFAULT_FPS)
    }

    /// Create low resolution format
    pub fn low() -> Self {
        Self::new(MIN_RESOLUTION_WIDTH, MIN_RESOLUTION_HEIGHT, DEFAULT_FPS)
    }

    /// Set format type
    pub fn with_format_type(mut self, format_type: String) -> Self {
        self.format_type = format_type;
        self
    }
}

impl Default for CameraFormat {
    fn default() -> Self {
        Self::standard()
    }
}

/// Camera frame data with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraFrame {
    /// Unique identifier for the frame (UUID).
    pub id: String,
    /// Raw pixel data.
    pub data: Vec<u8>,
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
    /// Format identifier.
    pub format: String,
    /// Capture timestamp.
    pub timestamp: DateTime<Utc>,
    /// ID of the source device.
    pub device_id: String,
    /// Size of the data buffer in bytes.
    pub size_bytes: usize,
    /// Additional frame metadata.
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
            format: FORMAT_RGB.to_string(),
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
    /// Enable auto-focus.
    pub auto_focus: Option<bool>,
    /// Focus distance (0.0 = infinity, 1.0 = closest).
    pub focus_distance: Option<f32>,
    /// Enable auto-exposure.
    pub auto_exposure: Option<bool>,
    /// Exposure time in seconds.
    pub exposure_time: Option<f32>,
    /// ISO sensitivity value.
    pub iso_sensitivity: Option<u32>,
    /// White balance setting.
    pub white_balance: Option<WhiteBalance>,
    /// Aperture f-stop value.
    pub aperture: Option<f32>,
    /// Digital zoom factor.
    pub zoom: Option<f32>,
    /// Brightness adjustment (-1.0 to 1.0).
    pub brightness: Option<f32>,
    /// Contrast adjustment (-1.0 to 1.0).
    pub contrast: Option<f32>,
    /// Saturation adjustment (-1.0 to 1.0).
    pub saturation: Option<f32>,
    /// Sharpness adjustment (-1.0 to 1.0).
    pub sharpness: Option<f32>,
    /// Enable noise reduction.
    pub noise_reduction: Option<bool>,
    /// Enable image stabilization.
    pub image_stabilization: Option<bool>,
}

/// White balance presets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WhiteBalance {
    /// Automatic white balance.
    Auto,
    /// Daylight preset (approx 5000K-6500K).
    Daylight,
    /// Fluorescent light preset (approx 4000K-5000K).
    Fluorescent,
    /// Incandescent/Tungsten light preset (approx 2500K-3000K).
    Incandescent,
    /// Flash preset.
    Flash,
    /// Cloudy sky preset (approx 6500K-8000K).
    Cloudy,
    /// Shade preset (approx 8000K+).
    Shade,
    /// Custom color temperature in Kelvin (e.g. 5000).
    Custom(u32),
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
    /// Create a preset for professional photography.
    pub fn professional() -> Self {
        Self {
            auto_focus: Some(false),
            focus_distance: Some(0.5),
            auto_exposure: Some(false),
            exposure_time: Some(1.0 / 60.0),
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
    /// Number of photos to capture.
    pub count: u32,
    /// Time interval between shots in milliseconds.
    pub interval_ms: u32,
    /// Optional exposure bracketing configuration.
    pub bracketing: Option<ExposureBracketing>,
    /// Whether to vary focus distance for each shot (focus stacking).
    pub focus_stacking: bool,
    /// Whether to automatically save all frames to disk.
    pub auto_save: bool,
    /// Directory to save frames if auto_save is enabled.
    pub save_directory: Option<String>,
}

/// Exposure bracketing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureBracketing {
    /// List of exposure compensation values in stops (e.g. `[-2.0, 0.0, 2.0]`).
    pub stops: Vec<f32>,
    /// Base exposure time in seconds.
    pub base_exposure: f32,
}

impl BurstConfig {
    /// Create a standard HDR burst configuration.
    ///
    /// Captures 3 frames at -1.0, 0.0, and +1.0 EV.
    pub fn hdr_burst() -> Self {
        Self {
            count: 3,
            interval_ms: 200,
            bracketing: Some(ExposureBracketing {
                stops: vec![-1.0, 0.0, 1.0],
                base_exposure: 1.0 / 125.0,
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
    /// Supports auto-focus.
    pub supports_auto_focus: bool,
    /// Supports manual focus.
    pub supports_manual_focus: bool,
    /// Supports auto-exposure.
    pub supports_auto_exposure: bool,
    /// Supports manual exposure.
    pub supports_manual_exposure: bool,
    /// Supports white balance adjustment.
    pub supports_white_balance: bool,
    /// Supports zoom (optical or digital).
    pub supports_zoom: bool,
    /// Supports flash.
    pub supports_flash: bool,
    /// Supports burst mode capture.
    pub supports_burst_mode: bool,
    /// Supports HDR mode.
    pub supports_hdr: bool,
    /// Maximum supported resolution (width, height).
    pub max_resolution: (u32, u32),
    /// Maximum supported framerate.
    pub max_fps: f32,
    /// Range of supported exposure times (min, max) in seconds.
    pub exposure_range: Option<(f32, f32)>,
    /// Range of supported ISO values (min, max).
    pub iso_range: Option<(u32, u32)>,
    /// Range of supported focus distances (min, max).
    pub focus_range: Option<(f32, f32)>,
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
    /// Exposure time in seconds.
    pub exposure_time: Option<f32>,
    /// ISO sensitivity.
    pub iso_sensitivity: Option<u32>,
    /// White balance setting.
    pub white_balance: Option<WhiteBalance>,
    /// Focus distance (0.0-1.0).
    pub focus_distance: Option<f32>,
    /// Aperture f-stop.
    pub aperture: Option<f32>,
    /// Whether flash fired.
    pub flash_fired: Option<bool>,
    /// Scene mode description.
    pub scene_mode: Option<String>,
    /// Full capture settings snapshot.
    pub capture_settings: Option<CameraControls>,
}

/// Performance metrics for camera operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraPerformanceMetrics {
    /// Capture latency in milliseconds.
    pub capture_latency_ms: f32,
    /// Processing time in milliseconds.
    pub processing_time_ms: f32,
    /// Memory usage in megabytes.
    pub memory_usage_mb: f32,
    /// Actual frames per second delivered.
    pub fps_actual: f32,
    /// Number of dropped frames.
    pub dropped_frames: u32,
    /// Number of buffer overruns.
    pub buffer_overruns: u32,
    /// Overall quality score (0.0-1.0).
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
    /// Device identifier.
    pub device_id: String,
    /// Desired camera format.
    pub format: CameraFormat,
    /// Initial camera controls.
    pub controls: CameraControls,
}

impl Default for CameraInitParams {
    fn default() -> Self {
        Self::new("0".to_string())
    }
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
    #[must_use]
    pub fn with_format(mut self, format: CameraFormat) -> Self {
        self.format = format;
        self
    }

    /// Set camera controls
    #[must_use]
    pub fn with_controls(mut self, controls: CameraControls) -> Self {
        self.controls = controls;
        self
    }

    /// Enable/disable auto focus
    #[must_use]
    pub fn with_auto_focus(mut self, enabled: bool) -> Self {
        self.controls.auto_focus = Some(enabled);
        self
    }

    /// Enable/disable auto exposure  
    #[must_use]
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
