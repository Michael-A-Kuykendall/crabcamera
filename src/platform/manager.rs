use crate::platform::PlatformCamera;
use crate::types::{CameraFormat, CameraFrame, CameraInitParams};
use std::collections::HashMap;
use std::sync::{Arc, Mutex as SyncMutex};
use tokio::sync::RwLock;

// Global camera registry with async-friendly locking for the map, but sync locking for the camera
lazy_static::lazy_static! {
    static ref CAMERA_REGISTRY: Arc<RwLock<HashMap<String, Arc<SyncMutex<PlatformCamera>>>>> = Arc::new(RwLock::new(HashMap::new()));
}

/// Get existing camera without creating if it doesn't exist
pub async fn get_existing_camera(device_id: &str) -> Option<Arc<SyncMutex<PlatformCamera>>> {
    let registry = CAMERA_REGISTRY.read().await;
    registry.get(device_id).cloned()
}

/// Release a camera (stop and remove from registry)
pub async fn release_camera(device_id: &str) -> Result<String, String> {
    log::info!("Releasing camera: {}", device_id);

    let mut registry = CAMERA_REGISTRY.write().await;

    if let Some(camera) = registry.remove(device_id) {
        let camera_clone = camera.clone();
        let device_id_clone = device_id.to_string();
        tokio::task::spawn_blocking(move || {
            if let Ok(mut camera_guard) = camera_clone.lock() {
                let _ = camera_guard.stop_stream(); // Ignore errors on cleanup
                log::info!("Camera {} released", device_id_clone);
            }
        })
        .await
        .ok();
        Ok(format!("Camera {} released", device_id))
    } else {
        let msg = format!("No active camera found with ID: {}", device_id);
        log::info!("{}", msg);
        Ok(msg) // Not an error if camera wasn't active
    }
}

/// Get existing camera or create new one

pub async fn get_or_create_camera(
    device_id: String,
    format: CameraFormat,
) -> Result<Arc<SyncMutex<PlatformCamera>>, String> {
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
            let camera_arc = Arc::new(SyncMutex::new(camera));
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
pub async fn reconnect_camera(
    device_id: String,
    format: CameraFormat,
    max_retries: u32,
) -> Result<Arc<SyncMutex<PlatformCamera>>, String> {
    log::info!(
        "Attempting to reconnect camera: {} (max retries: {})",
        device_id,
        max_retries
    );

    // Remove old camera from registry
    {
        let mut registry = CAMERA_REGISTRY.write().await;
        if let Some(old_camera) = registry.remove(&device_id) {
            let old_camera_clone = old_camera.clone();
            tokio::task::spawn_blocking(move || {
                if let Ok(mut camera_guard) = old_camera_clone.lock() {
                    let _ = camera_guard.stop_stream();
                    log::debug!("Removed old camera instance from registry");
                }
            })
            .await
            .ok();
        }
    }

    // Retry connection with exponential backoff
    for attempt in 1..=max_retries {
        log::debug!(
            "Reconnection attempt {}/{} for camera: {}",
            attempt,
            max_retries,
            device_id
        );

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

    Err(format!(
        "Failed to reconnect camera after {} attempts",
        max_retries
    ))
}

/// Capture with automatic reconnection on failure
pub async fn capture_with_reconnect(
    device_id: String,
    format: CameraFormat,
    max_reconnect_attempts: u32,
) -> Result<CameraFrame, String> {
    log::debug!(
        "Attempting capture with reconnect for device: {}",
        device_id
    );

    let camera = match get_or_create_camera(device_id.clone(), format.clone()).await {
        Ok(cam) => cam,
        Err(e) => return Err(format!("Failed to get camera: {}", e)),
    };

    // Try normal capture first
    let camera_clone = camera.clone();
    let capture_result = tokio::task::spawn_blocking(move || {
        let mut camera_guard = camera_clone
            .lock()
            .map_err(|_| "Mutex poisoned".to_string())?;

        // Ensure stream is started
        if let Err(e) = camera_guard.start_stream() {
            log::warn!("Failed to start stream: {}", e);
        }

        // Discard warmup frames - cameras need time to stabilize exposure/focus
        // This is especially important for USB cameras that power up on stream start
        // Using 5 frames with 30ms delay for reasonable warmup without excessive latency
        for i in 0..5 {
            match camera_guard.capture_frame() {
                Ok(_) => {
                    log::debug!("Warmup frame {} captured", i + 1);
                }
                Err(e) => {
                    log::debug!(
                        "Warmup frame {} failed (normal during startup): {}",
                        i + 1,
                        e
                    );
                }
            }
            // Small delay between warmup frames
            std::thread::sleep(std::time::Duration::from_millis(30));
        }

        // Now capture the real frame
        camera_guard
            .capture_frame()
            .map_err(|e| format!("Initial capture failed: {}, attempting reconnect", e))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?;

    if let Ok(frame) = capture_result {
        return Ok(frame);
    }

    // Initial capture failed, try reconnecting
    let camera = reconnect_camera(device_id.clone(), format, max_reconnect_attempts).await?;

    // Try capture after reconnect with warmup
    tokio::task::spawn_blocking(move || {
        let mut camera_guard = camera.lock().map_err(|_| "Mutex poisoned".to_string())?;

        if let Err(e) = camera_guard.start_stream() {
            log::warn!("Failed to start stream after reconnect: {}", e);
        }

        // Warmup after reconnect too
        for _ in 0..10 {
            let _ = camera_guard.capture_frame();
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        camera_guard
            .capture_frame()
            .map_err(|e| format!("Capture failed after reconnection: {}", e))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}
