use tauri::command;
use crate::types::{CameraControls, BurstConfig, CameraFrame, WhiteBalance};
use crate::commands::capture::get_or_create_camera;
use std::time::Instant;

/// Apply advanced camera controls
#[command]
pub async fn set_camera_controls(device_id: String, controls: CameraControls) -> Result<String, String> {
    log::info!("Setting camera controls for device: {}", device_id);
    
    let camera_arc = get_or_create_camera(device_id.clone(), crate::types::CameraFormat::standard()).await?;
    let mut camera = camera_arc.lock().await;
    
    // Apply controls to camera (platform-specific implementation)
    match camera.apply_controls(&controls) {
        Ok(_) => {
            log::info!("Camera controls applied successfully for device: {}", device_id);
            Ok(format!("Controls applied to camera {}", device_id))
        }
        Err(e) => {
            log::error!("Failed to apply camera controls: {}", e);
            Err(format!("Failed to apply controls: {}", e))
        }
    }
}

/// Get current camera controls
#[command]
pub async fn get_camera_controls(device_id: String) -> Result<CameraControls, String> {
    log::info!("Getting camera controls for device: {}", device_id);
    
    let camera_arc = get_or_create_camera(device_id.clone(), crate::types::CameraFormat::standard()).await?;
    let camera = camera_arc.lock().await;
    
    match camera.get_controls() {
        Ok(controls) => {
            log::debug!("Retrieved camera controls for device: {}", device_id);
            Ok(controls)
        }
        Err(e) => {
            log::error!("Failed to get camera controls: {}", e);
            Err(format!("Failed to get controls: {}", e))
        }
    }
}

/// Capture burst sequence with advanced controls
#[command]
pub async fn capture_burst_sequence(
    device_id: String,
    config: BurstConfig,
) -> Result<Vec<CameraFrame>, String> {
    log::info!("Starting burst capture: {} frames from device {}", config.count, device_id);
    
    if config.count == 0 || config.count > 50 {
        return Err("Invalid burst count (must be 1-50)".to_string());
    }
    
    let camera_arc = get_or_create_camera(device_id.clone(), crate::types::CameraFormat::hd()).await?;
    let mut camera = camera_arc.lock().await;
    
    // Start stream
    if let Err(e) = camera.start_stream() {
        log::warn!("Failed to start camera stream: {}", e);
    }
    
    let mut frames = Vec::with_capacity(config.count as usize);
    let start_time = Instant::now();
    
    for i in 0..config.count {
        log::debug!("Capturing burst frame {} of {}", i + 1, config.count);
        
        // Apply exposure bracketing if configured
        if let Some(ref bracketing) = config.bracketing {
            if let Some(stop) = bracketing.stops.get(i as usize % bracketing.stops.len()) {
                let exposure_time = bracketing.base_exposure * 2.0_f32.powf(*stop);
                let controls = CameraControls {
                    auto_exposure: Some(false),
                    exposure_time: Some(exposure_time),
                    ..CameraControls::default()
                };
                
                if let Err(e) = camera.apply_controls(&controls) {
                    log::warn!("Failed to apply exposure bracketing: {}", e);
                }
            }
        }
        
        // Apply focus stacking if configured
        if config.focus_stacking {
            let focus_distance = (i as f32) / (config.count as f32 - 1.0); // 0.0 to 1.0
            let controls = CameraControls {
                auto_focus: Some(false),
                focus_distance: Some(focus_distance),
                ..CameraControls::default()
            };
            
            if let Err(e) = camera.apply_controls(&controls) {
                log::warn!("Failed to apply focus stacking: {}", e);
            }
            
            // Wait for focus adjustment
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }
        
        // Capture frame with performance monitoring
        let capture_start = Instant::now();
        match camera.capture_frame() {
            Ok(mut frame) => {
                let capture_time = capture_start.elapsed();
                
                // Add performance metadata
                frame.metadata.capture_settings = camera.get_controls().ok();
                
                frames.push(frame);
                log::debug!("Burst frame {} captured in {:?}", i + 1, capture_time);
            }
            Err(e) => {
                log::error!("Failed to capture burst frame {}: {}", i + 1, e);
                return Err(format!("Failed to capture burst frame {}: {}", i + 1, e));
            }
        }
        
        // Wait between captures (except for the last one)
        if i < config.count - 1 {
            tokio::time::sleep(tokio::time::Duration::from_millis(config.interval_ms as u64)).await;
        }
    }
    
    let total_time = start_time.elapsed();
    log::info!("Burst capture completed: {} frames in {:?} ({:.2} fps)", 
        frames.len(), total_time, frames.len() as f32 / total_time.as_secs_f32());
    
    // Auto-save if configured
    if config.auto_save {
        if let Some(ref save_dir) = config.save_directory {
            save_burst_sequence(&frames, save_dir).await?;
        }
    }
    
    Ok(frames)
}

/// Enable manual focus mode and set focus distance
#[command]
pub async fn set_manual_focus(device_id: String, focus_distance: f32) -> Result<String, String> {
    if !(0.0..=1.0).contains(&focus_distance) {
        return Err("Focus distance must be between 0.0 (infinity) and 1.0 (closest)".to_string());
    }
    
    let controls = CameraControls {
        auto_focus: Some(false),
        focus_distance: Some(focus_distance),
        ..CameraControls::default()
    };
    
    set_camera_controls(device_id, controls).await
}

/// Set manual exposure settings
#[command]
pub async fn set_manual_exposure(
    device_id: String,
    exposure_time: f32,
    iso_sensitivity: u32,
) -> Result<String, String> {
    if exposure_time <= 0.0 || exposure_time > 10.0 {
        return Err("Exposure time must be between 0.0 and 10.0 seconds".to_string());
    }
    
    if !(50..=12800).contains(&iso_sensitivity) {
        return Err("ISO sensitivity must be between 50 and 12800".to_string());
    }
    
    let controls = CameraControls {
        auto_exposure: Some(false),
        exposure_time: Some(exposure_time),
        iso_sensitivity: Some(iso_sensitivity),
        ..CameraControls::default()
    };
    
    set_camera_controls(device_id, controls).await
}

/// Set white balance mode
#[command]
pub async fn set_white_balance(device_id: String, white_balance: WhiteBalance) -> Result<String, String> {
    let controls = CameraControls {
        white_balance: Some(white_balance),
        ..CameraControls::default()
    };
    
    set_camera_controls(device_id, controls).await
}

/// Enable HDR mode with automatic exposure bracketing
#[command]
pub async fn capture_hdr_sequence(device_id: String) -> Result<Vec<CameraFrame>, String> {
    log::info!("Capturing HDR sequence from device: {}", device_id);
    
    let config = BurstConfig::hdr_burst();
    capture_burst_sequence(device_id, config).await
}

/// Capture focus stacked sequence for macro photography
#[command]
pub async fn capture_focus_stack(device_id: String, stack_count: u32) -> Result<Vec<CameraFrame>, String> {
    log::info!("Capturing focus stack: {} frames from device {}", stack_count, device_id);
    
    if !(3..=20).contains(&stack_count) {
        return Err("Focus stack count must be between 3 and 20".to_string());
    }
    
    let config = BurstConfig {
        count: stack_count,
        interval_ms: 1000,  // 1 second between focus adjustments
        bracketing: None,
        focus_stacking: true,
        auto_save: true,
        save_directory: Some("focus_stack".to_string()),
    };
    
    capture_burst_sequence(device_id, config).await
}

/// Get camera performance metrics
#[command]
pub async fn get_camera_performance(device_id: String) -> Result<crate::types::CameraPerformanceMetrics, String> {
    let camera_arc = get_or_create_camera(device_id.clone(), crate::types::CameraFormat::standard()).await?;
    let camera = camera_arc.lock().await;
    
    match camera.get_performance_metrics() {
        Ok(metrics) => {
            log::debug!("Performance metrics for {}: {:.2}ms latency, {:.2} fps", 
                device_id, metrics.capture_latency_ms, metrics.fps_actual);
            Ok(metrics)
        }
        Err(e) => {
            log::error!("Failed to get performance metrics: {}", e);
            Err(format!("Failed to get performance metrics: {}", e))
        }
    }
}


/// Test camera capabilities and return supported features
#[command]
pub async fn test_camera_capabilities(device_id: String) -> Result<crate::types::CameraCapabilities, String> {
    log::info!("Testing camera capabilities for device: {}", device_id);
    
    let camera_arc = get_or_create_camera(device_id.clone(), crate::types::CameraFormat::standard()).await?;
    let camera = camera_arc.lock().await;
    
    match camera.test_capabilities() {
        Ok(capabilities) => {
            log::info!("Camera {} capabilities: manual_focus={}, manual_exposure={}, max_res={}x{}", 
                device_id, capabilities.supports_manual_focus, capabilities.supports_manual_exposure,
                capabilities.max_resolution.0, capabilities.max_resolution.1);
            Ok(capabilities)
        }
        Err(e) => {
            log::error!("Failed to test camera capabilities: {}", e);
            Err(format!("Failed to test capabilities: {}", e))
        }
    }
}

// Helper functions

/// Save burst sequence to disk
async fn save_burst_sequence(frames: &[CameraFrame], save_dir: &str) -> Result<(), String> {
    log::info!("Saving {} frames to directory: {}", frames.len(), save_dir);
    
    // Create directory if it doesn't exist
    if let Err(e) = tokio::fs::create_dir_all(save_dir).await {
        return Err(format!("Failed to create directory {}: {}", save_dir, e));
    }
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    
    // Save each frame
    for (i, frame) in frames.iter().enumerate() {
        let filename = format!("{}/burst_{}_{:03}.jpg", save_dir, timestamp, i + 1);
        
        // Convert to JPEG for smaller file size
        let img = image::RgbImage::from_vec(frame.width, frame.height, frame.data.clone())
            .ok_or_else(|| "Failed to create image from frame data".to_string())?;
        
        let dynamic_img = image::DynamicImage::ImageRgb8(img);
        
        // Save with compression in a spawn_blocking task
        let filename_clone = filename.clone();
        match tokio::task::spawn_blocking(move || {
            dynamic_img.save_with_format(&filename_clone, image::ImageFormat::Jpeg)
        }).await {
            Ok(Ok(_)) => {
                log::debug!("Saved frame {} to {}", i + 1, filename);
            }
            Ok(Err(e)) => {
                log::error!("Failed to save frame {}: {}", i + 1, e);
                return Err(format!("Failed to save frame {}: {}", i + 1, e));
            }
            Err(e) => {
                log::error!("Task join error for frame {}: {}", i + 1, e);
                return Err(format!("Failed to save frame {}: task error", i + 1));
            }
        }
    }
    
    log::info!("Successfully saved {} frames to {}", frames.len(), save_dir);
    Ok(())
}

/// Calculate optimal exposure settings for current lighting
#[allow(dead_code)]
async fn calculate_optimal_exposure(camera: &mut crate::platform::PlatformCamera) -> Result<(f32, u32), String> {
    // Take a test shot to analyze lighting
    let test_frame = camera.capture_frame()
        .map_err(|e| format!("Failed to capture test frame: {}", e))?;
    
    // Calculate average brightness
    let mut brightness_sum = 0u64;
    let pixel_count = test_frame.data.len() / 3; // RGB8 format
    
    for chunk in test_frame.data.chunks(3) {
        if chunk.len() == 3 {
            // Luminance calculation
            let luminance = (0.299 * chunk[0] as f32 + 0.587 * chunk[1] as f32 + 0.114 * chunk[2] as f32) as u8;
            brightness_sum += luminance as u64;
        }
    }
    
    let average_brightness = brightness_sum as f32 / pixel_count as f32;
    
    // Calculate optimal exposure based on brightness
    let target_brightness = 128.0; // Mid-gray
    let exposure_adjustment = target_brightness / average_brightness;
    
    // Base settings
    let base_exposure = 1.0 / 125.0; // 1/125s
    let base_iso = 400;
    
    let optimal_exposure = (base_exposure * exposure_adjustment).clamp(1.0/4000.0, 1.0/2.0);
    let optimal_iso = if exposure_adjustment > 2.0 {
        (base_iso as f32 * (exposure_adjustment / 2.0)).min(3200.0) as u32
    } else {
        base_iso
    };
    
    log::debug!("Optimal exposure calculated: {}s at ISO {}", optimal_exposure, optimal_iso);
    Ok((optimal_exposure, optimal_iso))
}