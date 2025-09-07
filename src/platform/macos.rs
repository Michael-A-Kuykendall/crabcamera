use nokhwa::{Camera, query, pixel_format::RgbFormat, utils::{RequestedFormat, RequestedFormatType}};
use crate::types::{CameraDeviceInfo, CameraFormat, CameraFrame, CameraInitParams};
use crate::errors::CameraError;
use std::sync::{Arc, Mutex};

/// List available cameras on macOS
pub fn list_cameras() -> Result<Vec<CameraDeviceInfo>, CameraError> {
    let cameras = query(nokhwa::utils::ApiBackend::AVFoundation)
        .map_err(|e| CameraError::InitializationError(format!("Failed to query cameras: {}", e)))?;
    
    let mut device_list = Vec::new();
    for camera_info in cameras {
        let mut device = CameraDeviceInfo::new(
            camera_info.index().to_string(),
            camera_info.human_name(),
        );
        
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
pub fn initialize_camera(params: CameraInitParams) -> Result<MacOSCamera, CameraError> {
    let device_index = params.device_id.parse::<u32>()
        .map_err(|_| CameraError::InitializationError("Invalid device ID".to_string()))?;
    
    // Create requested format based on the desired format
    let requested_format = RequestedFormat::new::<RgbFormat>(
        RequestedFormatType::Exact(nokhwa::utils::FrameFormat::new(
            nokhwa::utils::Resolution::new(params.format.width, params.format.height),
            nokhwa::utils::FrameRate::new(params.format.fps as u32),
            nokhwa::pixel_format::RgbFormat::Rgb,
        ))
    );
    
    let camera = Camera::new(nokhwa::utils::CameraIndex::Index(device_index), requested_format)
        .map_err(|e| CameraError::InitializationError(format!("Failed to initialize camera: {}", e)))?;
    
    Ok(MacOSCamera {
        camera: Arc::new(Mutex::new(camera)),
        device_id: params.device_id,
        format: params.format,
    })
}

/// macOS-specific camera wrapper
pub struct MacOSCamera {
    camera: Arc<Mutex<Camera>>,
    device_id: String,
    format: CameraFormat,
}

impl MacOSCamera {
    /// Capture frame from macOS camera using AVFoundation
    pub fn capture_frame(&self) -> Result<CameraFrame, CameraError> {
        let mut camera = self.camera.lock()
            .map_err(|_| CameraError::CaptureError("Failed to lock camera".to_string()))?;
        
        let frame = camera.frame()
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
        self.camera.lock().map(|c| c.is_stream_open()).unwrap_or(false)
    }

    /// Start camera stream
    pub fn start_stream(&self) -> Result<(), CameraError> {
        let mut camera = self.camera.lock()
            .map_err(|_| CameraError::InitializationError("Failed to lock camera".to_string()))?;
        
        camera.open_stream()
            .map_err(|e| CameraError::InitializationError(format!("Failed to start stream: {}", e)))?;
        
        Ok(())
    }

    /// Stop camera stream
    pub fn stop_stream(&self) -> Result<(), CameraError> {
        let mut camera = self.camera.lock()
            .map_err(|_| CameraError::InitializationError("Failed to lock camera".to_string()))?;
        
        camera.stop_stream()
            .map_err(|e| CameraError::InitializationError(format!("Failed to stop stream: {}", e)))?;
        
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