use tauri::command;
use crate::types::{CameraFrame, CameraInitParams, CameraFormat};
use crate::platform::PlatformCamera;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

// Global camera registry to manage active cameras
lazy_static::lazy_static! {
    static ref CAMERA_REGISTRY: Arc<Mutex<HashMap<String, PlatformCamera>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// Capture a single photo from the specified camera
#[command]
pub async fn capture_single_photo(device_id: Option<String>, format: Option<CameraFormat>) -> Result<CameraFrame, String> {
    log::info!("Capturing single photo from camera: {:?}", device_id);
    
    // Use default camera if none specified
    let camera_id = device_id.unwrap_or_else(|| "0".to_string());
    let capture_format = format.unwrap_or_else(|| CameraFormat::standard());
    
    // Try to get existing camera or create new one
    let mut camera = match get_or_create_camera(camera_id.clone(), capture_format).await {
        Ok(cam) => cam,
        Err(e) => {
            log::error!("Failed to get/create camera: {}", e);
            return Err(e);
        }
    };
    
    // Ensure stream is started
    if let Err(e) = camera.start_stream() {
        log::warn!("Failed to start camera stream: {}", e);
        // Continue anyway as some platforms don't require explicit stream start
    }
    
    // Capture frame
    match camera.capture_frame() {
        Ok(frame) => {
            log::info!("Successfully captured frame: {}x{} ({} bytes)", 
                frame.width, frame.height, frame.size_bytes);
            Ok(frame)
        }
        Err(e) => {
            log::error!("Failed to capture frame: {}", e);
            Err(format!("Failed to capture frame: {}", e))
        }
    }
}

/// Capture multiple photos in sequence
#[command]
pub async fn capture_photo_sequence(
    device_id: String,
    count: u32,
    interval_ms: u32,
    format: Option<CameraFormat>
) -> Result<Vec<CameraFrame>, String> {
    log::info!("Capturing {} photos from camera {} with {}ms interval", count, device_id, interval_ms);
    
    if count == 0 || count > 20 {
        return Err("Invalid photo count (must be 1-20)".to_string());
    }
    
    let capture_format = format.unwrap_or_else(|| CameraFormat::standard());
    let mut camera = match get_or_create_camera(device_id.clone(), capture_format).await {
        Ok(cam) => cam,
        Err(e) => return Err(e),
    };
    
    // Start stream once
    if let Err(e) = camera.start_stream() {
        log::warn!("Failed to start camera stream: {}", e);
    }
    
    let mut frames = Vec::new();
    
    for i in 0..count {
        log::debug!("Capturing photo {} of {}", i + 1, count);
        
        match camera.capture_frame() {
            Ok(frame) => frames.push(frame),
            Err(e) => {
                log::error!("Failed to capture frame {}: {}", i + 1, e);
                return Err(format!("Failed to capture frame {}: {}", i + 1, e));
            }
        }
        
        // Wait between captures (except for the last one)
        if i < count - 1 {
            tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms as u64)).await;
        }
    }
    
    log::info!("Successfully captured {} photos", frames.len());
    Ok(frames)
}

/// Start continuous capture from a camera (for live preview)
#[command]
pub async fn start_camera_preview(device_id: String, format: Option<CameraFormat>) -> Result<String, String> {
    log::info!("Starting camera preview for device: {}", device_id);
    
    let capture_format = format.unwrap_or_else(|| CameraFormat::standard());
    let camera = match get_or_create_camera(device_id.clone(), capture_format).await {
        Ok(cam) => cam,
        Err(e) => return Err(e),
    };
    
    match camera.start_stream() {
        Ok(_) => {
            log::info!("Camera preview started for device: {}", device_id);
            Ok(format!("Preview started for camera {}", device_id))
        }
        Err(e) => {
            log::error!("Failed to start camera preview: {}", e);
            Err(format!("Failed to start camera preview: {}", e))
        }
    }
}

/// Stop camera preview
#[command]
pub async fn stop_camera_preview(device_id: String) -> Result<String, String> {
    log::info!("Stopping camera preview for device: {}", device_id);
    
    let registry = CAMERA_REGISTRY.lock()
        .map_err(|_| "Failed to access camera registry".to_string())?;
    
    if let Some(camera) = registry.get(&device_id) {
        match camera.stop_stream() {
            Ok(_) => {
                log::info!("Camera preview stopped for device: {}", device_id);
                Ok(format!("Preview stopped for camera {}", device_id))
            }
            Err(e) => {
                log::error!("Failed to stop camera preview: {}", e);
                Err(format!("Failed to stop camera preview: {}", e))
            }
        }
    } else {
        let msg = format!("No active camera found with ID: {}", device_id);
        log::warn!("{}", msg);
        Err(msg)
    }
}

/// Release a camera (stop and remove from registry)
#[command]
pub async fn release_camera(device_id: String) -> Result<String, String> {
    log::info!("Releasing camera: {}", device_id);
    
    let mut registry = CAMERA_REGISTRY.lock()
        .map_err(|_| "Failed to access camera registry".to_string())?;
    
    if let Some(camera) = registry.remove(&device_id) {
        let _ = camera.stop_stream(); // Ignore errors on cleanup
        log::info!("Camera {} released", device_id);
        Ok(format!("Camera {} released", device_id))
    } else {
        let msg = format!("No active camera found with ID: {}", device_id);
        log::info!("{}", msg);
        Ok(msg) // Not an error if camera wasn't active
    }
}

/// Get capture statistics for a camera
#[command]
pub async fn get_capture_stats(device_id: String) -> Result<CaptureStats, String> {
    let registry = CAMERA_REGISTRY.lock()
        .map_err(|_| "Failed to access camera registry".to_string())?;
    
    if let Some(camera) = registry.get(&device_id) {
        let is_active = camera.is_available();
        let device_id_opt = camera.get_device_id();
        
        Ok(CaptureStats {
            device_id: device_id.clone(),
            is_active,
            device_info: device_id_opt.map(|s| s.to_string()),
        })
    } else {
        Ok(CaptureStats {
            device_id: device_id.clone(),
            is_active: false,
            device_info: None,
        })
    }
}

/// Save captured frame to disk
#[command]
pub async fn save_frame_to_disk(frame: CameraFrame, file_path: String) -> Result<String, String> {
    log::info!("Saving frame {} to disk: {}", frame.id, file_path);
    
    use std::io::Write;
    
    match std::fs::File::create(&file_path) {
        Ok(mut file) => {
            match file.write_all(&frame.data) {
                Ok(_) => {
                    log::info!("Frame saved successfully to: {}", file_path);
                    Ok(format!("Frame saved to {}", file_path))
                }
                Err(e) => {
                    log::error!("Failed to write frame data: {}", e);
                    Err(format!("Failed to write frame data: {}", e))
                }
            }
        }
        Err(e) => {
            log::error!("Failed to create file: {}", e);
            Err(format!("Failed to create file: {}", e))
        }
    }
}

// Helper functions

/// Get existing camera or create new one
async fn get_or_create_camera(device_id: String, format: CameraFormat) -> Result<PlatformCamera, String> {
    let mut registry = CAMERA_REGISTRY.lock()
        .map_err(|_| "Failed to access camera registry".to_string())?;
    
    // Check if camera already exists
    if registry.contains_key(&device_id) {
        // Remove and return the camera (we'll put it back after use)
        if let Some(camera) = registry.remove(&device_id) {
            log::debug!("Using existing camera: {}", device_id);
            return Ok(camera);
        }
    }
    
    // Create new camera
    log::debug!("Creating new camera: {}", device_id);
    let params = CameraInitParams::new(device_id.clone()).with_format(format);
    
    match PlatformCamera::new(params) {
        Ok(camera) => {
            registry.insert(device_id.clone(), camera);
            // Get the camera back out (this is a bit clunky but ensures proper ownership)
            if let Some(camera) = registry.remove(&device_id) {
                Ok(camera)
            } else {
                Err("Failed to retrieve created camera".to_string())
            }
        }
        Err(e) => {
            log::error!("Failed to create camera: {}", e);
            Err(format!("Failed to create camera: {}", e))
        }
    }
}

/// Capture statistics structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CaptureStats {
    pub device_id: String,
    pub is_active: bool,
    pub device_info: Option<String>,
}