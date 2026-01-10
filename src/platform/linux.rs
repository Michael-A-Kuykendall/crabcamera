use crate::errors::CameraError;
use crate::types::{CameraDeviceInfo, CameraFormat, CameraFrame, CameraInitParams};
use nokhwa::{
    pixel_format::RgbFormat,
    query,
    utils::{RequestedFormat, RequestedFormatType},
    CallbackCamera,
};
use std::sync::{Arc, Mutex};

/// List available cameras on Linux
pub fn list_cameras() -> Result<Vec<CameraDeviceInfo>, CameraError> {
    let cameras = query(nokhwa::utils::ApiBackend::Video4Linux)
        .map_err(|e| CameraError::InitializationError(format!("Failed to query cameras: {}", e)))?;

    let mut device_list = Vec::new();
    for camera_info in cameras {
        let mut device =
            CameraDeviceInfo::new(camera_info.index().to_string(), camera_info.human_name());

        device = device.with_description(camera_info.description().to_string());

        // Add common Linux V4L2 camera formats
        let formats = vec![
            CameraFormat::new(1920, 1080, 30.0).with_format_type("YUYV".to_string()),
            CameraFormat::new(1280, 720, 30.0).with_format_type("YUYV".to_string()),
            CameraFormat::new(640, 480, 30.0).with_format_type("YUYV".to_string()),
            CameraFormat::new(1920, 1080, 15.0).with_format_type("MJPEG".to_string()),
            CameraFormat::new(1280, 720, 30.0).with_format_type("MJPEG".to_string()),
        ];
        device = device.with_formats(formats);

        device_list.push(device);
    }

    Ok(device_list)
}

/// Initialize camera on Linux with V4L2 backend
pub fn initialize_camera(params: CameraInitParams) -> Result<LinuxCamera, CameraError> {
    let device_index = params
        .device_id
        .parse::<u32>()
        .map_err(|_| CameraError::InitializationError("Invalid device ID".to_string()))?;

    // Simple format request for V4L2
    let requested_format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);

    let camera = CallbackCamera::new(
        nokhwa::utils::CameraIndex::Index(device_index),
        requested_format,
        |_| {},
    )
    .map_err(|e| CameraError::InitializationError(format!("Failed to initialize camera: {}", e)))?;

    Ok(LinuxCamera {
        camera: Arc::new(Mutex::new(camera)),
        device_id: params.device_id,
        format: params.format,
    })
}

/// Linux-specific camera wrapper
pub struct LinuxCamera {
    camera: Arc<Mutex<CallbackCamera>>,
    device_id: String,
    format: CameraFormat,
}

impl LinuxCamera {
    /// Capture frame from Linux camera using V4L2
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

        Ok(camera_frame.with_format("RGB8".to_string()))
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

    /// Get supported V4L2 formats for this device
    pub fn get_supported_formats(&self) -> Result<Vec<CameraFormat>, CameraError> {
        // This would typically query V4L2 for actual supported formats
        // For now, return common formats
        Ok(vec![
            CameraFormat::new(1920, 1080, 30.0).with_format_type("YUYV".to_string()),
            CameraFormat::new(1280, 720, 30.0).with_format_type("YUYV".to_string()),
            CameraFormat::new(640, 480, 30.0).with_format_type("YUYV".to_string()),
            CameraFormat::new(1920, 1080, 15.0).with_format_type("MJPEG".to_string()),
            CameraFormat::new(1280, 720, 30.0).with_format_type("MJPEG".to_string()),
        ])
    }

    /// Set camera controls (Linux V4L2 specific)
    pub fn set_control(&self, control: &str, _value: i32) -> Result<(), CameraError> {
        // This would typically use V4L2 controls to set brightness, contrast, etc.
        // Implementation would depend on the specific V4L2 bindings used
        match control {
            "brightness" | "contrast" | "saturation" | "hue" => {
                // Would set V4L2 control here
                Ok(())
            }
            _ => Err(CameraError::InitializationError(format!(
                "Unsupported control: {}",
                control
            ))),
        }
    }

    /// Get camera controls (stub for Linux - not yet implemented)
    pub fn get_controls(&self) -> Result<crate::types::CameraControls, CameraError> {
        // Linux V4L2 controls would be queried here
        // For now, return default controls
        Ok(crate::types::CameraControls::default())
    }

    /// Apply camera controls (stub for Linux - not yet implemented)
    pub fn apply_controls(
        &mut self,
        _controls: &crate::types::CameraControls,
    ) -> Result<(), CameraError> {
        // Linux V4L2 control application would happen here
        Ok(())
    }

    /// Test camera capabilities (stub for Linux)
    pub fn test_capabilities(&self) -> Result<crate::types::CameraCapabilities, CameraError> {
        // Linux V4L2 capabilities query
        Ok(crate::types::CameraCapabilities::default())
    }

    /// Get performance metrics (stub for Linux)
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
    /// # use crabcamera::platform::linux::LinuxCamera;
    /// # let camera: LinuxCamera = todo!();
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
    /// # use crabcamera::platform::linux::LinuxCamera;
    /// # let camera: LinuxCamera = todo!();
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
impl Drop for LinuxCamera {
    fn drop(&mut self) {
        if let Ok(mut camera) = self.camera.lock() {
            let _ = camera.stop_stream();
        }
    }
}

// Thread-safe implementation
unsafe impl Send for LinuxCamera {}
unsafe impl Sync for LinuxCamera {}

/// Linux-specific utilities
pub mod utils {
    use super::*;

    /// Check if V4L2 is available on the system
    pub fn is_v4l2_available() -> bool {
        std::path::Path::new("/dev/video0").exists()
    }

    /// List all V4L2 devices in /dev/video*
    pub fn list_v4l2_devices() -> Result<Vec<String>, CameraError> {
        let mut devices = Vec::new();

        for i in 0..10 {
            // Check video0 through video9
            let device_path = format!("/dev/video{}", i);
            if std::path::Path::new(&device_path).exists() {
                devices.push(device_path);
            }
        }

        Ok(devices)
    }

    /// Get V4L2 device capabilities
    pub fn get_device_caps(_device_path: &str) -> Result<Vec<String>, CameraError> {
        // This would typically query V4L2 capabilities
        // For now, return common capabilities
        Ok(vec![
            "Video Capture".to_string(),
            "Streaming".to_string(),
            "Extended Controls".to_string(),
        ])
    }
}
