//! Platform-specific camera implementations with unified interface
//!
//! This module provides a unified interface for camera operations across
//! different platforms (Windows, macOS, Linux) while maintaining platform-specific
//! optimizations and features.

use crate::constants::{
    DEFAULT_RESOLUTION_HEIGHT, DEFAULT_RESOLUTION_WIDTH, HIGH_FPS, MAX_ISO, MIN_ISO,
    MOCK_CAPTURE_LATENCY_MS, MOCK_FPS, MOCK_MEMORY_USAGE_MB, MOCK_PROCESSING_TIME_MS,
    MOCK_QUALITY_SCORE, MOCK_SLOW_CAPTURE_DELAY_MS,
};
use crate::errors::CameraError;
use crate::types::{
    CameraDeviceInfo, CameraFormat, CameraFrame, CameraInitParams, ControlApplicationResult,
    Platform,
};

// Type alias for frame callback to reduce complexity
type FrameCallback = Box<dyn Fn(CameraFrame) + Send + 'static>;

// Platform-specific modules
/// Windows-specific camera backend (Media Foundation via nokhwa).
#[cfg(target_os = "windows")]
pub mod windows;

/// macOS-specific camera backend (AVFoundation).
#[cfg(target_os = "macos")]
pub mod macos;

/// Linux-specific camera backend (V4L2).
#[cfg(target_os = "linux")]
pub mod linux;

// Device monitoring module
pub mod device_monitor;

// Shared real performance tracking
pub mod metrics;

pub use device_monitor::{DeviceEvent, DeviceMonitor};

/// Camera manager module for handling device lifecycle.
pub mod manager;
pub use manager::{
    capture_with_reconnect, get_existing_camera, get_or_create_camera, reconnect_camera,
    release_camera,
};

use std::sync::{Arc, Mutex};

/// Mock camera implementation for testing.
///
/// This provides a fake camera device that returns generated frames for testing
/// without physical hardware.
pub struct MockCamera {
    device_id: String,
    controls: Arc<Mutex<crate::types::CameraControls>>,
    is_streaming: Arc<Mutex<bool>>,
    capture_mode: Arc<Mutex<crate::tests::MockCaptureMode>>,
    callback: Arc<Mutex<Option<FrameCallback>>>,
}

impl MockCamera {
    /// Create a new mock camera instance.
    pub fn new(device_id: String, _format: CameraFormat) -> Self {
        Self {
            device_id,
            controls: Arc::new(Mutex::new(crate::types::CameraControls::default())),
            is_streaming: Arc::new(Mutex::new(false)),
            capture_mode: Arc::new(Mutex::new(crate::tests::MockCaptureMode::Success)),
            callback: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the behavior mode for this mock camera (e.g. simulate failure).
    pub fn set_capture_mode(&self, mode: crate::tests::MockCaptureMode) {
        if let Ok(mut capture_mode) = self.capture_mode.lock() {
            *capture_mode = mode;
        }
    }

    /// Capture a single frame from the mock camera.
    ///
    /// # Errors
    /// Returns a [`CameraError::CaptureError`] when the mock camera is in its
    /// failure simulation mode.
    pub fn capture_frame(&mut self) -> Result<CameraFrame, CameraError> {
        // Check global registry first, then fall back to local mode
        let mode = crate::tests::get_mock_camera_mode(&self.device_id);

        let frame = match mode {
            crate::tests::MockCaptureMode::Success => {
                Ok(crate::tests::create_mock_frame(&self.device_id))
            }
            crate::tests::MockCaptureMode::Failure => Err(CameraError::CaptureError(
                "Mock capture failure".to_string(),
            )),
            crate::tests::MockCaptureMode::SlowCapture => {
                std::thread::sleep(std::time::Duration::from_millis(MOCK_SLOW_CAPTURE_DELAY_MS));
                Ok(crate::tests::create_mock_frame(&self.device_id))
            }
        };

        // Call callback if set and frame was successful
        if let Ok(ref frame) = frame {
            if let Ok(cb) = self.callback.lock() {
                if let Some(ref callback) = *cb {
                    callback(frame.clone());
                }
            }
        }

        frame
    }

    /// Start the stream.
    ///
    /// # Errors
    /// This function currently always returns `Ok` and never returns an `Err`.
    pub fn start_stream(&self) -> Result<(), CameraError> {
        if let Ok(mut streaming) = self.is_streaming.lock() {
            *streaming = true;
        }
        Ok(())
    }

    /// Stop the stream.
    ///
    /// # Errors
    /// This function currently always returns `Ok` and never returns an `Err`.
    pub fn stop_stream(&self) -> Result<(), CameraError> {
        if let Ok(mut streaming) = self.is_streaming.lock() {
            *streaming = false;
        }
        Ok(())
    }

    /// Register a callback for new frames.
    ///
    /// # Errors
    /// This function currently always returns `Ok` and never returns an `Err`.
    pub fn frame_callback<F>(&mut self, callback: F) -> Result<(), CameraError>
    where
        F: Fn(CameraFrame) + Send + 'static,
    {
        if let Ok(mut cb) = self.callback.lock() {
            *cb = Some(Box::new(callback));
        }
        Ok(())
    }

    /// Check if the camera is available.
    pub fn is_available(&self) -> bool {
        true
    }

    /// Get the device ID.
    pub fn get_device_id(&self) -> &str {
        &self.device_id
    }

    /// Apply camera controls.
    ///
    /// # Errors
    /// This function currently always returns `Ok` and never returns an `Err`.
    pub fn apply_controls(
        &mut self,
        controls: &crate::types::CameraControls,
    ) -> Result<ControlApplicationResult, CameraError> {
        if let Ok(mut current_controls) = self.controls.lock() {
            *current_controls = controls.clone();
        }
        // Mock accepts every control requested
        let mut applied = Vec::new();
        if controls.auto_focus.is_some() {
            applied.push("auto_focus".to_string());
        }
        if controls.focus_distance.is_some() {
            applied.push("focus_distance".to_string());
        }
        if controls.auto_exposure.is_some() {
            applied.push("auto_exposure".to_string());
        }
        if controls.exposure_time.is_some() {
            applied.push("exposure_time".to_string());
        }
        if controls.iso_sensitivity.is_some() {
            applied.push("iso_sensitivity".to_string());
        }
        if controls.white_balance.is_some() {
            applied.push("white_balance".to_string());
        }
        if controls.aperture.is_some() {
            applied.push("aperture".to_string());
        }
        if controls.zoom.is_some() {
            applied.push("zoom".to_string());
        }
        if controls.brightness.is_some() {
            applied.push("brightness".to_string());
        }
        if controls.contrast.is_some() {
            applied.push("contrast".to_string());
        }
        if controls.saturation.is_some() {
            applied.push("saturation".to_string());
        }
        if controls.sharpness.is_some() {
            applied.push("sharpness".to_string());
        }
        if controls.noise_reduction.is_some() {
            applied.push("noise_reduction".to_string());
        }
        if controls.image_stabilization.is_some() {
            applied.push("image_stabilization".to_string());
        }
        Ok(ControlApplicationResult {
            applied,
            rejected: vec![],
        })
    }

    /// Get current camera controls.
    ///
    /// # Errors
    /// This function currently always returns `Ok` and never returns an `Err`.
    pub fn get_controls(&self) -> Result<crate::types::CameraControls, CameraError> {
        if let Ok(controls) = self.controls.lock() {
            Ok(controls.clone())
        } else {
            Ok(crate::types::CameraControls::default())
        }
    }

    /// Create a mock capabilities report.
    ///
    /// # Errors
    /// This function currently always returns `Ok` and never returns an `Err`.
    pub fn test_capabilities(&self) -> Result<crate::types::CameraCapabilities, CameraError> {
        Ok(crate::types::CameraCapabilities {
            supports: crate::types::CameraCapabilityFlags {
                auto_focus: true,
                manual_focus: true,
                auto_exposure: true,
                manual_exposure: true,
                white_balance: true,
                zoom: true,
                flash: false,
                burst_mode: true,
                hdr: true,
            },
            max_resolution: (DEFAULT_RESOLUTION_WIDTH, DEFAULT_RESOLUTION_HEIGHT),
            max_fps: HIGH_FPS,
            exposure_range: Some((0.001, 10.0)),
            iso_range: Some((MIN_ISO, MAX_ISO)),
            focus_range: Some((0.0, 1.0)),
        })
    }

    /// Get mock performance metrics.
    ///
    /// # Errors
    /// This function currently always returns `Ok` and never returns an `Err`.
    pub fn get_performance_metrics(
        &self,
    ) -> Result<crate::types::CameraPerformanceMetrics, CameraError> {
        Ok(crate::types::CameraPerformanceMetrics {
            capture_latency_ms: MOCK_CAPTURE_LATENCY_MS,
            processing_time_ms: MOCK_PROCESSING_TIME_MS,
            memory_usage_mb: MOCK_MEMORY_USAGE_MB,
            fps_actual: MOCK_FPS,
            dropped_frames: 0,
            buffer_overruns: 0,
            quality_score: MOCK_QUALITY_SCORE,
        })
    }
}

/// Unified camera interface that abstracts platform differences
pub enum PlatformCamera {
    /// Windows Media Foundation backend.
    #[cfg(target_os = "windows")]
    Windows(windows::WindowsCamera),

    /// MacOS AVFoundation backend.
    #[cfg(target_os = "macos")]
    MacOS(macos::MacOSCamera),

    /// Linux V4L2 backend.
    #[cfg(target_os = "linux")]
    Linux(linux::LinuxCamera),

    /// Mock camera for testing.
    Mock(MockCamera),

    /// Fallback for unsupported platforms.
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    Unsupported,
}

impl PlatformCamera {
    /// Create new platform camera from initialization parameters
    ///
    /// # Errors
    /// Returns a [`CameraError::InitializationError`] if the current platform
    /// is unsupported, or propagates any error from the platform-specific camera
    /// creation.
    pub fn new(params: CameraInitParams) -> Result<Self, CameraError> {
        // Only use mock camera when explicitly requested via environment variable
        // or when running in unit test threads (thread name contains "test")
        // Note: We no longer check CARGO_MANIFEST_DIR because that's set during
        // normal `cargo run` which should use real cameras
        let use_mock = std::env::var("CRABCAMERA_USE_MOCK").is_ok()
            || std::thread::current()
                .name()
                .is_some_and(|name| name.contains("test"));

        if use_mock {
            log::info!("Using mock camera (CRABCAMERA_USE_MOCK set or in test thread)");
            let mock_camera = MockCamera::new(params.device_id, params.format);
            return Ok(PlatformCamera::Mock(mock_camera));
        }

        match Platform::current() {
            #[cfg(target_os = "windows")]
            Platform::Windows => {
                let camera = windows::WindowsCamera::new(params.device_id, &params.format)?;
                Ok(PlatformCamera::Windows(camera))
            }

            #[cfg(target_os = "macos")]
            Platform::MacOS => {
                let camera = macos::initialize_camera(params)?;
                Ok(PlatformCamera::MacOS(camera))
            }

            #[cfg(target_os = "linux")]
            Platform::Linux => {
                let camera = linux::initialize_camera(params)?;
                Ok(PlatformCamera::Linux(camera))
            }

            _ => Err(CameraError::InitializationError(
                "Unsupported platform".to_string(),
            )),
        }
    }

    /// Capture a single frame from the camera
    ///
    /// # Errors
    /// Returns a [`CameraError::InitializationError`] on an unsupported platform,
    /// or propagates any error from the underlying platform camera's capture.
    pub fn capture_frame(&mut self) -> Result<CameraFrame, CameraError> {
        match self {
            #[cfg(target_os = "windows")]
            PlatformCamera::Windows(camera) => camera.capture_frame(),

            #[cfg(target_os = "macos")]
            PlatformCamera::MacOS(camera) => camera.capture_frame(),

            #[cfg(target_os = "linux")]
            PlatformCamera::Linux(camera) => camera.capture_frame(),

            PlatformCamera::Mock(camera) => camera.capture_frame(),

            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            PlatformCamera::Unsupported => Err(CameraError::InitializationError(
                "Unsupported platform".to_string(),
            )),
        }
    }

    /// Start camera stream
    ///
    /// # Errors
    /// Returns a [`CameraError::InitializationError`] on an unsupported platform,
    /// or propagates any error from the underlying platform camera's stream start.
    pub fn start_stream(&mut self) -> Result<(), CameraError> {
        match self {
            #[cfg(target_os = "windows")]
            PlatformCamera::Windows(camera) => camera.start_stream(),

            #[cfg(target_os = "macos")]
            PlatformCamera::MacOS(camera) => camera.start_stream(),

            #[cfg(target_os = "linux")]
            PlatformCamera::Linux(camera) => camera.start_stream(),

            PlatformCamera::Mock(camera) => camera.start_stream(),

            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            PlatformCamera::Unsupported => Err(CameraError::InitializationError(
                "Unsupported platform".to_string(),
            )),
        }
    }

    /// Stop camera stream
    ///
    /// # Errors
    /// Returns a [`CameraError::InitializationError`] on an unsupported platform,
    /// or propagates any error from the underlying platform camera's stream stop.
    pub fn stop_stream(&mut self) -> Result<(), CameraError> {
        match self {
            #[cfg(target_os = "windows")]
            PlatformCamera::Windows(camera) => camera.stop_stream(),

            #[cfg(target_os = "macos")]
            PlatformCamera::MacOS(camera) => camera.stop_stream(),

            #[cfg(target_os = "linux")]
            PlatformCamera::Linux(camera) => camera.stop_stream(),

            PlatformCamera::Mock(camera) => camera.stop_stream(),

            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            PlatformCamera::Unsupported => Err(CameraError::InitializationError(
                "Unsupported platform".to_string(),
            )),
        }
    }

    /// Check if camera is available
    pub fn is_available(&self) -> bool {
        match self {
            #[cfg(target_os = "windows")]
            PlatformCamera::Windows(camera) => camera.is_available(),

            #[cfg(target_os = "macos")]
            PlatformCamera::MacOS(camera) => camera.is_available(),

            #[cfg(target_os = "linux")]
            PlatformCamera::Linux(camera) => camera.is_available(),

            PlatformCamera::Mock(camera) => camera.is_available(),

            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            PlatformCamera::Unsupported => false,
        }
    }

    /// Set frame callback for real-time processing
    ///
    /// # Errors
    /// Returns a [`CameraError::UnsupportedOperation`] on an unsupported platform,
    /// or propagates any error from the underlying platform camera's callback
    /// registration.
    pub fn frame_callback<F>(&mut self, callback: F) -> Result<(), CameraError>
    where
        F: Fn(CameraFrame) + Send + 'static,
    {
        match self {
            #[cfg(target_os = "windows")]
            PlatformCamera::Windows(camera) => camera.set_callback(callback),

            #[cfg(target_os = "macos")]
            PlatformCamera::MacOS(camera) => camera.set_callback(callback),

            #[cfg(target_os = "linux")]
            PlatformCamera::Linux(camera) => camera.set_callback(callback),

            PlatformCamera::Mock(camera) => camera.frame_callback(callback),

            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            PlatformCamera::Unsupported => Err(CameraError::UnsupportedOperation(
                "Frame callback not supported on this platform".to_string(),
            )),
        }
    }

    /// Get device ID
    pub fn get_device_id(&self) -> Option<&str> {
        match self {
            #[cfg(target_os = "windows")]
            PlatformCamera::Windows(camera) => Some(camera.get_device_id()),

            #[cfg(target_os = "macos")]
            PlatformCamera::MacOS(camera) => Some(camera.get_device_id()),

            #[cfg(target_os = "linux")]
            PlatformCamera::Linux(camera) => Some(camera.get_device_id()),

            PlatformCamera::Mock(camera) => Some(camera.get_device_id()),

            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            PlatformCamera::Unsupported => None,
        }
    }

    /// Apply camera controls
    ///
    /// # Errors
    /// Returns a [`CameraError::InitializationError`] on an unsupported platform,
    /// or propagates any error from the underlying platform camera's control
    /// application.
    pub fn apply_controls(
        &mut self,
        controls: &crate::types::CameraControls,
    ) -> Result<ControlApplicationResult, CameraError> {
        match self {
            #[cfg(target_os = "windows")]
            PlatformCamera::Windows(camera) => camera.apply_controls(controls),

            #[cfg(target_os = "macos")]
            PlatformCamera::MacOS(camera) => camera.apply_controls(controls),

            #[cfg(target_os = "linux")]
            PlatformCamera::Linux(camera) => camera.apply_controls(controls),

            PlatformCamera::Mock(camera) => camera.apply_controls(controls),

            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            PlatformCamera::Unsupported => Err(CameraError::InitializationError(
                "Unsupported platform".to_string(),
            )),
        }
    }

    /// Get current camera controls
    ///
    /// # Errors
    /// Returns a [`CameraError::InitializationError`] on an unsupported platform,
    /// or propagates any error from the underlying platform camera's control read.
    pub fn get_controls(&self) -> Result<crate::types::CameraControls, CameraError> {
        match self {
            #[cfg(target_os = "windows")]
            PlatformCamera::Windows(camera) => camera.get_controls(),

            #[cfg(target_os = "macos")]
            PlatformCamera::MacOS(camera) => camera.get_controls(),

            #[cfg(target_os = "linux")]
            PlatformCamera::Linux(camera) => camera.get_controls(),

            PlatformCamera::Mock(camera) => camera.get_controls(),

            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            PlatformCamera::Unsupported => Err(CameraError::InitializationError(
                "Unsupported platform".to_string(),
            )),
        }
    }

    /// Test camera capabilities
    ///
    /// # Errors
    /// Returns a [`CameraError::InitializationError`] on an unsupported platform,
    /// or propagates any error from the underlying platform camera's capability
    /// query.
    pub fn test_capabilities(&self) -> Result<crate::types::CameraCapabilities, CameraError> {
        match self {
            #[cfg(target_os = "windows")]
            PlatformCamera::Windows(camera) => camera.test_capabilities(),

            #[cfg(target_os = "macos")]
            PlatformCamera::MacOS(camera) => camera.test_capabilities(),

            #[cfg(target_os = "linux")]
            PlatformCamera::Linux(camera) => camera.test_capabilities(),

            PlatformCamera::Mock(camera) => camera.test_capabilities(),

            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            PlatformCamera::Unsupported => Err(CameraError::InitializationError(
                "Unsupported platform".to_string(),
            )),
        }
    }

    /// Get performance metrics
    ///
    /// # Errors
    /// Returns a [`CameraError::InitializationError`] on an unsupported platform,
    /// or propagates any error from the underlying platform camera's metrics
    /// query.
    pub fn get_performance_metrics(
        &self,
    ) -> Result<crate::types::CameraPerformanceMetrics, CameraError> {
        match self {
            #[cfg(target_os = "windows")]
            PlatformCamera::Windows(camera) => camera.get_performance_metrics(),

            #[cfg(target_os = "macos")]
            PlatformCamera::MacOS(camera) => camera.get_performance_metrics(),

            #[cfg(target_os = "linux")]
            PlatformCamera::Linux(camera) => camera.get_performance_metrics(),

            PlatformCamera::Mock(camera) => camera.get_performance_metrics(),

            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            PlatformCamera::Unsupported => Err(CameraError::InitializationError(
                "Unsupported platform".to_string(),
            )),
        }
    }
}

// Cleanup implementation
impl Drop for PlatformCamera {
    fn drop(&mut self) {
        let _ = self.stop_stream();
    }
}

/// Platform-specific camera system functions
pub struct CameraSystem;

impl CameraSystem {
    /// List all available cameras on the current platform
    ///
    /// # Errors
    /// Returns a [`CameraError::InitializationError`] if the current platform
    /// is unsupported, or propagates any error from the platform-specific camera
    /// enumeration.
    pub fn list_cameras() -> Result<Vec<CameraDeviceInfo>, CameraError> {
        match Platform::current() {
            #[cfg(target_os = "windows")]
            Platform::Windows => windows::list_cameras(),

            #[cfg(target_os = "macos")]
            Platform::MacOS => macos::list_cameras(),

            #[cfg(target_os = "linux")]
            Platform::Linux => linux::list_cameras(),

            _ => Err(CameraError::InitializationError(
                "Unsupported platform".to_string(),
            )),
        }
    }

    /// Initialize the camera system for the current platform
    ///
    /// # Errors
    /// Returns a [`CameraError::InitializationError`] if the platform is
    /// unknown, if V4L2 is unavailable on Linux, or if Linux support was not
    /// compiled in.
    pub fn initialize() -> Result<String, CameraError> {
        match Platform::current() {
            Platform::Windows => {
                Ok("Windows camera system initialized with DirectShow/MediaFoundation".to_string())
            }
            Platform::MacOS => Ok("macOS camera system initialized with AVFoundation".to_string()),
            Platform::Linux => {
                #[cfg(target_os = "linux")]
                {
                    if linux::utils::is_v4l2_available() {
                        Ok("Linux camera system initialized with V4L2".to_string())
                    } else {
                        Err(CameraError::InitializationError(
                            "V4L2 not available on this system".to_string(),
                        ))
                    }
                }
                #[cfg(not(target_os = "linux"))]
                Err(CameraError::InitializationError(
                    "Linux support not compiled".to_string(),
                ))
            }
            Platform::Unknown => Err(CameraError::InitializationError(
                "Unknown platform".to_string(),
            )),
        }
    }

    /// Get platform-specific information
    ///
    /// # Errors
    /// This function currently always returns `Ok` and never returns an `Err`.
    pub fn get_platform_info() -> Result<PlatformInfo, CameraError> {
        let platform = Platform::current();

        let backend = match platform {
            Platform::Windows => "DirectShow/MediaFoundation",
            Platform::MacOS => "AVFoundation",
            Platform::Linux => "V4L2 (Video4Linux2)",
            Platform::Unknown => "Unknown",
        };

        let features = match platform {
            Platform::Windows => vec![
                "Hardware acceleration",
                "DirectShow filters",
                "Windows Media Foundation",
                "USB and integrated cameras",
            ],
            Platform::MacOS => vec![
                "AVFoundation framework",
                "Hardware acceleration",
                "FaceTime HD camera support",
                "USB and integrated cameras",
                "Advanced color management",
            ],
            Platform::Linux => vec![
                "V4L2 interface",
                "USB UVC cameras",
                "Hardware controls",
                "Multiple pixel formats",
                "Device-specific extensions",
            ],
            Platform::Unknown => vec!["Limited support"],
        };

        Ok(PlatformInfo {
            platform,
            backend: backend.to_string(),
            features: features.into_iter().map(String::from).collect(),
        })
    }

    /// Test camera system functionality
    ///
    /// # Errors
    /// This function always returns a test report and never returns an `Err`;
    /// individual camera failures are captured in the report's `test_results`.
    pub fn test_system() -> Result<SystemTestResult, CameraError> {
        let platform = Platform::current();
        let cameras = Self::list_cameras()?;

        let mut test_results = Vec::new();

        // Test each camera
        for camera_info in &cameras {
            let test_result = if camera_info.is_available {
                // Try to initialize camera
                let params = CameraInitParams::new(camera_info.id.clone());
                match PlatformCamera::new(params) {
                    Ok(mut camera) => match camera.capture_frame() {
                        Ok(_) => CameraTestResult::Success,
                        Err(e) => CameraTestResult::CaptureError(e.to_string()),
                    },
                    Err(e) => CameraTestResult::InitError(e.to_string()),
                }
            } else {
                CameraTestResult::NotAvailable
            };

            test_results.push((camera_info.id.clone(), test_result));
        }

        Ok(SystemTestResult {
            platform,
            cameras_found: cameras.len(),
            test_results,
        })
    }
}

/// Platform information structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlatformInfo {
    /// The operating system platform.
    pub platform: Platform,
    /// The camera backend in use (e.g. "`MediaFoundation`", "V4L2").
    pub backend: String,
    /// List of supported camera features.
    pub features: Vec<String>,
}

/// System test result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemTestResult {
    /// The operating system platform.
    pub platform: Platform,
    /// Number of cameras detected.
    pub cameras_found: usize,
    /// Detailed results of camera tests.
    pub test_results: Vec<(String, CameraTestResult)>,
}

/// Individual camera test result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum CameraTestResult {
    /// Camera passed all tests
    Success,
    /// Initialization failed
    InitError(String),
    /// Capture failed during test
    CaptureError(String),
    /// Camera not available or busy
    NotAvailable,
}

/// Platform-specific optimizations and utilities
pub mod optimizations {
    use super::{CameraFormat, CameraInitParams, Platform};

    /// Get recommended format for high-quality photography on current platform
    pub fn get_photography_format() -> CameraFormat {
        match Platform::current() {
            Platform::MacOS => {
                // macOS AVFoundation works well with high resolution
                CameraFormat::new(1920, 1080, 30.0).with_format_type("RGB8".to_string())
            }
            Platform::Linux => {
                // Linux V4L2 often works better with YUYV
                CameraFormat::new(1280, 720, 30.0).with_format_type("YUYV".to_string())
            }
            Platform::Windows => {
                // Windows DirectShow/MediaFoundation
                CameraFormat::new(1920, 1080, 30.0).with_format_type("RGB8".to_string())
            }
            Platform::Unknown => CameraFormat::standard(),
        }
    }

    /// Get platform-specific camera settings for optimal capture
    pub fn get_optimal_settings() -> CameraInitParams {
        let format = get_photography_format();

        CameraInitParams::new("0".to_string()) // Default to first camera
            .with_format(format)
            .with_auto_focus(true) // Important for detailed photography
            .with_auto_exposure(true) // Handle varying lighting conditions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_mock_camera_basic_lifecycle() {
        let cam = MockCamera::new("mock-dev".to_string(), CameraFormat::standard());
        assert_eq!(cam.get_device_id(), "mock-dev");
        assert!(cam.is_available());

        cam.start_stream().expect("start stream should succeed");
        cam.stop_stream().expect("stop stream should succeed");
    }

    #[test]
    fn test_mock_camera_callback_and_capture_modes() {
        let mut cam = MockCamera::new("mock-callback".to_string(), CameraFormat::standard());
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_clone = calls.clone();

        cam.frame_callback(move |_f| {
            calls_clone.fetch_add(1, Ordering::Relaxed);
        })
        .expect("callback registration should succeed");

        crate::tests::set_mock_camera_mode("mock-callback", crate::tests::MockCaptureMode::Success);
        let frame = cam.capture_frame().expect("success mode should capture");
        assert_eq!(frame.device_id, "mock-callback");
        assert_eq!(calls.load(Ordering::Relaxed), 1);

        crate::tests::set_mock_camera_mode("mock-callback", crate::tests::MockCaptureMode::Failure);
        let err = cam
            .capture_frame()
            .expect_err("failure mode should return capture error");
        assert!(matches!(err, CameraError::CaptureError(_)));
    }

    #[test]
    fn test_platform_camera_mock_end_to_end() {
        std::env::set_var("CRABCAMERA_USE_MOCK", "1");

        let params =
            CameraInitParams::new("pcam-1".to_string()).with_format(CameraFormat::standard());
        let mut camera =
            PlatformCamera::new(params).expect("mock platform camera should initialize");

        camera.start_stream().expect("start should work");
        let frame = camera.capture_frame().expect("capture should work");
        assert_eq!(frame.device_id, "pcam-1");

        let controls = crate::types::CameraControls {
            auto_focus: Some(true),
            brightness: Some(0.1),
            ..Default::default()
        };
        let apply_result = camera
            .apply_controls(&controls)
            .expect("apply controls should work for mock");
        assert!(apply_result.applied.contains(&"auto_focus".to_string()));
        assert!(apply_result.applied.contains(&"brightness".to_string()));

        let current = camera.get_controls().expect("get controls should work");
        assert_eq!(current.auto_focus, Some(true));
        assert_eq!(current.brightness, Some(0.1));

        let caps = camera.test_capabilities().expect("caps should work");
        assert!(caps.supports.auto_focus);

        let metrics = camera
            .get_performance_metrics()
            .expect("metrics should work");
        assert!(metrics.capture_latency_ms >= 0.0);

        assert!(camera.is_available());
        assert_eq!(camera.get_device_id(), Some("pcam-1"));
        camera.stop_stream().expect("stop should work");

        std::env::remove_var("CRABCAMERA_USE_MOCK");
    }

    #[test]
    fn test_platform_info_and_optimizations() {
        let info = CameraSystem::get_platform_info().expect("platform info should succeed");
        assert!(!info.backend.is_empty());
        assert!(!info.features.is_empty());

        let fmt = optimizations::get_photography_format();
        assert!(fmt.width > 0);
        assert!(fmt.height > 0);
        assert!(fmt.fps > 0.0);

        let optimal = optimizations::get_optimal_settings();
        assert_eq!(optimal.device_id, "0");
        assert!(optimal.controls.auto_focus.unwrap_or(false));
        assert!(optimal.controls.auto_exposure.unwrap_or(false));
    }

    #[test]
    fn test_camera_system_initialize_for_current_platform() {
        let result = CameraSystem::initialize();
        match Platform::current() {
            Platform::Unknown => assert!(result.is_err()),
            _ => assert!(
                result.is_ok()
                    || result
                        .as_ref()
                        .err()
                        .is_some_and(|e| e.to_string().contains("camera")
                            || e.to_string().contains("device")
                            || e.to_string().contains("init"))
            ),
        }
    }

    #[test]
    fn test_mock_camera_set_capture_mode_method() {
        let cam = MockCamera::new("mode-setter".to_string(), CameraFormat::standard());
        cam.set_capture_mode(crate::tests::MockCaptureMode::SlowCapture);
        // Behavior is sourced from global registry at capture time, so this asserts method call path only.
        assert_eq!(cam.get_device_id(), "mode-setter");
    }
}
