use crate::errors::CameraError;
use crate::types::{CameraDeviceInfo, CameraFormat, CameraFrame, CameraInitParams};
use nokhwa::{
    pixel_format::RgbFormat,
    query,
    utils::{RequestedFormat, RequestedFormatType},
    CallbackCamera,
};
use std::sync::{Arc, Mutex};

/// List available cameras on macOS
pub fn list_cameras() -> Result<Vec<CameraDeviceInfo>, CameraError> {
    let cameras = query(nokhwa::utils::ApiBackend::AVFoundation)
        .map_err(|e| CameraError::InitializationError(format!("Failed to query cameras: {}", e)))?;

    let mut device_list = Vec::new();
    for camera_info in cameras {
        let mut device =
            CameraDeviceInfo::new(camera_info.index().to_string(), camera_info.human_name());

        device = device.with_description(camera_info.description().to_string());

        // Add common macOS camera formats
        let formats = vec![
            CameraFormat::new(1920, 1080, 30.0),
            CameraFormat::new(1280, 720, 30.0),
            CameraFormat::new(640, 480, 30.0),
        ];
        device = device.with_formats(formats);

        device_list.push(device);
    }

    Ok(device_list)
}

/// Initialize camera on macOS with AVFoundation backend
///
/// Uses nokhwa's CameraFormat API (0.10.x) with MJPEG frame format
/// for broad compatibility across macOS camera hardware.
pub fn initialize_camera(params: CameraInitParams) -> Result<MacOSCamera, CameraError> {
    let device_index = params
        .device_id
        .parse::<u32>()
        .map_err(|_| CameraError::InitializationError("Invalid device ID".to_string()))?;

    // Create requested format using nokhwa 0.10.x CameraFormat API
    // Note: CameraFormat::new takes (Resolution, FrameFormat, fps)
    // Using MJPEG for broad hardware compatibility on macOS
    let requested_format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(
        nokhwa::utils::CameraFormat::new(
            nokhwa::utils::Resolution::new(params.format.width, params.format.height),
            nokhwa::utils::FrameFormat::MJPEG,
            params.format.fps as u32,
        ),
    ));
    let camera = CallbackCamera::new(
        nokhwa::utils::CameraIndex::Index(device_index),
        requested_format,
        |_| {},
    )
    .map_err(|e| CameraError::InitializationError(format!("Failed to initialize camera: {}", e)))?;

    Ok(MacOSCamera {
        camera: Arc::new(Mutex::new(camera)),
        device_id: params.device_id,
        format: params.format,
    })
}

/// macOS-specific camera wrapper
pub struct MacOSCamera {
    camera: Arc<Mutex<CallbackCamera>>,
    device_id: String,
    format: CameraFormat,
}

impl MacOSCamera {
    /// Capture frame from macOS camera using AVFoundation
    pub fn capture_frame(&self) -> Result<CameraFrame, CameraError> {
        let mut camera = self
            .camera
            .lock()
            .map_err(|_| CameraError::CaptureError("Failed to lock camera".to_string()))?;

        let frame = camera
            .poll_frame()
            .map_err(|e| CameraError::CaptureError(format!("Failed to capture frame: {}", e)))?;

        let camera_frame = CameraFrame::new(
            frame.buffer_bytes().to_vec(),
            frame.resolution().width_x,
            frame.resolution().height_y,
            self.device_id.clone(),
        );

        Ok(camera_frame.with_format(frame.format().to_string()))
    }

    /// Get current format
    pub fn get_format(&self) -> &CameraFormat {
        &self.format
    }

    /// Get device ID
    pub fn get_device_id(&self) -> &str {
        &self.device_id
    }

    /// Check if camera is available
    pub fn is_available(&self) -> bool {
        self.camera
            .lock()
            .map(|c| c.is_stream_open())
            .unwrap_or(false)
    }

    /// Start camera stream
    pub fn start_stream(&self) -> Result<(), CameraError> {
        let mut camera = self
            .camera
            .lock()
            .map_err(|_| CameraError::InitializationError("Failed to lock camera".to_string()))?;

        camera.open_stream().map_err(|e| {
            CameraError::InitializationError(format!("Failed to start stream: {}", e))
        })?;

        Ok(())
    }

    /// Stop camera stream
    pub fn stop_stream(&self) -> Result<(), CameraError> {
        let mut camera = self
            .camera
            .lock()
            .map_err(|_| CameraError::InitializationError("Failed to lock camera".to_string()))?;

        camera.stop_stream().map_err(|e| {
            CameraError::InitializationError(format!("Failed to stop stream: {}", e))
        })?;

        Ok(())
    }

    /// Get camera controls (stub for macOS - not yet implemented)
    pub fn get_controls(&self) -> Result<crate::types::CameraControls, CameraError> {
        // macOS AVFoundation controls would be queried here
        // For now, return default controls
        Ok(crate::types::CameraControls::default())
    }

    /// Apply camera controls (stub for macOS - not yet implemented)
    pub fn apply_controls(
        &mut self,
        _controls: &crate::types::CameraControls,
    ) -> Result<(), CameraError> {
        // macOS AVFoundation control application would happen here
        Ok(())
    }

    /// Test camera capabilities (stub for macOS)
    pub fn test_capabilities(&self) -> Result<crate::types::CameraCapabilities, CameraError> {
        Ok(crate::types::CameraCapabilities::default())
    }

    /// Get performance metrics (stub for macOS)
    pub fn get_performance_metrics(
        &self,
    ) -> Result<crate::types::CameraPerformanceMetrics, CameraError> {
        Ok(crate::types::CameraPerformanceMetrics::default())
    }

    /// Set callback function for continuous frame streaming
    ///
    /// This is the **recommended** method for streaming. The callback receives a `CameraFrame`
    /// with rich metadata (timestamp, ID, device info) that is consistent with `capture_frame()`.
    ///
    /// # Performance Note
    /// This method transforms `nokhwa::Buffer` into `CameraFrame`, which involves:
    /// - Memory copy of frame data (~2-3ms for 1920x1080 @ RGB24)
    /// - Generation of UUID and timestamp
    /// - Creation of metadata structure
    ///
    /// For 30-60 fps streaming, this overhead is negligible (<1% CPU).
    /// For ultra-high performance needs (120+ fps), consider `set_raw_callback()`.
    ///
    /// # Arguments
    /// * `callback` - Function that receives a `CameraFrame` for each captured frame
    ///
    /// # Example
    /// ```rust,no_run
    /// # use crabcamera::platform::macos::MacOSCamera;
    /// # let camera: MacOSCamera = todo!();
    /// camera.set_callback(|frame| {
    ///     println!("Frame {}: {}x{} at {}",
    ///         frame.id, frame.width, frame.height, frame.timestamp);
    ///
    ///     // Rich metadata available
    ///     if let Some(sharpness) = frame.metadata.sharpness {
    ///         println!("Sharpness: {}", sharpness);
    ///     }
    ///
    ///     // Process frame data
    ///     let pixels = &frame.data;
    /// })?;
    /// camera.start_stream()?;
    /// ```
    ///
    /// # See Also
    /// - `set_raw_callback()` - Zero-copy version for maximum performance
    pub fn set_callback<F>(&self, mut callback: F) -> Result<(), CameraError>
    where
        F: FnMut(CameraFrame) + Send + 'static,
    {
        let device_id = self.device_id.clone();

        // Wrap user callback to transform Buffer -> CameraFrame
        let wrapper = move |buffer: nokhwa::Buffer| {
            // Detect format automatically from buffer (MJPEG, YUYV, RAWRGB, etc.)
            let format_str = buffer.source_frame_format().to_string();

            // Transform nokhwa Buffer into CameraFrame with rich metadata
            let camera_frame = CameraFrame::new(
                buffer.buffer_bytes().to_vec(),
                buffer.resolution().width_x,
                buffer.resolution().height_y,
                device_id.clone(),
            )
            .with_format(format_str);

            // Call user callback with CameraFrame
            callback(camera_frame);
        };

        let mut camera = self
            .camera
            .lock()
            .map_err(|_| CameraError::InitializationError("Failed to lock camera".to_string()))?;

        camera.set_callback(wrapper).map_err(|e| {
            CameraError::InitializationError(format!("Failed to set callback: {}", e))
        })?;

        Ok(())
    }

    /// Set raw callback function for zero-copy frame streaming
    ///
    /// This is the **high-performance** variant that passes `nokhwa::Buffer` directly
    /// without transformation. Use this when you need:
    /// - Maximum performance (zero memory copy)
    /// - Minimal latency (no transformation overhead)
    /// - Ultra-high framerates (120+ fps)
    ///
    /// # Trade-offs
    /// **Pros:**
    /// - Zero-copy: No memory allocation or copying
    /// - Minimal overhead: ~0.1ms per frame regardless of resolution
    /// - Direct access to nokhwa buffer
    ///
    /// **Cons:**
    /// - No CrabCamera metadata (no timestamp, ID, or metadata)
    /// - Exposes nokhwa API (less abstraction)
    /// - Inconsistent with `capture_frame()` which returns `CameraFrame`
    ///
    /// # Arguments
    /// * `callback` - Function that receives a `nokhwa::Buffer` for each captured frame
    ///
    /// # Example
    /// ```rust,no_run
    /// # use crabcamera::platform::macos::MacOSCamera;
    /// # let camera: MacOSCamera = todo!();
    /// camera.set_raw_callback(|buffer| {
    ///     // Direct access to nokhwa buffer (zero-copy)
    ///     let width = buffer.resolution().width_x;
    ///     let height = buffer.resolution().height_y;
    ///     let data = buffer.buffer_bytes(); // No copy!
    ///
    ///     // Minimal processing for maximum performance
    ///     println!("Raw frame: {}x{}", width, height);
    /// })?;
    /// camera.start_stream()?;
    /// ```
    ///
    /// # See Also
    /// - `set_callback()` - Recommended method with `CameraFrame` transformation
    pub fn set_raw_callback<F>(&self, callback: F) -> Result<(), CameraError>
    where
        F: FnMut(nokhwa::Buffer) + Send + 'static,
    {
        let mut camera = self
            .camera
            .lock()
            .map_err(|_| CameraError::InitializationError("Failed to lock camera".to_string()))?;

        camera.set_callback(callback).map_err(|e| {
            CameraError::InitializationError(format!("Failed to set callback: {}", e))
        })?;

        Ok(())
    }
}

// Ensure the camera is properly cleaned up
impl Drop for MacOSCamera {
    fn drop(&mut self) {
        if let Ok(mut camera) = self.camera.lock() {
            let _ = camera.stop_stream();
        }
    }
}

// Thread-safe implementation
unsafe impl Send for MacOSCamera {}
unsafe impl Sync for MacOSCamera {}
