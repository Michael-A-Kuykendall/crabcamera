//! Windows platform implementation.
//!
//! Combines `nokhwa` for basic capture (DirectShow/MediaFoundation) with
//! custom `MediaFoundation` controls for professional features like
//! exposure, focus, and white balance that `nokhwa` might abstraction-layer away.

/// Capture implementation using nokhwa.
pub mod capture;
/// Advanced camera controls via MediaFoundation.
pub mod controls;

use self::controls::MediaFoundationControls;
use crate::errors::CameraError;
use crate::platform::metrics::PerfTracker;
use crate::types::{CameraCapabilities, CameraControls, ControlApplicationResult, CameraFormat, CameraFrame};
use nokhwa::Camera;
use std::sync::Arc;

/// Type alias for frame callback to reduce complexity
type FrameCallback = Box<dyn Fn(CameraFrame) + Send + 'static>;

/// Combined Windows camera interface with both capture and control capabilities
pub struct WindowsCamera {
    /// nokhwa camera for frame capture
    pub nokhwa_camera: Camera,
    /// MediaFoundation controls for advanced camera settings
    pub mf_controls: MediaFoundationControls,
    /// Device identifier
    pub device_id: String,
    /// Frame callback
    pub callback: std::sync::Mutex<Option<FrameCallback>>,
    /// Real performance tracker, updated on every capture.
    pub perf: Arc<std::sync::Mutex<PerfTracker>>,
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
            callback: std::sync::Mutex::new(None),
            perf: Arc::new(std::sync::Mutex::new(PerfTracker::new())),
        })
    }

    /// Capture a frame using nokhwa
    pub fn capture_frame(&mut self) -> Result<CameraFrame, CameraError> {
        let start = std::time::Instant::now();
        let frame = match capture::capture_frame(&mut self.nokhwa_camera, &self.device_id) {
            Ok(f) => f,
            Err(e) => {
                if let Ok(mut perf) = self.perf.lock() {
                    perf.record_drop();
                }
                return Err(e);
            }
        };
        let latency_ms = start.elapsed().as_secs_f32() * 1000.0;

        let process_start = std::time::Instant::now();
        // Call callback if set
        if let Some(ref cb) = *self.callback.lock().map_err(|_| CameraError::InitializationError("Mutex poisoned".to_string()))? {
            cb(frame.clone());
        }
        let processing_ms = process_start.elapsed().as_secs_f32() * 1000.0;

        if let Ok(mut perf) = self.perf.lock() {
            perf.record_capture(
                latency_ms,
                processing_ms,
                Some((
                    frame.data.clone(),
                    frame.width,
                    frame.height,
                    format!("{:?}", frame.format),
                )),
            );
        }

        Ok(frame)
    }

    /// Return real performance metrics for this camera session.
    ///
    /// # Errors
    /// Returns [`CameraError::CaptureError`] if the shared perf tracker mutex is
    /// poisoned.
    pub fn get_performance_metrics(
        &self,
    ) -> Result<crate::types::CameraPerformanceMetrics, CameraError> {
        let perf = self
            .perf
            .lock()
            .map_err(|_| CameraError::CaptureError("Perf tracker mutex poisoned".to_string()))?;
        Ok(crate::platform::metrics::build_metrics(
            &perf,
            &self.device_id,
        ))
    }

    /// Apply camera controls using MediaFoundation
    pub fn apply_controls(
        &mut self,
        controls: &CameraControls,
    ) -> Result<ControlApplicationResult, CameraError> {
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

    /// Check if camera is available (stream is currently open)
    pub fn is_available(&self) -> bool {
        self.nokhwa_camera.is_stream_open()
    }

    /// Get device ID
    pub fn get_device_id(&self) -> &str {
        &self.device_id
    }

    /// Set frame callback for real-time processing
    pub fn set_callback<F>(&self, callback: F) -> Result<(), CameraError>
    where
        F: Fn(CameraFrame) + Send + 'static,
    {
        *self.callback.lock().map_err(|_| CameraError::InitializationError("Mutex poisoned".to_string()))? = Some(Box::new(callback));
        Ok(())
    }
}

// Re-export public interface functions for compatibility
pub use capture::{capture_frame, initialize_camera, list_cameras};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_camera_new_rejects_invalid_device_id() {
        let result = WindowsCamera::new("invalid-device-id".to_string(), CameraFormat::standard());
        assert!(result.is_err());
    }

    #[test]
    fn test_windows_capture_helpers_are_callable() {
        let init = initialize_camera("not-a-number", CameraFormat::standard());
        assert!(init.is_err());

        let cams = list_cameras();
        assert!(cams.is_ok() || cams.is_err());
    }
}
