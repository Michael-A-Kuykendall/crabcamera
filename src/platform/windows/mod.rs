// Windows platform implementation combining nokhwa capture with MediaFoundation controls

pub mod capture;
pub mod controls;

use self::controls::MediaFoundationControls;
use crate::errors::CameraError;
use crate::types::{CameraCapabilities, CameraControls, CameraFormat, CameraFrame};
use nokhwa::Camera;

/// Combined Windows camera interface with both capture and control capabilities
pub struct WindowsCamera {
    /// nokhwa camera for frame capture
    pub nokhwa_camera: Camera,
    /// MediaFoundation controls for advanced camera settings
    pub mf_controls: MediaFoundationControls,
    /// Device identifier
    pub device_id: String,
}

impl WindowsCamera {
    /// Create new Windows camera with both capture and control capabilities
    pub fn new(device_id: String, format: CameraFormat) -> Result<Self, CameraError> {
        log::info!(
            "Initializing Windows camera {} with MediaFoundation controls",
            device_id
        );

        // Initialize nokhwa camera for capture
        let nokhwa_camera = capture::initialize_camera(&device_id, format)?;

        // Initialize MediaFoundation controls
        let device_index = device_id
            .parse::<u32>()
            .map_err(|_| CameraError::InitializationError("Invalid device ID".to_string()))?;
        let mf_controls = MediaFoundationControls::new(device_index)?;

        Ok(WindowsCamera {
            nokhwa_camera,
            mf_controls,
            device_id,
        })
    }

    /// Capture a frame using nokhwa
    pub fn capture_frame(&mut self) -> Result<CameraFrame, CameraError> {
        capture::capture_frame(&mut self.nokhwa_camera, &self.device_id)
    }

    /// Apply camera controls using MediaFoundation
    pub fn apply_controls(
        &mut self,
        controls: &CameraControls,
    ) -> Result<Vec<String>, CameraError> {
        self.mf_controls.apply_controls(controls)
    }

    /// Get current camera control values
    pub fn get_controls(&self) -> Result<CameraControls, CameraError> {
        self.mf_controls.get_controls()
    }

    /// Test camera capabilities
    pub fn test_capabilities(&self) -> Result<CameraCapabilities, CameraError> {
        self.mf_controls.get_capabilities()
    }

    /// Start camera stream - must be called before capture_frame
    pub fn start_stream(&mut self) -> Result<(), CameraError> {
        log::debug!("Opening camera stream for device {}", self.device_id);
        self.nokhwa_camera
            .open_stream()
            .map_err(|e| CameraError::StreamError(format!("Failed to open stream: {}", e)))
    }

    /// Stop camera stream
    pub fn stop_stream(&mut self) -> Result<(), CameraError> {
        log::debug!("Stopping camera stream for device {}", self.device_id);
        self.nokhwa_camera
            .stop_stream()
            .map_err(|e| CameraError::StreamError(format!("Failed to stop stream: {}", e)))
    }

    /// Check if the stream is currently open
    pub fn is_stream_open(&self) -> bool {
        self.nokhwa_camera.is_stream_open()
    }

    /// Check if camera is available
    pub fn is_available(&self) -> bool {
        // Camera availability is determined by successful initialization
        // Since the Camera object was created successfully, it's available
        // For more robust checking, we could attempt a test frame capture
        true
    }

    /// Get device ID
    pub fn get_device_id(&self) -> &str {
        &self.device_id
    }
}

// Re-export public interface functions for compatibility
pub use capture::{capture_frame, initialize_camera, list_cameras};
