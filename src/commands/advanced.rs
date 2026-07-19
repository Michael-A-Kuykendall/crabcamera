use crate::commands::capture::get_or_create_camera;
use crate::constants::*;
use crate::types::{BurstConfig, CameraControls, CameraFrame, ControlApplicationResult, WhiteBalance};
use std::time::Instant;
use tauri::command;

/// Apply advanced camera controls
#[command]
pub async fn set_camera_controls(
    device_id: String,
    controls: CameraControls,
) -> Result<ControlApplicationResult, String> {
    log::info!("Setting camera controls for device: {}", device_id);

    let camera_arc =
        get_or_create_camera(device_id.clone(), crate::types::CameraFormat::standard()).await?;

    let device_id_clone = device_id.clone();
    tokio::task::spawn_blocking(move || {
        let mut camera = camera_arc
            .lock()
            .map_err(|_| "Mutex poisoned".to_string())?;

        let result = camera.apply_controls(&controls).map_err(|e| {
            log::error!("Failed to apply camera controls: {}", e);
            format!("Failed to apply controls: {}", e)
        })?;

        log::info!(
            "Camera controls applied for device {} (applied={}, rejected={})",
            device_id_clone,
            result.applied.len(),
            result.rejected.len()
        );

        Ok(result)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Get current camera controls
#[command]
pub async fn get_camera_controls(device_id: String) -> Result<CameraControls, String> {
    log::info!("Getting camera controls for device: {}", device_id);

    let camera_arc =
        get_or_create_camera(device_id.clone(), crate::types::CameraFormat::standard()).await?;

    let device_id_clone = device_id.clone();
    tokio::task::spawn_blocking(move || {
        let camera = camera_arc
            .lock()
            .map_err(|_| "Mutex poisoned".to_string())?;

        match camera.get_controls() {
            Ok(controls) => {
                log::debug!("Retrieved camera controls for device: {}", device_id_clone);
                Ok(controls)
            }
            Err(e) => {
                log::error!("Failed to get camera controls: {}", e);
                Err(format!("Failed to get controls: {}", e))
            }
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Capture burst sequence with advanced controls
#[command]
pub async fn capture_burst_sequence(
    device_id: String,
    config: BurstConfig,
) -> Result<Vec<CameraFrame>, String> {
    log::info!(
        "Starting burst capture: {} frames from device {}",
        config.count,
        device_id
    );

    if config.count == 0 || config.count > 50 {
        return Err("Invalid burst count (must be 1-50)".to_string());
    }

    if config.focus_stacking && config.count < 2 {
        return Err("Focus stacking requires at least 2 frames (count >= 2)".to_string());
    }

    if let Some(ref bracketing) = config.bracketing {
        if bracketing.stops.is_empty() {
            return Err(
                "Exposure bracketing requires at least one stop value".to_string(),
            );
        }
        if bracketing.base_exposure <= 0.0 {
            return Err(
                "Exposure bracketing base_exposure must be greater than zero".to_string(),
            );
        }
    }

    let camera_arc =
        get_or_create_camera(device_id.clone(), crate::types::CameraFormat::hd()).await?;

    // Start stream
    {
        let camera_arc = camera_arc.clone();
        tokio::task::spawn_blocking(move || {
            if let Ok(mut camera) = camera_arc.lock() {
                if let Err(e) = camera.start_stream() {
                    log::warn!("Failed to start camera stream: {}", e);
                }
            }
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?;
    }

    let mut frames = Vec::with_capacity(config.count as usize);
    let start_time = Instant::now();

    for i in 0..config.count {
        log::debug!("Capturing burst frame {} of {}", i + 1, config.count);

        let camera_arc = camera_arc.clone();
        let config_clone = config.clone();

        let frame = tokio::task::spawn_blocking(move || {
            let mut camera = camera_arc
                .lock()
                .map_err(|_| "Mutex poisoned".to_string())?;

            // Apply exposure bracketing if configured
            if let Some(ref bracketing) = config_clone.bracketing {
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
            if config_clone.focus_stacking {
                let focus_distance = (i as f32) / (config_clone.count as f32 - 1.0); // 0.0 to 1.0
                let controls = CameraControls {
                    auto_focus: Some(false),
                    focus_distance: Some(focus_distance),
                    ..CameraControls::default()
                };

                if let Err(e) = camera.apply_controls(&controls) {
                    log::warn!("Failed to apply focus stacking: {}", e);
                }

                // Wait for focus adjustment (blocking sleep is okay here as we are in spawn_blocking)
                std::thread::sleep(std::time::Duration::from_millis(200));
            }

            // Capture frame with performance monitoring
            let capture_start = Instant::now();
            match camera.capture_frame() {
                Ok(mut frame) => {
                    let capture_time = capture_start.elapsed();

                    // Add performance metadata
                    frame.metadata.capture_settings = camera.get_controls().ok();

                    log::debug!("Burst frame {} captured in {:?}", i + 1, capture_time);
                    Ok(frame)
                }
                Err(e) => {
                    log::error!("Failed to capture burst frame {}: {}", i + 1, e);
                    Err(format!("Failed to capture burst frame {}: {}", i + 1, e))
                }
            }
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))??;

        frames.push(frame);

        // Wait between captures (except for the last one)
        if i < config.count - 1 {
            tokio::time::sleep(tokio::time::Duration::from_millis(
                config.interval_ms as u64,
            ))
            .await;
        }
    }

    let total_time = start_time.elapsed();
    log::info!(
        "Burst capture completed: {} frames in {:?} ({:.2} fps)",
        frames.len(),
        total_time,
        frames.len() as f32 / total_time.as_secs_f32()
    );

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
pub async fn set_manual_focus(device_id: String, focus_distance: f32) -> Result<ControlApplicationResult, String> {
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
) -> Result<ControlApplicationResult, String> {
    if exposure_time <= 0.0 || exposure_time > 10.0 {
        return Err("Exposure time must be between 0.0 and 10.0 seconds".to_string());
    }

    if !(MIN_ISO..=MAX_ISO).contains(&iso_sensitivity) {
        return Err(format!("ISO sensitivity must be between {} and {}", MIN_ISO, MAX_ISO));
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
pub async fn set_white_balance(
    device_id: String,
    white_balance: WhiteBalance,
) -> Result<ControlApplicationResult, String> {
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

/// Capture focus stacked sequence for macro photography (legacy - use focus_stack module)
#[command]
pub async fn capture_focus_stack_legacy(
    device_id: String,
    stack_count: u32,
) -> Result<Vec<CameraFrame>, String> {
    log::info!(
        "Capturing focus stack (legacy): {} frames from device {}",
        stack_count,
        device_id
    );

    if !(3..=20).contains(&stack_count) {
        return Err("Focus stack count must be between 3 and 20".to_string());
    }

    let config = BurstConfig {
        count: stack_count,
        interval_ms: 1000, // 1 second between focus adjustments
        bracketing: None,
        focus_stacking: true,
        auto_save: true,
        save_directory: Some("focus_stack".to_string()),
    };

    capture_burst_sequence(device_id, config).await
}

/// Get camera performance metrics
#[command]
pub async fn get_camera_performance(
    device_id: String,
) -> Result<crate::types::CameraPerformanceMetrics, String> {
    let camera_arc =
        get_or_create_camera(device_id.clone(), crate::types::CameraFormat::standard()).await?;

    let device_id_clone = device_id.clone();
    tokio::task::spawn_blocking(move || {
        let camera = camera_arc
            .lock()
            .map_err(|_| "Mutex poisoned".to_string())?;

        match camera.get_performance_metrics() {
            Ok(metrics) => {
                log::debug!(
                    "Performance metrics for {}: {:.2}ms latency, {:.2} fps",
                    device_id_clone,
                    metrics.capture_latency_ms,
                    metrics.fps_actual
                );
                Ok(metrics)
            }
            Err(e) => {
                log::error!("Failed to get performance metrics: {}", e);
                Err(format!("Failed to get performance metrics: {}", e))
            }
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Test camera capabilities and return supported features
#[command]
pub async fn test_camera_capabilities(
    device_id: String,
) -> Result<crate::types::CameraCapabilities, String> {
    log::info!("Testing camera capabilities for device: {}", device_id);

    let camera_arc =
        get_or_create_camera(device_id.clone(), crate::types::CameraFormat::standard()).await?;

    let device_id_clone = device_id.clone();
    tokio::task::spawn_blocking(move || {
        let camera = camera_arc
            .lock()
            .map_err(|_| "Mutex poisoned".to_string())?;

        match camera.test_capabilities() {
            Ok(capabilities) => {
                log::info!(
                    "Camera {} capabilities: manual_focus={}, manual_exposure={}, max_res={}x{}",
                    device_id_clone,
                    capabilities.supports_manual_focus,
                    capabilities.supports_manual_exposure,
                    capabilities.max_resolution.0,
                    capabilities.max_resolution.1
                );
                Ok(capabilities)
            }
            Err(e) => {
                log::error!("Failed to test camera capabilities: {}", e);
                Err(format!("Failed to test capabilities: {}", e))
            }
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
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
        })
        .await
        {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ExposureBracketing;

    fn enable_mock_camera() {
        std::env::set_var("CRABCAMERA_USE_MOCK", "1");
    }

    #[tokio::test]
    async fn test_set_manual_focus_rejects_out_of_range_value() {
        let result = set_manual_focus("0".to_string(), 1.5).await;
        assert!(result.is_err());
        assert!(
            result
                .err()
                .unwrap_or_default()
                .contains("Focus distance must be between 0.0")
        );
    }

    #[tokio::test]
    async fn test_set_manual_exposure_rejects_invalid_exposure_time() {
        let result = set_manual_exposure("0".to_string(), 0.0, MIN_ISO).await;
        assert!(result.is_err());
        assert!(
            result
                .err()
                .unwrap_or_default()
                .contains("Exposure time must be between 0.0 and 10.0 seconds")
        );
    }

    #[tokio::test]
    async fn test_set_manual_exposure_rejects_invalid_iso() {
        let result = set_manual_exposure("0".to_string(), 0.01, MIN_ISO.saturating_sub(1)).await;
        assert!(result.is_err());
        assert!(
            result
                .err()
                .unwrap_or_default()
                .contains("ISO sensitivity must be between")
        );
    }

    #[tokio::test]
    async fn test_capture_burst_sequence_rejects_invalid_count() {
        let config = BurstConfig {
            count: 0,
            interval_ms: 10,
            bracketing: None,
            focus_stacking: false,
            auto_save: false,
            save_directory: None,
        };

        let result = capture_burst_sequence("0".to_string(), config).await;
        assert!(result.is_err());
        assert!(
            result
                .err()
                .unwrap_or_default()
                .contains("Invalid burst count")
        );
    }

    #[tokio::test]
    async fn test_capture_burst_sequence_rejects_invalid_focus_stacking_count() {
        let config = BurstConfig {
            count: 1,
            interval_ms: 10,
            bracketing: None,
            focus_stacking: true,
            auto_save: false,
            save_directory: None,
        };

        let result = capture_burst_sequence("0".to_string(), config).await;
        assert!(result.is_err());
        assert!(
            result
                .err()
                .unwrap_or_default()
                .contains("Focus stacking requires at least 2 frames")
        );
    }

    #[tokio::test]
    async fn test_capture_burst_sequence_rejects_empty_bracketing_stops() {
        let config = BurstConfig {
            count: 3,
            interval_ms: 10,
            bracketing: Some(ExposureBracketing {
                stops: vec![],
                base_exposure: 0.01,
            }),
            focus_stacking: false,
            auto_save: false,
            save_directory: None,
        };

        let result = capture_burst_sequence("0".to_string(), config).await;
        assert!(result.is_err());
        assert!(
            result
                .err()
                .unwrap_or_default()
                .contains("Exposure bracketing requires at least one stop value")
        );
    }

    #[tokio::test]
    async fn test_capture_burst_sequence_rejects_non_positive_base_exposure() {
        let config = BurstConfig {
            count: 3,
            interval_ms: 10,
            bracketing: Some(ExposureBracketing {
                stops: vec![-1.0, 0.0, 1.0],
                base_exposure: 0.0,
            }),
            focus_stacking: false,
            auto_save: false,
            save_directory: None,
        };

        let result = capture_burst_sequence("0".to_string(), config).await;
        assert!(result.is_err());
        assert!(
            result
                .err()
                .unwrap_or_default()
                .contains("Exposure bracketing base_exposure must be greater than zero")
        );
    }

    #[tokio::test]
    async fn test_capture_focus_stack_legacy_rejects_out_of_range_stack_count() {
        let result = capture_focus_stack_legacy("0".to_string(), 2).await;
        assert!(result.is_err());
        assert!(
            result
                .err()
                .unwrap_or_default()
                .contains("Focus stack count must be between 3 and 20")
        );
    }

    #[tokio::test]
    async fn test_save_burst_sequence_rejects_invalid_frame_data_shape() {
        let invalid_frame = CameraFrame::new(vec![1, 2, 3], 16, 16, "0".to_string());
        let result = save_burst_sequence(&[invalid_frame], "test_outputs/invalid_burst").await;

        assert!(result.is_err());
        assert!(
            result
                .err()
                .unwrap_or_default()
                .contains("Failed to create image from frame data")
        );
    }

    #[tokio::test]
    async fn test_get_and_set_camera_controls_with_mock() {
        enable_mock_camera();

        let controls = CameraControls {
            auto_focus: Some(true),
            brightness: Some(0.1),
            ..Default::default()
        };

        let apply = set_camera_controls("0".to_string(), controls)
            .await
            .expect("set controls should succeed with mock");
        assert!(!apply.applied.is_empty());

        let fetched = get_camera_controls("0".to_string())
            .await
            .expect("get controls should succeed with mock");
        assert_eq!(fetched.auto_focus, Some(true));

        std::env::remove_var("CRABCAMERA_USE_MOCK");
    }

    #[tokio::test]
    async fn test_capture_burst_sequence_success_with_mock() {
        enable_mock_camera();

        let config = BurstConfig {
            count: 2,
            interval_ms: 0,
            bracketing: None,
            focus_stacking: false,
            auto_save: false,
            save_directory: None,
        };

        let frames = capture_burst_sequence("0".to_string(), config)
            .await
            .expect("burst capture should succeed with mock");
        assert_eq!(frames.len(), 2);

        std::env::remove_var("CRABCAMERA_USE_MOCK");
    }

    #[tokio::test]
    async fn test_performance_and_capabilities_with_mock() {
        enable_mock_camera();

        let metrics = get_camera_performance("0".to_string())
            .await
            .expect("performance should succeed");
        assert!(metrics.fps_actual > 0.0);

        let caps = test_camera_capabilities("0".to_string())
            .await
            .expect("capabilities should succeed");
        assert!(caps.supports_manual_focus);

        std::env::remove_var("CRABCAMERA_USE_MOCK");
    }

    #[tokio::test]
    async fn test_wrapper_commands_hdr_focus_legacy_and_white_balance() {
        enable_mock_camera();

        let wb = set_white_balance("0".to_string(), WhiteBalance::Daylight)
            .await
            .expect("set_white_balance should succeed with mock");
        assert!(!wb.applied.is_empty());

        let hdr = capture_hdr_sequence("0".to_string())
            .await
            .expect("hdr wrapper should succeed with mock");
        assert!(!hdr.is_empty());

        let stack = capture_focus_stack_legacy("0".to_string(), 3)
            .await
            .expect("focus stack legacy should succeed with mock");
        assert_eq!(stack.len(), 3);

        std::env::remove_var("CRABCAMERA_USE_MOCK");
    }
}
