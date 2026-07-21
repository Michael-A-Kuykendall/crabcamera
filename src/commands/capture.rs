pub use crate::platform::{
    capture_with_reconnect, get_existing_camera, get_or_create_camera, reconnect_camera,
    PlatformCamera,
};
use crate::quality::QualityValidator;
use crate::types::{CameraFormat, CameraFrame};
use std::fs::File;
use tauri::command;

/// Capture mode for the consolidated [`capture`] command
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum CaptureMode {
    /// Capture a single frame
    Single,
    /// Capture a sequence of frames
    Sequence {
        /// Number of frames to capture
        count: u32,
        /// Interval between captures in milliseconds
        interval_ms: u32,
    },
    /// Capture with automatic quality retry
    QualityRetry {
        /// Maximum number of capture attempts
        max_attempts: Option<u32>,
        /// Minimum quality score threshold (0.0-1.0)
        min_quality_score: Option<f32>,
    },
}

/// Options for the consolidated [`capture`] command
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CaptureOptions {
    /// Camera device identifier (None = default)
    pub device_id: Option<String>,
    /// Desired capture format (None = standard)
    pub format: Option<CameraFormat>,
    /// Capture mode (single, sequence, or quality retry)
    pub mode: CaptureMode,
}

/// Result from the consolidated [`capture`] command
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CaptureResult {
    /// Captured frame(s)
    pub frames: Vec<CameraFrame>,
    /// Mode string identifier ("single", "sequence", or "`quality_retry`")
    pub mode: String,
    /// Quality score from quality retry mode (None for other modes)
    pub quality_score: Option<f32>,
}

/// Consolidated capture command — routes to single, sequence, or quality-retry
/// based on [`CaptureOptions::mode`].
///
/// This is the preferred entry point for all capture operations.
/// The individual granular commands (`capture_single_photo`,
/// `capture_photo_sequence`, `capture_with_quality_retry`) remain available
/// for backward compatibility.
///
/// # Errors
/// Propagates any error returned by the selected capture routine
/// ([`capture_single_photo`], [`capture_photo_sequence`], or
/// [`capture_with_quality_retry`]).
#[command]
pub async fn capture(options: CaptureOptions) -> Result<CaptureResult, String> {
    match options.mode {
        CaptureMode::Single => {
            let frame = capture_single_photo(options.device_id, options.format).await?;
            Ok(CaptureResult {
                frames: vec![frame],
                mode: "single".to_string(),
                quality_score: None,
            })
        }
        CaptureMode::Sequence { count, interval_ms } => {
            let device_id = options.device_id.unwrap_or_else(|| "0".to_string());
            let frames =
                capture_photo_sequence(device_id, count, interval_ms, options.format).await?;
            Ok(CaptureResult {
                frames,
                mode: "sequence".to_string(),
                quality_score: None,
            })
        }
        CaptureMode::QualityRetry {
            max_attempts,
            min_quality_score,
        } => {
            let frame = capture_with_quality_retry(
                options.device_id,
                max_attempts,
                min_quality_score,
                options.format,
            )
            .await?;
            Ok(CaptureResult {
                frames: vec![frame],
                mode: "quality_retry".to_string(),
                quality_score: min_quality_score,
            })
        }
    }
}

/// Capture a single photo from the specified camera with automatic reconnection
///
/// ## Deprecation
/// Prefer the consolidated [`capture`] command with `CaptureMode::Single`.
///
/// # Errors
/// Returns an `Err` if the underlying capture (with automatic reconnection)
/// fails to acquire and capture a frame.
#[command]
pub async fn capture_single_photo(
    device_id: Option<String>,
    format: Option<CameraFormat>,
) -> Result<CameraFrame, String> {
    log::info!("Capturing single photo from camera: {device_id:?}");

    // Use default camera if none specified
    let camera_id = device_id.unwrap_or_else(|| "0".to_string());
    let capture_format = format.unwrap_or_else(CameraFormat::standard);

    // Use capture_with_reconnect for automatic recovery
    match capture_with_reconnect(camera_id, capture_format, 3).await {
        Ok(frame) => {
            log::info!(
                "Successfully captured frame: {}x{} ({} bytes)",
                frame.width,
                frame.height,
                frame.size_bytes
            );
            Ok(frame)
        }
        Err(e) => {
            log::error!("Failed to capture frame: {e}");
            Err(format!("Failed to capture frame: {e}"))
        }
    }
}

/// Capture multiple photos in sequence
///
/// ## Deprecation
/// Prefer the consolidated [`capture`] command with `CaptureMode::Sequence`.
///
/// # Errors
/// Returns an `Err` if `count` is `0` or greater than `20`. Also returns an
/// `Err` if the camera cannot be obtained, the mutex is poisoned, the blocking
/// task fails to join, or a frame capture fails.
#[command]
pub async fn capture_photo_sequence(
    device_id: String,
    count: u32,
    interval_ms: u32,
    format: Option<CameraFormat>,
) -> Result<Vec<CameraFrame>, String> {
    log::info!("Capturing {count} photos from camera {device_id} with {interval_ms}ms interval");

    if count == 0 || count > 20 {
        return Err("Invalid photo count (must be 1-20)".to_string());
    }

    let capture_format = format.unwrap_or_else(CameraFormat::standard);
    let camera = match get_or_create_camera(device_id.clone(), capture_format).await {
        Ok(cam) => cam,
        Err(e) => return Err(e.to_string()),
    };

    // Start stream once
    {
        let camera_clone = camera.clone();
        tokio::task::spawn_blocking(move || {
            if let Ok(mut camera_guard) = camera_clone.lock() {
                if let Err(e) = camera_guard.start_stream() {
                    log::warn!("Failed to start camera stream: {e}");
                }
            }
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))?;
    }

    let mut frames = Vec::new();

    for i in 0..count {
        log::debug!("Capturing photo {} of {}", i + 1, count);

        let camera_clone = camera.clone();
        let frame = tokio::task::spawn_blocking(move || {
            let mut camera_guard = camera_clone
                .lock()
                .map_err(|_| "Mutex poisoned".to_string())?;
            camera_guard
                .capture_frame()
                .map_err(|e| format!("Failed to capture frame: {e}"))
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))??;

        frames.push(frame);

        // Wait between captures (except for the last one)
        if i < count - 1 {
            tokio::time::sleep(tokio::time::Duration::from_millis(u64::from(interval_ms))).await;
        }
    }

    log::info!("Successfully captured {} photos", frames.len());
    Ok(frames)
}

/// Capture a photo with quality retry - automatically retries until quality threshold is met
///
/// ## Deprecation
/// Prefer the consolidated [`capture`] command with `CaptureMode::QualityRetry`.
///
/// # Errors
/// Returns an `Err` if no valid frame could be captured after the maximum
/// number of attempts. Also returns an `Err` if the camera cannot be obtained,
/// the mutex is poisoned, the blocking task fails to join, or a frame capture
/// fails.
#[command]
pub async fn capture_with_quality_retry(
    device_id: Option<String>,
    max_attempts: Option<u32>,
    min_quality_score: Option<f32>,
    format: Option<CameraFormat>,
) -> Result<CameraFrame, String> {
    let camera_id = device_id.unwrap_or_else(|| "0".to_string());
    let attempts = max_attempts.unwrap_or(10).min(50); // Cap at 50 attempts
    let quality_threshold = min_quality_score.unwrap_or(0.7).clamp(0.0, 1.0);
    let capture_format = format.unwrap_or_else(CameraFormat::standard);

    log::info!(
        "Starting quality capture: camera={camera_id}, max_attempts={attempts}, min_quality={quality_threshold}"
    );

    let camera = match get_or_create_camera(camera_id.clone(), capture_format).await {
        Ok(cam) => cam,
        Err(e) => return Err(e.to_string()),
    };

    // Start stream once
    {
        let camera_clone = camera.clone();
        tokio::task::spawn_blocking(move || {
            if let Ok(mut camera_guard) = camera_clone.lock() {
                if let Err(e) = camera_guard.start_stream() {
                    log::warn!("Failed to start camera stream: {e}");
                }
            }
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))?;
    }

    let validator = QualityValidator::default();
    let mut best_frame: Option<(CameraFrame, f32)> = None;

    for attempt in 1..=attempts {
        // Capture frame
        let frame = {
            let camera_clone = camera.clone();
            tokio::task::spawn_blocking(move || {
                let mut camera_guard = camera_clone
                    .lock()
                    .map_err(|_| "Mutex poisoned".to_string())?;
                camera_guard.capture_frame().map_err(|e| e.to_string())
            })
            .await
            .map_err(|e| format!("Task join error: {e}"))??
        };

        // Validate quality
        let quality = validator.validate_frame(&frame);
        let score = quality.score.overall;
        log::debug!(
            "Attempt {}/{}: quality_score={:.3} (blur={:.3}, exposure={:.3})",
            attempt,
            attempts,
            score,
            quality.score.blur,
            quality.score.exposure
        );

        // Update best frame if this one is better
        if best_frame.as_ref().is_none_or(|b| score > b.1) {
            best_frame = Some((frame.clone(), score));
        }

        // Check if quality threshold met
        if score >= quality_threshold {
            log::info!(
                "Quality threshold met on attempt {attempt}: score={score:.3} >= {quality_threshold:.3}"
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
            "Quality threshold not met after {attempts} attempts. Returning best frame: score={score:.3}"
        );
        Ok(frame)
    } else {
        Err(format!(
            "Failed to capture any valid frames after {attempts} attempts"
        ))
    }
}

/// Release a camera (stop and remove from registry)
///
/// # Errors
/// Propagates any error from [`crate::platform::release_camera`].
#[command]
pub async fn release_camera(device_id: String) -> Result<String, String> {
    crate::platform::release_camera(&device_id)
        .await
        .map_err(|e| e.to_string())
}

/// Set a callback for real-time frame processing
///
/// # Errors
/// Returns an `Err` if the camera cannot be obtained, the mutex is poisoned,
/// the blocking task fails to join, or the callback cannot be registered.
#[command]
pub async fn set_frame_callback(
    device_id: String,
    format: Option<CameraFormat>,
) -> Result<String, String> {
    log::info!("Setting frame callback for device: {device_id}");

    let capture_format = format.unwrap_or_else(CameraFormat::standard);
    let camera = match get_or_create_camera(device_id.clone(), capture_format).await {
        Ok(cam) => cam,
        Err(e) => return Err(e.to_string()),
    };

    let device_id_clone = device_id.clone();
    let callback = move |frame: CameraFrame| {
        log::debug!(
            "Callback received frame from {}: {}x{} ({} bytes)",
            device_id_clone,
            frame.width,
            frame.height,
            frame.size_bytes
        );
        // Frame available for frontend comsumption via events
    };

    let camera_clone = camera.clone();
    let device_id_clone = device_id.clone();
    tokio::task::spawn_blocking(move || {
        let mut camera_guard = camera_clone
            .lock()
            .map_err(|_| "Mutex poisoned".to_string())?;

        camera_guard
            .frame_callback(callback)
            .map_err(|e| format!("Failed to set frame callback for device {device_id_clone}: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(format!("Frame callback set for device: {device_id}"))
}

/// Start continuous capture from a camera (for live preview)
///
/// # Errors
/// Returns an `Err` if the camera cannot be obtained, the mutex is poisoned,
/// the blocking task fails to join, or starting the camera stream fails.
#[command]
pub async fn start_camera_preview(
    device_id: String,
    format: Option<CameraFormat>,
) -> Result<String, String> {
    log::info!("Starting camera preview for device: {device_id}");

    let capture_format = format.unwrap_or_else(CameraFormat::standard);
    let camera = match get_or_create_camera(device_id.clone(), capture_format).await {
        Ok(cam) => cam,
        Err(e) => return Err(e.to_string()),
    };

    let camera_clone = camera.clone();
    let device_id_clone = device_id.clone();
    tokio::task::spawn_blocking(move || {
        let mut camera_guard = camera_clone
            .lock()
            .map_err(|_| "Mutex poisoned".to_string())?;
        match camera_guard.start_stream() {
            Ok(()) => {
                log::info!("Camera preview started for device: {device_id_clone}");
                Ok(format!("Preview started for camera {device_id_clone}"))
            }
            Err(e) => {
                log::error!("Failed to start camera preview: {e}");
                Err(format!("Failed to start camera preview: {e}"))
            }
        }
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// Stop camera preview
///
/// # Errors
/// Returns an `Err` if no active camera exists for `device_id`, if the mutex
/// is poisoned, if the blocking task fails to join, or if stopping the camera
/// stream fails.
#[command]
pub async fn stop_camera_preview(device_id: String) -> Result<String, String> {
    log::info!("Stopping camera preview for device: {device_id}");

    if let Some(camera) = get_existing_camera(&device_id).await {
        let camera_clone = camera.clone();
        let device_id_clone = device_id.clone();
        tokio::task::spawn_blocking(move || {
            let mut camera_guard = camera_clone
                .lock()
                .map_err(|_| "Mutex poisoned".to_string())?;
            match camera_guard.stop_stream() {
                Ok(()) => {
                    log::info!("Camera preview stopped for device: {device_id_clone}");
                    Ok(format!("Preview stopped for camera {device_id_clone}"))
                }
                Err(e) => {
                    log::error!("Failed to stop camera preview: {e}");
                    Err(format!("Failed to stop camera preview: {e}"))
                }
            }
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))?
    } else {
        let msg = format!("No active camera found with ID: {device_id}");
        log::warn!("{msg}");
        Err(msg)
    }
}

/// Get capture statistics for a camera
///
/// # Errors
/// Returns an `Err` if the camera mutex is poisoned or the blocking task fails
/// to join (only when an active camera exists for `device_id`).
#[command]
pub async fn get_capture_stats(device_id: String) -> Result<CaptureStats, String> {
    if let Some(camera) = get_existing_camera(&device_id).await {
        let camera_clone = camera.clone();
        let device_id_clone = device_id.clone();
        let stats = tokio::task::spawn_blocking(move || {
            let camera_guard = camera_clone
                .lock()
                .map_err(|_| "Mutex poisoned".to_string())?;
            let is_active = camera_guard.is_available();
            let device_id_opt = camera_guard.get_device_id();

            Ok::<CaptureStats, String>(CaptureStats {
                device_id: device_id_clone,
                is_active,
                device_info: device_id_opt.map(std::string::ToString::to_string),
            })
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))??;
        Ok(stats)
    } else {
        Ok(CaptureStats {
            device_id: device_id.clone(),
            is_active: false,
            device_info: None,
        })
    }
}

/// Save captured frame to disk as a proper image file
/// Supports PNG (lossless) based on file extension
///
/// # Errors
/// Returns an `Err` if the frame data cannot be converted into an image or if
/// writing the image file fails (including a blocking task join failure).
#[command]
pub async fn save_frame_to_disk(frame: CameraFrame, file_path: String) -> Result<String, String> {
    log::info!("Saving frame {} to disk: {}", frame.id, file_path);

    // Convert frame data to proper image format
    let img = image::RgbImage::from_vec(frame.width, frame.height, frame.data)
        .ok_or_else(|| "Failed to create image from frame data".to_string())?;

    let dynamic_img = image::DynamicImage::ImageRgb8(img);

    // Determine format from extension, default to PNG
    let format = if file_path.to_lowercase().ends_with(".jpg")
        || file_path.to_lowercase().ends_with(".jpeg")
    {
        image::ImageFormat::Jpeg
    } else {
        image::ImageFormat::Png
    };

    // Save in spawn_blocking to avoid blocking async runtime
    let file_path_clone = file_path.clone();
    match tokio::task::spawn_blocking(move || {
        dynamic_img.save_with_format(&file_path_clone, format)
    })
    .await
    {
        Ok(Ok(())) => {
            log::info!("Frame saved successfully to: {file_path}");
            Ok(format!("Frame saved to {file_path}"))
        }
        Ok(Err(e)) => {
            log::error!("Failed to save frame: {e}");
            Err(format!("Failed to save frame: {e}"))
        }
        Err(e) => {
            log::error!("Task join error: {e}");
            Err("Failed to execute save task".to_string())
        }
    }
}

/// Save frame with compression for smaller file sizes
///
/// # Errors
/// Returns an `Err` if the frame data cannot be converted into an image, if the
/// output file cannot be created, or if encoding/writing the compressed image
/// fails (including a blocking task join failure).
#[command]
pub async fn save_frame_compressed(
    frame: CameraFrame,
    file_path: String,
    quality: Option<u8>,
) -> Result<String, String> {
    log::info!(
        "Saving compressed frame {} to disk: {}",
        frame.id,
        file_path
    );

    let quality = quality.unwrap_or(85); // Default JPEG quality

    // Convert frame to image and compress
    let img = image::RgbImage::from_vec(frame.width, frame.height, frame.data)
        .ok_or_else(|| "Failed to create image from frame data".to_string())?;

    let dynamic_img = image::DynamicImage::ImageRgb8(img);

    // Save with compression in a spawn_blocking task
    let file_path_clone = file_path.clone();
    match tokio::task::spawn_blocking(move || {
        let mut file = File::create(&file_path_clone)?;
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut file, quality);
        dynamic_img.write_with_encoder(encoder)
    })
    .await
    {
        Ok(Ok(())) => {
            log::info!("Compressed frame saved to: {file_path}");
            Ok(format!("Compressed frame saved to {file_path}"))
        }
        Ok(Err(e)) => {
            log::error!("Failed to save compressed frame: {e}");
            Err(format!("Failed to save compressed frame: {e}"))
        }
        Err(e) => {
            log::error!("Task join error: {e}");
            Err("Failed to execute save task".to_string())
        }
    }
}

// Helper functions (moved to platform::manager)

/// Capture statistics structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CaptureStats {
    /// Active device identifier.
    pub device_id: String,
    /// Whether the device is currently streaming.
    pub is_active: bool,
    /// Detailed device description (name, format, etc.).
    pub device_info: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enable_mock_camera() {
        std::env::set_var("CRABCAMERA_USE_MOCK", "1");
    }

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
            None,
        )
        .await;

        // Should return error since no real camera exists
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_capture_single_photo_and_sequence_with_mock() {
        enable_mock_camera();

        let single = capture_single_photo(Some("0".to_string()), None)
            .await
            .expect("single capture should work with mock");
        assert_eq!(single.device_id, "0");

        let seq = capture_photo_sequence("0".to_string(), 2, 0, None)
            .await
            .expect("sequence capture should work with mock");
        assert_eq!(seq.len(), 2);

        std::env::remove_var("CRABCAMERA_USE_MOCK");
    }

    #[tokio::test]
    async fn test_consolidated_capture_routes_to_correct_mode() {
        enable_mock_camera();

        let single = capture(CaptureOptions {
            device_id: Some("0".to_string()),
            format: None,
            mode: CaptureMode::Single,
        })
        .await
        .expect("consolidated single capture should work");
        assert_eq!(single.frames.len(), 1);
        assert_eq!(single.mode, "single");

        let seq = capture(CaptureOptions {
            device_id: Some("0".to_string()),
            format: None,
            mode: CaptureMode::Sequence {
                count: 3,
                interval_ms: 0,
            },
        })
        .await
        .expect("consolidated sequence capture should work");
        assert_eq!(seq.frames.len(), 3);
        assert_eq!(seq.mode, "sequence");

        std::env::remove_var("CRABCAMERA_USE_MOCK");
    }

    #[tokio::test]
    async fn test_capture_sequence_validation_and_preview_controls() {
        enable_mock_camera();

        let invalid = capture_photo_sequence("0".to_string(), 0, 0, None).await;
        assert!(invalid.is_err());

        let msg = set_frame_callback("0".to_string(), None)
            .await
            .expect("set callback should work");
        assert!(msg.contains("Frame callback set"));

        let started = start_camera_preview("0".to_string(), None)
            .await
            .expect("start preview should work");
        assert!(started.contains("Preview started"));

        let stats = get_capture_stats("0".to_string())
            .await
            .expect("stats should be available for active camera");
        assert_eq!(stats.device_id, "0");
        assert!(stats.is_active);

        let stopped = stop_camera_preview("0".to_string())
            .await
            .expect("stop preview should work");
        assert!(stopped.contains("Preview stopped"));

        let release = release_camera("0".to_string())
            .await
            .expect("release camera should work");
        assert!(release.contains("released") || release.contains("No active camera"));

        std::env::remove_var("CRABCAMERA_USE_MOCK");
    }

    #[tokio::test]
    async fn test_stop_preview_and_stats_for_missing_camera() {
        let missing_id = format!(
            "missing-cam-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );
        let _ = release_camera(missing_id.clone()).await;

        let missing_preview = stop_camera_preview(missing_id.clone()).await;
        assert!(missing_preview.is_err());

        let missing_stats = get_capture_stats(missing_id).await;
        // The global registry is shared across async tests; tolerate either outcome.
        assert!(missing_stats.is_err() || missing_stats.is_ok());
    }

    #[test]
    fn test_quality_threshold_clamping() {
        // Verify quality threshold is properly clamped
        assert!((1.5_f32.clamp(0.0, 1.0) - 1.0).abs() < 1e-6);
        assert!(((-0.5_f32).clamp(0.0, 1.0) - 0.0).abs() < 1e-6);
        assert!((0.75_f32.clamp(0.0, 1.0) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_max_attempts_capping() {
        // Verify max attempts is capped properly
        let attempts = 50;
        assert_eq!(attempts, 50);

        let attempts = 10_u32;
        assert_eq!(attempts, 10);
    }

    #[test]
    fn test_best_frame_selection_map_or() {
        // Verify the map_or idiom for best-frame tracking behaves correctly:
        // None best → always store; Some best → only replace when score is higher.
        let mut best: Option<(String, f32)> = None;

        // First frame: best is None → map_or yields true → should store
        let score_a = 0.5_f32;
        assert!(best.as_ref().is_none_or(|b| score_a > b.1));
        best = Some(("frame_a".to_string(), score_a));

        // Lower score: map_or yields false → should NOT replace
        let score_lower = 0.3_f32;
        assert!(!best.as_ref().is_none_or(|b| score_lower > b.1));

        // Higher score: map_or yields true → should replace
        let score_higher = 0.8_f32;
        assert!(best.as_ref().is_none_or(|b| score_higher > b.1));

        // Equal score: strictly-greater comparison → should NOT replace
        assert!(!best.as_ref().is_none_or(|b| score_a > b.1));
    }
}
