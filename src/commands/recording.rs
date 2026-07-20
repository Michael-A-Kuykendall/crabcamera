//! Tauri commands for video recording
//!
//! These commands provide an interface for recording video from cameras.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex as SyncMutex};
use tauri::command;
use tokio::sync::RwLock;

use crate::constants::{
    DEFAULT_CAMERA_ID, RECORDING_QUALITY_PRESET_1080P, RECORDING_QUALITY_PRESET_4K,
    RECORDING_QUALITY_PRESET_720P, RECORDING_QUALITY_PRESET_HIGH, RECORDING_QUALITY_PRESET_LOW,
    RECORDING_QUALITY_PRESET_MEDIUM, RECORDING_SESSION_PREFIX,
};
#[cfg(feature = "audio")]
use crate::constants::{AUDIO_BITRATE, AUDIO_CHANNELS, AUDIO_DEVICE_DEFAULT, AUDIO_SAMPLE_RATE};
use crate::platform::PlatformCamera;
use crate::recording::{Recorder, RecordingConfig, RecordingQuality, RecordingStats};
use crate::types::CameraFormat;

// Global recorder registry
static RECORDER_REGISTRY: LazyLock<Arc<RwLock<HashMap<String, Arc<SyncMutex<RecordingSession>>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Active recording session combining camera and recorder
struct RecordingSession {
    recorder: Option<Recorder>,
    camera: Arc<SyncMutex<PlatformCamera>>,
    is_running: bool,
}

/// Start recording from a camera to a file
///
/// # Arguments
/// * `device_id` - Camera device ID (or "0" for default)
/// * `output_path` - Path to save the MP4 file
/// * `width` - Video width in pixels
/// * `height` - Video height in pixels
///
/// # Note on argument count
/// This command has more arguments than clippy's default threshold because Tauri `#[command]`
/// functions must receive all parameters as flat primitives — Tauri's JS-to-Rust invoke bridge
/// does not support deserializing a wrapper struct from `invoke()` without an explicit
/// `serde` workaround. Until a `RecordingRequest` newtype is threaded through both the Rust
/// command signature and the frontend `invoke` call (changing the public JS API surface),
/// the flat signature is intentional. The suppression is scoped to this one function.
/// * `fps` - Target frame rate
/// * `quality` - Recording quality preset (optional)
/// * `title` - Metadata title (optional)
/// * `audio_device_id` - Audio device ID for recording (optional, enables audio when provided)
///
/// # Returns
/// * Session ID for tracking the recording
///
/// # Errors
/// Returns an `Err` if the camera cannot be initialized or its stream cannot
/// be started, if the camera mutex is poisoned, or if the [`Recorder`] cannot
/// be created.
#[allow(clippy::too_many_arguments)]
#[command]
pub async fn start_recording(
    device_id: Option<String>,
    output_path: String,
    width: u32,
    height: u32,
    fps: f64,
    quality: Option<String>,
    title: Option<String>,
    #[cfg(feature = "audio")] audio_device_id: Option<String>,
) -> Result<String, String> {
    let camera_id = device_id.unwrap_or_else(|| DEFAULT_CAMERA_ID.to_string());

    #[cfg(feature = "audio")]
    {
        if let Some(ref audio_id) = audio_device_id {
            log::info!(
                "Starting recording from camera {camera_id} with audio {audio_id} to {output_path}"
            );
        } else {
            log::info!(
                "Starting recording from camera {camera_id} (no audio) to {output_path}"
            );
        }
    }
    #[cfg(not(feature = "audio"))]
    log::info!(
        "Starting recording from camera {} to {}",
        camera_id,
        output_path
    );

    // Parse quality preset
    let recording_quality = match quality.as_deref() {
        Some(q) if q == RECORDING_QUALITY_PRESET_LOW || q == RECORDING_QUALITY_PRESET_720P => {
            Some(RecordingQuality::Low)
        }
        Some(q) if q == RECORDING_QUALITY_PRESET_MEDIUM || q == RECORDING_QUALITY_PRESET_1080P => {
            Some(RecordingQuality::Medium)
        }
        Some(q) if q == RECORDING_QUALITY_PRESET_HIGH || q == RECORDING_QUALITY_PRESET_4K => {
            Some(RecordingQuality::High)
        }
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

    // Add audio configuration if audio device specified
    // Per #TauriAudioCommands: ! start_recording_accepts_audio_device_option
    #[cfg(feature = "audio")]
    if let Some(audio_id) = audio_device_id {
        config = config.with_audio(crate::recording::AudioConfig {
            device_id: if audio_id == AUDIO_DEVICE_DEFAULT {
                None
            } else {
                Some(audio_id)
            },
            sample_rate: AUDIO_SAMPLE_RATE,
            channels: AUDIO_CHANNELS,
            bitrate: AUDIO_BITRATE,
        });
    }

    // Initialize camera
    let camera = super::capture::get_or_create_camera(
        camera_id.clone(),
        CameraFormat::new(config.width, config.height, fps as f32),
    )
    .await
    .map_err(|e| format!("Failed to initialize camera: {e}"))?;

    // Start camera stream
    {
        let mut cam = camera
            .lock()
            .map_err(|_| "Camera mutex poisoned".to_string())?;
        cam.start_stream()
            .map_err(|e| format!("Failed to start camera stream: {e}"))?;
    }

    // Create recorder
    let recorder = Recorder::new(&output_path, config)
        .map_err(|e| format!("Failed to create recorder: {e}"))?;

    // Generate session ID
    let session_id = format!(
        "{}{}",
        RECORDING_SESSION_PREFIX,
        chrono::Utc::now().timestamp_millis()
    );

    // Store session
    let session = RecordingSession {
        recorder: Some(recorder),
        camera,
        is_running: true,
    };

    {
        let mut registry = RECORDER_REGISTRY.write().await;
        registry.insert(session_id.clone(), Arc::new(SyncMutex::new(session)));
    }

    log::info!("Recording started: session {session_id}");
    Ok(session_id)
}

/// Write frames from the camera to the recording
///
/// This should be called repeatedly to capture frames.
/// Returns the number of frames recorded so far.
///
/// # Errors
/// Returns an `Err` if the recording session is not found, if the session or
/// camera mutex is poisoned, if recording is not running, if the camera frame
/// capture fails, if no recorder is available, or if writing the frame fails.
#[command]
pub async fn record_frame(session_id: String) -> Result<u64, String> {
    let session_arc = {
        let registry = RECORDER_REGISTRY.read().await;
        registry
            .get(&session_id)
            .cloned()
            .ok_or_else(|| format!("Recording session not found: {session_id}"))?
    };

    let mut session = session_arc
        .lock()
        .map_err(|_| "Mutex poisoned".to_string())?;

    if !session.is_running {
        return Err("Recording is not running".to_string());
    }

    // Capture frame from camera
    let frame = {
        let mut camera = session
            .camera
            .lock()
            .map_err(|_| "Mutex poisoned".to_string())?;
        camera
            .capture_frame()
            .map_err(|e| format!("Failed to capture frame: {e}"))?
    };

    // Write to recorder
    let recorder = session
        .recorder
        .as_mut()
        .ok_or_else(|| "Recorder not available".to_string())?;
    recorder
        .write_frame(&frame)
        .map_err(|e| format!("Failed to write frame: {e}"))?;

    Ok(recorder.frame_count())
}

/// Stop recording and finalize the file
///
/// # Returns
/// * Recording statistics (frames, duration, file size, etc.)
///
/// # Errors
/// Returns an `Err` if the recording session is not found, if the session or
/// camera mutex is poisoned, if the recorder has already been taken, or if
/// finalizing the recording fails.
#[command]
pub async fn stop_recording(session_id: String) -> Result<RecordingStats, String> {
    // Remove session from registry
    let session_arc = {
        let mut registry = RECORDER_REGISTRY.write().await;
        registry
            .remove(&session_id)
            .ok_or_else(|| format!("Recording session not found: {session_id}"))?
    };

    // Get exclusive access and stop
    let mut session = session_arc
        .lock()
        .map_err(|_| "Mutex poisoned".to_string())?;

    // Stop camera stream
    {
        let mut camera = session
            .camera
            .lock()
            .map_err(|_| "Camera mutex poisoned".to_string())?;
        let _ = camera.stop_stream();
    }

    // Finish recording
    let stats = session
        .recorder
        .take()
        .ok_or_else(|| "Recorder already taken".to_string())?
        .finish()
        .map_err(|e| format!("Failed to finalize recording: {e}"))?;

    log::info!(
        "Recording stopped: {} frames, {:.2}s, {} bytes",
        stats.video_frames,
        stats.duration_secs,
        stats.bytes_written
    );

    Ok(stats)
}

/// Get the status of an active recording
///
/// # Errors
/// Returns an `Err` if the recording session is not found, or if the session
/// or camera mutex is poisoned, or if no recorder is available.
#[command]
pub async fn get_recording_status(session_id: String) -> Result<RecordingStatus, String> {
    let session_arc = {
        let registry = RECORDER_REGISTRY.read().await;
        registry
            .get(&session_id)
            .cloned()
            .ok_or_else(|| format!("Recording session not found: {session_id}"))?
    };

    let session = session_arc
        .lock()
        .map_err(|_| "Mutex poisoned".to_string())?;

    let recorder = session
        .recorder
        .as_ref()
        .ok_or_else(|| "Recorder not available".to_string())?;

    // Build audio status if audio feature enabled
    #[cfg(feature = "audio")]
    let audio_status = if recorder.audio_enabled() {
        Some(AudioStatus {
            enabled: true,
            failed: recorder.audio_failed(),
        })
    } else {
        None
    };

    Ok(RecordingStatus {
        session_id,
        is_running: session.is_running,
        frame_count: recorder.frame_count(),
        dropped_frames: recorder.dropped_frames(),
        duration_secs: recorder.duration(),
        #[cfg(feature = "audio")]
        audio_status,
    })
}

/// List all active recording sessions
///
/// # Errors
/// This function always succeeds and never returns an `Err`.
#[command]
pub async fn list_recording_sessions() -> Result<Vec<String>, String> {
    let registry = RECORDER_REGISTRY.read().await;
    Ok(registry.keys().cloned().collect())
}

/// Recording status information
/// Per #`AudioErrorRecovery`: ! `session_status_reflects_audio_state`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingStatus {
    /// Unique identifier for the recording session.
    pub session_id: String,
    /// Whether the recording is actively capturing.
    pub is_running: bool,
    /// Total video frames successfully encoded.
    pub frame_count: u64,
    /// Frames dropped due to performance issues.
    pub dropped_frames: u64,
    /// Duration of the recording in seconds.
    pub duration_secs: f64,
    /// Audio recording status (None if audio not enabled)
    #[cfg(feature = "audio")]
    pub audio_status: Option<AudioStatus>,
}

/// Audio status within a recording session
/// Per #`AudioErrorRecovery`: ! `session_status_reflects_audio_state`
#[cfg(feature = "audio")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioStatus {
    /// Whether audio recording is enabled
    pub enabled: bool,
    /// Whether audio capture has failed
    pub failed: bool,
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
            #[cfg(feature = "audio")]
            audio_status: Some(AudioStatus {
                enabled: true,
                failed: false,
            }),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("test_123"));
        assert!(json.contains("100"));
        // JSON serialization uses camelCase for frontend compatibility
        #[cfg(feature = "audio")]
        {
            assert!(json.contains("audioStatus"));
        }
    }

    #[tokio::test]
    async fn test_write_frame_to_missing_session_returns_error() {
        let result = record_frame("nonexistent_session_xyz".to_string()).await;
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("nonexistent_session_xyz"),
            "error should identify the missing session, got: {msg}"
        );
    }

    #[tokio::test]
    async fn test_get_recording_status_missing_session_returns_error() {
        let result = get_recording_status("no_such_session_abc".to_string()).await;
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("no_such_session_abc"),
            "error should identify the missing session, got: {msg}"
        );
    }

    #[tokio::test]
    async fn test_stop_recording_missing_session_returns_error() {
        let result = stop_recording("ghost_session_999".to_string()).await;
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("ghost_session_999"),
            "error should identify the missing session, got: {msg}"
        );
    }
}
