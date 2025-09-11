//! Platform-specific camera implementations with unified interface
//!
//! This module provides a unified interface for camera operations across
//! different platforms (Windows, macOS, Linux) while maintaining platform-specific
//! optimizations and features.

use crate::types::{CameraDeviceInfo, CameraFormat, CameraFrame, CameraInitParams, Platform};
use crate::errors::CameraError;

// Platform-specific modules
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

/// Unified camera interface that abstracts platform differences
pub enum PlatformCamera {
    #[cfg(target_os = "windows")]
    Windows(nokhwa::Camera),
    
    #[cfg(target_os = "macos")]
    MacOS(macos::MacOSCamera),
    
    #[cfg(target_os = "linux")]
    Linux(linux::LinuxCamera),
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    Unsupported,
}

impl PlatformCamera {
    /// Create new platform camera from initialization parameters
    pub fn new(params: CameraInitParams) -> Result<Self, CameraError> {
        match Platform::current() {
            #[cfg(target_os = "windows")]
            Platform::Windows => {
                let camera = windows::initialize_camera(&params.device_id, params.format)?;
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
            
            _ => Err(CameraError::InitializationError("Unsupported platform".to_string())),
        }
    }

    /// Capture a single frame from the camera
    pub fn capture_frame(&mut self) -> Result<CameraFrame, CameraError> {
        match self {
            #[cfg(target_os = "windows")]
            PlatformCamera::Windows(camera) => windows::capture_frame(camera),
            
            #[cfg(target_os = "macos")]
            PlatformCamera::MacOS(camera) => camera.capture_frame(),
            
            #[cfg(target_os = "linux")]
            PlatformCamera::Linux(camera) => camera.capture_frame(),
            
            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            PlatformCamera::Unsupported => Err(CameraError::InitializationError("Unsupported platform".to_string())),
        }
    }

    /// Start camera stream
    pub fn start_stream(&self) -> Result<(), CameraError> {
        match self {
            #[cfg(target_os = "windows")]
            PlatformCamera::Windows(_) => {
                // Windows implementation doesn't need explicit stream start
                Ok(())
            }
            
            #[cfg(target_os = "macos")]
            PlatformCamera::MacOS(camera) => camera.start_stream(),
            
            #[cfg(target_os = "linux")]
            PlatformCamera::Linux(camera) => camera.start_stream(),
            
            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            PlatformCamera::Unsupported => Err(CameraError::InitializationError("Unsupported platform".to_string())),
        }
    }

    /// Stop camera stream
    pub fn stop_stream(&self) -> Result<(), CameraError> {
        match self {
            #[cfg(target_os = "windows")]
            PlatformCamera::Windows(_) => {
                // Windows implementation doesn't need explicit stream stop
                Ok(())
            }
            
            #[cfg(target_os = "macos")]
            PlatformCamera::MacOS(camera) => camera.stop_stream(),
            
            #[cfg(target_os = "linux")]
            PlatformCamera::Linux(camera) => camera.stop_stream(),
            
            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            PlatformCamera::Unsupported => Err(CameraError::InitializationError("Unsupported platform".to_string())),
        }
    }

    /// Check if camera is available
    pub fn is_available(&self) -> bool {
        match self {
            #[cfg(target_os = "windows")]
            PlatformCamera::Windows(_) => true, // Windows cameras are available when initialized
            
            #[cfg(target_os = "macos")]
            PlatformCamera::MacOS(camera) => camera.is_available(),
            
            #[cfg(target_os = "linux")]
            PlatformCamera::Linux(camera) => camera.is_available(),
            
            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            PlatformCamera::Unsupported => false,
        }
    }

    /// Get device ID
    pub fn get_device_id(&self) -> Option<&str> {
        match self {
            #[cfg(target_os = "windows")]
            PlatformCamera::Windows(_) => None, // Windows implementation needs updating
            
            #[cfg(target_os = "macos")]
            PlatformCamera::MacOS(camera) => Some(camera.get_device_id()),
            
            #[cfg(target_os = "linux")]
            PlatformCamera::Linux(camera) => Some(camera.get_device_id()),
            
            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            PlatformCamera::Unsupported => None,
        }
    }

    /// Apply camera controls
    pub fn apply_controls(&mut self, controls: &crate::types::CameraControls) -> Result<(), CameraError> {
        match self {
            #[cfg(target_os = "windows")]
            PlatformCamera::Windows(_camera) => {
                // TODO: Implement Windows camera controls
                log::warn!("Windows camera controls not yet implemented");
                Ok(())
            }
            
            #[cfg(target_os = "macos")]
            PlatformCamera::MacOS(camera) => camera.apply_controls(controls),
            
            #[cfg(target_os = "linux")]
            PlatformCamera::Linux(camera) => camera.apply_controls(controls),
            
            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            PlatformCamera::Unsupported => Err(CameraError::InitializationError("Unsupported platform".to_string())),
        }
    }

    /// Get current camera controls
    pub fn get_controls(&self) -> Result<crate::types::CameraControls, CameraError> {
        match self {
            #[cfg(target_os = "windows")]
            PlatformCamera::Windows(_camera) => {
                // Return default controls for Windows
                Ok(crate::types::CameraControls::default())
            }
            
            #[cfg(target_os = "macos")]
            PlatformCamera::MacOS(camera) => camera.get_controls(),
            
            #[cfg(target_os = "linux")]
            PlatformCamera::Linux(camera) => camera.get_controls(),
            
            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            PlatformCamera::Unsupported => Err(CameraError::InitializationError("Unsupported platform".to_string())),
        }
    }

    /// Test camera capabilities
    pub fn test_capabilities(&self) -> Result<crate::types::CameraCapabilities, CameraError> {
        match self {
            #[cfg(target_os = "windows")]
            PlatformCamera::Windows(_camera) => {
                // Return basic Windows capabilities
                Ok(crate::types::CameraCapabilities {
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
                })
            }
            
            #[cfg(target_os = "macos")]
            PlatformCamera::MacOS(camera) => camera.test_capabilities(),
            
            #[cfg(target_os = "linux")]
            PlatformCamera::Linux(camera) => camera.test_capabilities(),
            
            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            PlatformCamera::Unsupported => Err(CameraError::InitializationError("Unsupported platform".to_string())),
        }
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> Result<crate::types::CameraPerformanceMetrics, CameraError> {
        match self {
            #[cfg(target_os = "windows")]
            PlatformCamera::Windows(_camera) => {
                // Return basic metrics for Windows
                Ok(crate::types::CameraPerformanceMetrics::default())
            }
            
            #[cfg(target_os = "macos")]
            PlatformCamera::MacOS(camera) => camera.get_performance_metrics(),
            
            #[cfg(target_os = "linux")]
            PlatformCamera::Linux(camera) => camera.get_performance_metrics(),
            
            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            PlatformCamera::Unsupported => Err(CameraError::InitializationError("Unsupported platform".to_string())),
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
    pub fn list_cameras() -> Result<Vec<CameraDeviceInfo>, CameraError> {
        match Platform::current() {
            #[cfg(target_os = "windows")]
            Platform::Windows => windows::list_cameras(),
            
            #[cfg(target_os = "macos")]
            Platform::MacOS => macos::list_cameras(),
            
            #[cfg(target_os = "linux")]
            Platform::Linux => linux::list_cameras(),
            
            _ => Err(CameraError::InitializationError("Unsupported platform".to_string())),
        }
    }

    /// Initialize the camera system for the current platform
    pub fn initialize() -> Result<String, CameraError> {
        match Platform::current() {
            Platform::Windows => Ok("Windows camera system initialized with DirectShow/MediaFoundation".to_string()),
            Platform::MacOS => Ok("macOS camera system initialized with AVFoundation".to_string()),
            Platform::Linux => {
                #[cfg(target_os = "linux")]
                {
                    if linux::utils::is_v4l2_available() {
                        Ok("Linux camera system initialized with V4L2".to_string())
                    } else {
                        Err(CameraError::InitializationError("V4L2 not available on this system".to_string()))
                    }
                }
                #[cfg(not(target_os = "linux"))]
                Err(CameraError::InitializationError("Linux support not compiled".to_string()))
            }
            Platform::Unknown => Err(CameraError::InitializationError("Unknown platform".to_string())),
        }
    }

    /// Get platform-specific information
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
                    Ok(mut camera) => {
                        match camera.capture_frame() {
                            Ok(_) => CameraTestResult::Success,
                            Err(e) => CameraTestResult::CaptureError(e.to_string()),
                        }
                    }
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
    pub platform: Platform,
    pub backend: String,
    pub features: Vec<String>,
}

/// System test result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemTestResult {
    pub platform: Platform,
    pub cameras_found: usize,
    pub test_results: Vec<(String, CameraTestResult)>,
}

/// Individual camera test result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum CameraTestResult {
    Success,
    InitError(String),
    CaptureError(String),
    NotAvailable,
}

/// Platform-specific optimizations and utilities
pub mod optimizations {
    use super::*;

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
            .with_auto_focus(true)  // Important for detailed photography
            .with_auto_exposure(true) // Handle varying lighting conditions
    }
}