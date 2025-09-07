use nokhwa::{Camera, query, pixel_format::RgbFormat, utils::{RequestedFormat, RequestedFormatType}};
use crate::types::{CameraDeviceInfo, CameraFormat, CameraFrame};
use crate::errors::CameraError;

/// List available cameras on Windows
pub fn list_cameras() -> Result<Vec<CameraDeviceInfo>, CameraError> {
    let cameras = query(nokhwa::utils::ApiBackend::Auto)
        .map_err(|e| CameraError::InitializationError(format!("Failed to query cameras: {}", e)))?;
    
    let mut device_list = Vec::new();
    for camera_info in cameras {
        let mut device = CameraDeviceInfo::new(
            camera_info.index().to_string(),
            camera_info.human_name(),
        );
        
        device = device.with_description(camera_info.description().to_string());

        // Add common Windows camera formats
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

/// Initialize camera on Windows with DirectShow/MediaFoundation backend
pub fn initialize_camera(device_id: &str, _format: CameraFormat) -> Result<Camera, CameraError> {
    let device_index = device_id.parse::<u32>()
        .map_err(|_| CameraError::InitializationError("Invalid device ID".to_string()))?;
    
    let requested_format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
    
    let camera = Camera::new(nokhwa::utils::CameraIndex::Index(device_index), requested_format)
        .map_err(|e| CameraError::InitializationError(format!("Failed to initialize camera: {}", e)))?;
    
    Ok(camera)
}

/// Capture frame from Windows camera
pub fn capture_frame(camera: &mut Camera) -> Result<CameraFrame, CameraError> {
    let frame = camera.frame()
        .map_err(|e| CameraError::CaptureError(format!("Failed to capture frame: {}", e)))?;
    
    let camera_frame = CameraFrame::new(
        frame.buffer_bytes().to_vec(),
        frame.resolution().width_x,
        frame.resolution().height_y,
        "0".to_string(), // Default device ID - should be passed in properly
    );
    
    Ok(camera_frame.with_format("RGB8".to_string()))
}