use nokhwa::{Camera, query, pixel_format::RgbFormat, utils::{RequestedFormat, RequestedFormatType}};
use crate::types::{CameraDeviceInfo, CameraFormat, CameraFrame, CameraInitParams};
use crate::errors::CameraError;
use std::sync::{Arc, Mutex};

/// List available cameras on Linux
pub fn list_cameras() -> Result<Vec<CameraDeviceInfo>, CameraError> {
    let cameras = query(nokhwa::utils::ApiBackend::Video4Linux)
        .map_err(|e| CameraError::InitializationError(format!("Failed to query cameras: {}", e)))?;
    
    let mut device_list = Vec::new();
    for camera_info in cameras {
        let mut device = CameraDeviceInfo::new(
            camera_info.index().to_string(),
            camera_info.human_name(),
        );
        
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
    let device_index = params.device_id.parse::<u32>()
        .map_err(|_| CameraError::InitializationError("Invalid device ID".to_string()))?;
    
    // Create requested format - prefer YUYV for Linux compatibility
    let requested_format = RequestedFormat::new::<RgbFormat>(
        RequestedFormatType::Exact(nokhwa::utils::FrameFormat::new(
            nokhwa::utils::Resolution::new(params.format.width, params.format.height),
            nokhwa::utils::FrameRate::new(params.format.fps as u32),
            nokhwa::pixel_format::RgbFormat::Rgb,
        ))
    );
    
    let camera = Camera::new(nokhwa::utils::CameraIndex::Index(device_index), requested_format)
        .map_err(|e| CameraError::InitializationError(format!("Failed to initialize camera: {}", e)))?;
    
    Ok(LinuxCamera {
        camera: Arc::new(Mutex::new(camera)),
        device_id: params.device_id,
        format: params.format,
    })
}

/// Linux-specific camera wrapper
pub struct LinuxCamera {
    camera: Arc<Mutex<Camera>>,
    device_id: String,
    format: CameraFormat,
}

impl LinuxCamera {
    /// Capture frame from Linux camera using V4L2
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
    pub fn set_control(&self, control: &str, value: i32) -> Result<(), CameraError> {
        // This would typically use V4L2 controls to set brightness, contrast, etc.
        // Implementation would depend on the specific V4L2 bindings used
        match control {
            "brightness" | "contrast" | "saturation" | "hue" => {
                // Would set V4L2 control here
                Ok(())
            }
            _ => Err(CameraError::InitializationError(format!("Unsupported control: {}", control)))
        }
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
        
        for i in 0..10 {  // Check video0 through video9
            let device_path = format!("/dev/video{}", i);
            if std::path::Path::new(&device_path).exists() {
                devices.push(device_path);
            }
        }
        
        Ok(devices)
    }

    /// Get V4L2 device capabilities
    pub fn get_device_caps(device_path: &str) -> Result<Vec<String>, CameraError> {
        // This would typically query V4L2 capabilities
        // For now, return common capabilities
        Ok(vec![
            "Video Capture".to_string(),
            "Streaming".to_string(),
            "Extended Controls".to_string(),
        ])
    }
}