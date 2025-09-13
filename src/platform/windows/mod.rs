// Windows platform implementation combining nokhwa capture with MediaFoundation controls

pub mod capture;
pub mod controls;

use nokhwa::Camera;
use crate::types::{CameraControls, CameraCapabilities, CameraFormat, CameraFrame};
use crate::errors::CameraError;
use self::controls::MediaFoundationControls;

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
        log::info!("Initializing Windows camera {} with MediaFoundation controls", device_id);
        
        // Initialize nokhwa camera for capture
        let nokhwa_camera = capture::initialize_camera(&device_id, format)?;
        
        // Initialize MediaFoundation controls
        let device_index = device_id.parse::<u32>()
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
        capture::capture_frame(&mut self.nokhwa_camera)
    }

    /// Apply camera controls using MediaFoundation
    pub fn apply_controls(&mut self, controls: &CameraControls) -> Result<Vec<String>, CameraError> {
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

    /// Start camera stream (nokhwa handles this)
    pub fn start_stream(&self) -> Result<(), CameraError> {
        // nokhwa Camera doesn't require explicit stream start
        Ok(())
    }

    /// Stop camera stream (nokhwa handles this)
    pub fn stop_stream(&self) -> Result<(), CameraError> {
        // nokhwa Camera doesn't require explicit stream stop
        Ok(())
    }

    /// Check if camera is available
    pub fn is_available(&self) -> bool {
        // If we successfully created the camera, it's available
        true
    }

    /// Get device ID
    pub fn get_device_id(&self) -> &str {
        &self.device_id
    }
}

// Re-export public interface functions for compatibility
pub use capture::{list_cameras, initialize_camera, capture_frame};