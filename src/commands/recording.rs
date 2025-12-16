//! Tauri commands for video recording
//!
//! These commands provide an interface for recording video from cameras.

use tauri::command;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{RwLock, Mutex as AsyncMutex};

use crate::recording::{Recorder, RecordingConfig, RecordingStats, RecordingQuality};
use crate::platform::PlatformCamera;
use crate::types::CameraFormat;

// Global recorder registry
lazy_static::lazy_static! {
    static ref RECORDER_REGISTRY: Arc<RwLock<HashMap<String, Arc<AsyncMutex<RecordingSession>>>>> = 
        Arc::new(RwLock::new(HashMap::new()));
}

/// Active recording session combining camera and recorder
struct RecordingSession {
    recorder: Recorder,
    camera: Arc<AsyncMutex<PlatformCamera>>,
    is_running: bool,
}

/// Start recording from a camera to a file
/// 
/// # Arguments
/// * `device_id` - Camera device ID (or "0" for default)
/// * `output_path` - Path to save the MP4 file
/// * `width` - Video width in pixels
/// * `height` - Video height in pixels
/// * `fps` - Target frame rate
/// * `quality` - Recording quality preset (optional)
/// * `title` - Metadata title (optional)
/// 
/// # Returns
/// * Session ID for tracking the recording
#[command]
pub async fn start_recording(
    device_id: Option<String>,
    output_path: String,
    width: u32,
    height: u32,
    fps: f64,
    quality: Option<String>,
    title: Option<String>,
) -> Result<String, String> {
    let camera_id = device_id.unwrap_or_else(|| "0".to_string());
    log::info!("Starting recording from camera {} to {}", camera_id, output_path);

    // Parse quality preset
    let recording_quality = match quality.as_deref() {
        Some("low") | Some("720p") => Some(RecordingQuality::Low),
        Some("medium") | Some("1080p") => Some(RecordingQuality::Medium),
        Some("high") | Some("4k") => Some(RecordingQuality::High),
        _ => None,
    };

    // Build recording config
    let mut config = if let Some(q) = recording_quality {
        RecordingConfig::from_quality_with_fps(q, fps)
    } else {
        RecordingConfig::new(width, height, fps)
    };

    if let Some(t) = title {
        config = config.with_title(t);
    }

    // Initialize camera
    let camera = super::capture::get_or_create_camera(
        camera_id.clone(),
        CameraFormat::new(config.width, config.height, fps as f32),
    ).await.map_err(|e| format!("Failed to initialize camera: {}", e))?;

    // Start camera stream
    {
        let mut cam = camera.lock().await;
        cam.start_stream().map_err(|e| format!("Failed to start camera stream: {}", e))?;
    }

    // Create recorder
    let recorder = Recorder::new(&output_path, config)
        .map_err(|e| format!("Failed to create recorder: {}", e))?;

    // Generate session ID
    let session_id = format!("rec_{}", chrono::Utc::now().timestamp_millis());

    // Store session
    let session = RecordingSession {
        recorder,
        camera,
        is_running: true,
    };

    {
        let mut registry = RECORDER_REGISTRY.write().await;
        registry.insert(session_id.clone(), Arc::new(AsyncMutex::new(session)));
    }

    log::info!("Recording started: session {}", session_id);
    Ok(session_id)
}

/// Write frames from the camera to the recording
/// 
/// This should be called repeatedly to capture frames.
/// Returns the number of frames recorded so far.
#[command]
pub async fn record_frame(session_id: String) -> Result<u64, String> {
    let session_arc = {
        let registry = RECORDER_REGISTRY.read().await;
        registry.get(&session_id)
            .cloned()
            .ok_or_else(|| format!("Recording session not found: {}", session_id))?
    };

    let mut session = session_arc.lock().await;
    
    if !session.is_running {
        return Err("Recording is not running".to_string());
    }

    // Capture frame from camera
    let frame = {
        let mut camera = session.camera.lock().await;
        camera.capture_frame()
            .map_err(|e| format!("Failed to capture frame: {}", e))?
    };

    // Write to recorder
    session.recorder.write_frame(&frame)
        .map_err(|e| format!("Failed to write frame: {}", e))?;

    Ok(session.recorder.frame_count())
}

/// Stop recording and finalize the file
/// 
/// # Returns
/// * Recording statistics (frames, duration, file size, etc.)
#[command]
pub async fn stop_recording(session_id: String) -> Result<RecordingStats, String> {
    // Remove session from registry
    let session_arc = {
        let mut registry = RECORDER_REGISTRY.write().await;
        registry.remove(&session_id)
            .ok_or_else(|| format!("Recording session not found: {}", session_id))?
    };

    // Get exclusive access and stop
    let session = Arc::try_unwrap(session_arc)
        .map_err(|_| "Recording session is still in use".to_string())?
        .into_inner();

    // Stop camera stream
    {
        let mut camera = session.camera.lock().await;
        let _ = camera.stop_stream();
    }

    // Finish recording
    let stats = session.recorder.finish()
        .map_err(|e| format!("Failed to finalize recording: {}", e))?;

    log::info!("Recording stopped: {} frames, {:.2}s, {} bytes",
        stats.video_frames, stats.duration_secs, stats.bytes_written);

    Ok(stats)
}

/// Get the status of an active recording
#[command]
pub async fn get_recording_status(session_id: String) -> Result<RecordingStatus, String> {
    let session_arc = {
        let registry = RECORDER_REGISTRY.read().await;
        registry.get(&session_id)
            .cloned()
            .ok_or_else(|| format!("Recording session not found: {}", session_id))?
    };

    let session = session_arc.lock().await;
    
    Ok(RecordingStatus {
        session_id,
        is_running: session.is_running,
        frame_count: session.recorder.frame_count(),
        dropped_frames: session.recorder.dropped_frames(),
        duration_secs: session.recorder.duration(),
    })
}

/// List all active recording sessions
#[command]
pub async fn list_recording_sessions() -> Result<Vec<String>, String> {
    let registry = RECORDER_REGISTRY.read().await;
    Ok(registry.keys().cloned().collect())
}

/// Recording status information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecordingStatus {
    pub session_id: String,
    pub is_running: bool,
    pub frame_count: u64,
    pub dropped_frames: u64,
    pub duration_secs: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_status_serialization() {
        let status = RecordingStatus {
            session_id: "test_123".to_string(),
            is_running: true,
            frame_count: 100,
            dropped_frames: 2,
            duration_secs: 3.33,
        };
        
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("test_123"));
        assert!(json.contains("100"));
    }
}
