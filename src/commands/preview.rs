use std::sync::Arc;
use tauri::command;

use crate::preview::{PreviewConfig, PreviewStream};

static PREVIEW_HANDLE: tokio::sync::RwLock<Option<Arc<PreviewStream>>> =
    tokio::sync::RwLock::const_new(None);

#[command]
pub async fn start_preview_stream(
    device_id: String,
    fps_target: u32,
    downscale: f32,
    jpeg_quality: u8,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let config = PreviewConfig {
        fps_target,
        downscale,
        quality_sample_rate: 5,
        analyze_at_full_res: false,
        jpeg_quality,
    };

    let stream = PreviewStream::new();
    let camera = crate::platform::get_or_create_camera(
        device_id.clone(),
        crate::types::CameraFormat::standard(),
    )
    .await
    .map_err(|e| format!("Failed to get camera: {e}"))?;

    stream
        .start(
            camera.clone(),
            config,
            crate::quality::smart_trigger::SmartTrigger::new(
                crate::quality::smart_trigger::TriggerConfig::default(),
            ),
            Some(app),
        )
        .await?;

    let mut guard = PREVIEW_HANDLE.write().await;
    *guard = Some(Arc::new(stream));

    Ok("preview_started".to_string())
}

#[command]
pub async fn stop_preview_stream() -> Result<String, String> {
    let mut guard = PREVIEW_HANDLE.write().await;
    if let Some(ref stream) = *guard {
        stream.stop();
        *guard = None;
        Ok("preview_stopped".to_string())
    } else {
        Err("No active preview stream".to_string())
    }
}
