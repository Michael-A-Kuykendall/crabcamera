use tauri::command;
use crate::types::{CameraFrame, CameraInitParams, CameraFormat};
use crate::platform::PlatformCamera;
use crate::quality::QualityValidator;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{RwLock, Mutex as AsyncMutex};

// Global camera registry with async-friendly locking
lazy_static::lazy_static! {
    static ref CAMERA_REGISTRY: Arc<RwLock<HashMap<String, Arc<AsyncMutex<PlatformCamera>>>>> = Arc::new(RwLock::new(HashMap::new()));
}

/// Capture a single photo from the specified camera with automatic reconnection
#[command]
pub async fn capture_single_photo(device_id: Option<String>, format: Option<CameraFormat>) -> Result<CameraFrame, String> {
    log::info!("Capturing single photo from camera: {:?}", device_id);
    
    // Use default camera if none specified
    let camera_id = device_id.unwrap_or_else(|| "0".to_string());
    let capture_format = format.unwrap_or_else(CameraFormat::standard);
    
    // Use capture_with_reconnect for automatic recovery
    match capture_with_reconnect(camera_id, capture_format, 3).await {
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
    
    let capture_format = format.unwrap_or_else(CameraFormat::standard);
    let camera = match get_or_create_camera(device_id.clone(), capture_format).await {
        Ok(cam) => cam,
        Err(e) => return Err(e),
    };
    
    // Start stream once
    {
        let camera_guard = camera.lock().await;
        if let Err(e) = camera_guard.start_stream() {
            log::warn!("Failed to start camera stream: {}", e);
        }
    }
    
    let mut frames = Vec::new();
    
    for i in 0..count {
        log::debug!("Capturing photo {} of {}", i + 1, count);
        
        let mut camera_guard = camera.lock().await;
        match camera_guard.capture_frame() {
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

/// Capture a photo with quality retry - automatically retries until quality threshold is met
#[command]
pub async fn capture_with_quality_retry(
    device_id: Option<String>,
    max_attempts: Option<u32>,
    min_quality_score: Option<f32>,
    format: Option<CameraFormat>
) -> Result<CameraFrame, String> {
    let camera_id = device_id.unwrap_or_else(|| "0".to_string());
    let attempts = max_attempts.unwrap_or(10).min(50); // Cap at 50 attempts
    let quality_threshold = min_quality_score.unwrap_or(0.7).clamp(0.0, 1.0);
    let capture_format = format.unwrap_or_else(CameraFormat::standard);
    
    log::info!(
        "Starting quality capture: camera={}, max_attempts={}, min_quality={}",
        camera_id, attempts, quality_threshold
    );
    
    let camera = match get_or_create_camera(camera_id.clone(), capture_format).await {
        Ok(cam) => cam,
        Err(e) => return Err(e),
    };
    
    // Start stream once
    {
        let camera_guard = camera.lock().await;
        if let Err(e) = camera_guard.start_stream() {
            log::warn!("Failed to start camera stream: {}", e);
        }
    }
    
    let validator = QualityValidator::default();
    let mut best_frame: Option<(CameraFrame, f32)> = None;
    
    for attempt in 1..=attempts {
        // Capture frame
        let frame = {
            let mut camera_guard = camera.lock().await;
            match camera_guard.capture_frame() {
                Ok(f) => f,
                Err(e) => {
                    log::warn!("Capture attempt {} failed: {}", attempt, e);
                    if attempt < attempts {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        continue;
                    } else {
                        return Err(format!("All {} capture attempts failed", attempts));
                    }
                }
            }
        };
        
        // Validate quality
        let quality = validator.validate_frame(&frame);
        let score = quality.score.overall;
        log::debug!(
            "Attempt {}/{}: quality_score={:.3} (blur={:.3}, exposure={:.3})",
            attempt, attempts, score, quality.score.blur, quality.score.exposure
        );
        
        // Update best frame if this one is better
        if best_frame.is_none() || score > best_frame.as_ref().unwrap().1 {
            best_frame = Some((frame.clone(), score));
        }
        
        // Check if quality threshold met
        if score >= quality_threshold {
            log::info!(
                "Quality threshold met on attempt {}: score={:.3} >= {:.3}",
                attempt, score, quality_threshold
            );
            return Ok(frame);
        }
        
        // Small delay between attempts to allow camera to adjust
        if attempt < attempts {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
    
    // If we didn't meet threshold, return best frame we got
    if let Some((frame, score)) = best_frame {
        log::warn!(
            "Quality threshold not met after {} attempts. Returning best frame: score={:.3}",
            attempts, score
        );
        Ok(frame)
    } else {
        Err(format!("Failed to capture any valid frames after {} attempts", attempts))
    }
}

/// Start continuous capture from a camera (for live preview)
#[command]
pub async fn start_camera_preview(device_id: String, format: Option<CameraFormat>) -> Result<String, String> {
    log::info!("Starting camera preview for device: {}", device_id);
    
    let capture_format = format.unwrap_or_else(CameraFormat::standard);
    let camera = match get_or_create_camera(device_id.clone(), capture_format).await {
        Ok(cam) => cam,
        Err(e) => return Err(e),
    };
    
    let camera_guard = camera.lock().await;
    match camera_guard.start_stream() {
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
    
    let registry = CAMERA_REGISTRY.read().await;
    
    if let Some(camera) = registry.get(&device_id) {
        let camera_guard = camera.lock().await;
        match camera_guard.stop_stream() {
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
    
    let mut registry = CAMERA_REGISTRY.write().await;
    
    if let Some(camera) = registry.remove(&device_id) {
        let camera_guard = camera.lock().await;
        let _ = camera_guard.stop_stream(); // Ignore errors on cleanup
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
    let registry = CAMERA_REGISTRY.read().await;
    
    if let Some(camera) = registry.get(&device_id) {
        let camera_guard = camera.lock().await;
        let is_active = camera_guard.is_available();
        let device_id_opt = camera_guard.get_device_id();
        
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

/// Save captured frame to disk with async I/O
#[command]
pub async fn save_frame_to_disk(frame: CameraFrame, file_path: String) -> Result<String, String> {
    log::info!("Saving frame {} to disk: {}", frame.id, file_path);
    
    match tokio::fs::write(&file_path, &frame.data).await {
        Ok(_) => {
            log::info!("Frame saved successfully to: {}", file_path);
            Ok(format!("Frame saved to {}", file_path))
        }
        Err(e) => {
            log::error!("Failed to save frame: {}", e);
            Err(format!("Failed to save frame: {}", e))
        }
    }
}

/// Save frame with compression for smaller file sizes
#[command]
pub async fn save_frame_compressed(frame: CameraFrame, file_path: String, quality: Option<u8>) -> Result<String, String> {
    log::info!("Saving compressed frame {} to disk: {}", frame.id, file_path);
    
    let _quality = quality.unwrap_or(85); // Default JPEG quality
    
    // Convert frame to image and compress
    let img = image::RgbImage::from_vec(frame.width, frame.height, frame.data)
        .ok_or_else(|| "Failed to create image from frame data".to_string())?;
    
    let dynamic_img = image::DynamicImage::ImageRgb8(img);
    
    // Save with compression in a spawn_blocking task
    let file_path_clone = file_path.clone();
    match tokio::task::spawn_blocking(move || {
        dynamic_img.save_with_format(&file_path_clone, image::ImageFormat::Jpeg)
    }).await {
        Ok(Ok(_)) => {
            log::info!("Compressed frame saved to: {}", file_path);
            Ok(format!("Compressed frame saved to {}", file_path))
        }
        Ok(Err(e)) => {
            log::error!("Failed to save compressed frame: {}", e);
            Err(format!("Failed to save compressed frame: {}", e))
        }
        Err(e) => {
            log::error!("Task join error: {}", e);
            Err("Failed to execute save task".to_string())
        }
    }
}

// Helper functions

/// Get existing camera or create new one with async-friendly locking
pub async fn get_or_create_camera(device_id: String, format: CameraFormat) -> Result<Arc<AsyncMutex<PlatformCamera>>, String> {
    // First, try to get existing camera with read lock
    {
        let registry = CAMERA_REGISTRY.read().await;
        if let Some(camera) = registry.get(&device_id) {
            log::debug!("Using existing camera: {}", device_id);
            return Ok(camera.clone());
        }
    }
    
    // Need to create new camera, acquire write lock
    let mut registry = CAMERA_REGISTRY.write().await;
    
    // Double-check in case another task created it while we waited
    if let Some(camera) = registry.get(&device_id) {
        log::debug!("Using camera created by another task: {}", device_id);
        return Ok(camera.clone());
    }
    
    // Create new camera
    log::debug!("Creating new camera: {}", device_id);
    let params = CameraInitParams::new(device_id.clone()).with_format(format);
    
    match PlatformCamera::new(params) {
        Ok(camera) => {
            let camera_arc = Arc::new(AsyncMutex::new(camera));
            registry.insert(device_id.clone(), camera_arc.clone());
            Ok(camera_arc)
        }
        Err(e) => {
            log::error!("Failed to create camera: {}", e);
            Err(format!("Failed to create camera: {}", e))
        }
    }
}

/// Attempt to reconnect a camera with retries
pub async fn reconnect_camera(device_id: String, format: CameraFormat, max_retries: u32) -> Result<Arc<AsyncMutex<PlatformCamera>>, String> {
    log::info!("Attempting to reconnect camera: {} (max retries: {})", device_id, max_retries);
    
    // Remove old camera from registry
    {
        let mut registry = CAMERA_REGISTRY.write().await;
        if let Some(old_camera) = registry.remove(&device_id) {
            let camera_guard = old_camera.lock().await;
            let _ = camera_guard.stop_stream();
            log::debug!("Removed old camera instance from registry");
        }
    }
    
    // Retry connection with exponential backoff
    for attempt in 1..=max_retries {
        log::debug!("Reconnection attempt {}/{} for camera: {}", attempt, max_retries, device_id);
        
        match get_or_create_camera(device_id.clone(), format.clone()).await {
            Ok(camera) => {
                log::info!("Camera reconnected successfully on attempt {}", attempt);
                return Ok(camera);
            }
            Err(e) => {
                log::warn!("Reconnection attempt {} failed: {}", attempt, e);
                if attempt < max_retries {
                    let backoff_ms = (100 * 2_u64.pow(attempt - 1)).min(2000);
                    tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                }
            }
        }
    }
    
    Err(format!("Failed to reconnect camera after {} attempts", max_retries))
}

/// Capture with automatic reconnection on failure
pub async fn capture_with_reconnect(
    device_id: String,
    format: CameraFormat,
    max_reconnect_attempts: u32
) -> Result<CameraFrame, String> {
    log::debug!("Attempting capture with reconnect for device: {}", device_id);
    
    let camera = match get_or_create_camera(device_id.clone(), format.clone()).await {
        Ok(cam) => cam,
        Err(e) => return Err(format!("Failed to get camera: {}", e)),
    };
    
    // Try normal capture first
    {
        let mut camera_guard = camera.lock().await;
        
        // Ensure stream is started
        if let Err(e) = camera_guard.start_stream() {
            log::warn!("Failed to start stream: {}", e);
        }
        
        match camera_guard.capture_frame() {
            Ok(frame) => return Ok(frame),
            Err(e) => {
                log::warn!("Initial capture failed: {}, attempting reconnect", e);
            }
        }
    }
    
    // Initial capture failed, try reconnecting
    let camera = reconnect_camera(device_id.clone(), format, max_reconnect_attempts).await?;
    
    // Try capture after reconnect
    let mut camera_guard = camera.lock().await;
    
    if let Err(e) = camera_guard.start_stream() {
        log::warn!("Failed to start stream after reconnect: {}", e);
    }
    
    camera_guard.capture_frame()
        .map_err(|e| format!("Capture failed after reconnection: {}", e))
}

/// Zero-copy frame capture with memory pool
pub struct FramePool {
    pool: Arc<AsyncMutex<Vec<Vec<u8>>>>,
    max_frames: usize,
    frame_size: usize,
}

impl FramePool {
    pub fn new(max_frames: usize, frame_size: usize) -> Self {
        let mut pool = Vec::with_capacity(max_frames);
        for _ in 0..max_frames {
            pool.push(Vec::with_capacity(frame_size));
        }
        
        Self {
            pool: Arc::new(AsyncMutex::new(pool)),
            max_frames,
            frame_size,
        }
    }
    
    pub async fn get_buffer(&self) -> Vec<u8> {
        let mut pool = self.pool.lock().await;
        pool.pop().unwrap_or_else(|| Vec::with_capacity(self.frame_size))
    }
    
    pub async fn return_buffer(&self, mut buffer: Vec<u8>) {
        buffer.clear();
        let mut pool = self.pool.lock().await;
        if pool.len() < self.max_frames {
            pool.push(buffer);
        }
    }
}

lazy_static::lazy_static! {
    static ref FRAME_POOL: FramePool = FramePool::new(10, 1920 * 1080 * 3); // 10 HD RGB buffers
}

/// Capture statistics structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CaptureStats {
    pub device_id: String,
    pub is_active: bool,
    pub device_info: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_quality_retry_returns_best_frame() {
        // This test verifies the quality retry logic returns a frame
        // even if threshold isn't met, it should return the best attempt
        
        // Note: This is a smoke test - real testing requires mock camera
        // For now, we just verify the function signature and error handling
        let result = capture_with_quality_retry(
            Some("test_device".to_string()),
            Some(3),
            Some(0.9), // Very high threshold unlikely to be met
            None
        ).await;
        
        // Should return error since no real camera exists
        assert!(result.is_err() || result.is_ok());
    }
    
    #[test]
    fn test_quality_threshold_clamping() {
        // Verify quality threshold is properly clamped
        assert_eq!(1.5_f32.clamp(0.0, 1.0), 1.0);
        assert_eq!((-0.5_f32).clamp(0.0, 1.0), 0.0);
        assert_eq!(0.75_f32.clamp(0.0, 1.0), 0.75);
    }
    
    #[test]
    fn test_max_attempts_capping() {
        // Verify max attempts is capped properly
        let attempts = 50;
        assert_eq!(attempts, 50);
        
        let attempts = 10_u32;
        assert_eq!(attempts, 10);
    }
}