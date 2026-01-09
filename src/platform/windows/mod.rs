// Windows platform implementation combining nokhwa capture with MediaFoundation controls

pub mod capture;
pub mod controls;

use self::controls::MediaFoundationControls;
use crate::errors::CameraError;
use crate::types::{CameraCapabilities, CameraControls, CameraFormat, CameraFrame};
use nokhwa::CallbackCamera;

/// Combined Windows camera interface with both capture and control capabilities
pub struct WindowsCamera {
    /// nokhwa camera for frame capture
    pub nokhwa_camera: CallbackCamera,
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
        self.nokhwa_camera.is_stream_open().unwrap_or(false)
    }

    /// Check if camera is available
    pub fn is_available(&self) -> bool {
        // CallbackCamera availability is determined by successful initialization
        // Since the Camera object was created successfully, it's available
        // For more robust checking, we could attempt a test frame capture
        true
    }

    /// Get device ID
    pub fn get_device_id(&self) -> &str {
        &self.device_id
    }

    /// Set callback function for continuous frame streaming
    ///
    /// The callback will be called for each frame captured by the camera.
    /// Use this for high-performance streaming scenarios.
    ///
    /// # Arguments
    /// * `callback` - Function that receives a Buffer for each captured frame
    ///
    /// # Example
    /// ```rust,no_run
    /// camera.set_callback(|buffer| {
    ///     println!("Frame: {}x{}", buffer.resolution().width_x, buffer.resolution().height_y);
    /// })?;
    /// camera.start_stream()?;
    /// ```
    pub fn set_callback<F>(&mut self, callback: F) -> Result<(), CameraError>
    where
        F: FnMut(nokhwa::Buffer) + Send + 'static,
    {
        self.nokhwa_camera.set_callback(callback).map_err(|e| {
            CameraError::InitializationError(format!("Failed to set callback: {}", e))
        })?;

        Ok(())
    }
}

// Re-export public interface functions for compatibility
pub use capture::{capture_frame, initialize_camera, list_cameras};
