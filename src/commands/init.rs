use crate::platform::{CameraSystem, PlatformInfo, SystemTestResult};
use crate::types::{CameraDeviceInfo, CameraFormat, Platform};
use tauri::command;

/// Initialize the camera system for the current platform
#[command]
pub async fn initialize_camera_system() -> Result<String, String> {
    match CameraSystem::initialize() {
        Ok(message) => {
            log::info!("Camera system initialized: {}", message);
            Ok(message)
        }
        Err(e) => {
            log::error!("Failed to initialize camera system: {}", e);
            Err(format!("Failed to initialize camera system: {}", e))
        }
    }
}

/// Get list of available cameras on the current platform
#[command]
pub async fn get_available_cameras() -> Result<Vec<CameraDeviceInfo>, String> {
    match CameraSystem::list_cameras() {
        Ok(cameras) => {
            log::info!("Found {} cameras", cameras.len());
            for camera in &cameras {
                log::debug!(
                    "Camera: {} - {} (Available: {})",
                    camera.id,
                    camera.name,
                    camera.is_available
                );
            }
            Ok(cameras)
        }
        Err(e) => {
            log::error!("Failed to list cameras: {}", e);
            Err(format!("Failed to list cameras: {}", e))
        }
    }
}

/// Get platform-specific information
#[command]
pub async fn get_platform_info() -> Result<PlatformInfo, String> {
    match CameraSystem::get_platform_info() {
        Ok(info) => {
            log::info!(
                "Platform: {} using {}",
                info.platform.as_str(),
                info.backend
            );
            Ok(info)
        }
        Err(e) => {
            log::error!("Failed to get platform info: {}", e);
            Err(format!("Failed to get platform info: {}", e))
        }
    }
}

/// Test camera system functionality
#[command]
pub async fn test_camera_system() -> Result<SystemTestResult, String> {
    log::info!("Running camera system test...");

    match CameraSystem::test_system() {
        Ok(result) => {
            log::info!(
                "Camera system test completed: {} cameras found on {}",
                result.cameras_found,
                result.platform.as_str()
            );

            for (camera_id, test_result) in &result.test_results {
                match test_result {
                    crate::platform::CameraTestResult::Success => {
                        log::info!("Camera {} test: SUCCESS", camera_id);
                    }
                    crate::platform::CameraTestResult::InitError(err) => {
                        log::warn!("Camera {} init error: {}", camera_id, err);
                    }
                    crate::platform::CameraTestResult::CaptureError(err) => {
                        log::warn!("Camera {} capture error: {}", camera_id, err);
                    }
                    crate::platform::CameraTestResult::NotAvailable => {
                        log::info!("Camera {} not available", camera_id);
                    }
                }
            }

            Ok(result)
        }
        Err(e) => {
            log::error!("Camera system test failed: {}", e);
            Err(format!("Camera system test failed: {}", e))
        }
    }
}

/// Get the current platform information
#[command]
pub async fn get_current_platform() -> Result<String, String> {
    let platform = Platform::current();
    Ok(platform.as_str().to_string())
}

/// Check if a specific camera is available
#[command]
pub async fn check_camera_availability(device_id: String) -> Result<bool, String> {
    match CameraSystem::list_cameras() {
        Ok(cameras) => {
            let is_available = cameras
                .iter()
                .find(|camera| camera.id == device_id)
                .map(|camera| camera.is_available)
                .unwrap_or(false);

            log::debug!("Camera {} availability: {}", device_id, is_available);
            Ok(is_available)
        }
        Err(e) => {
            log::error!("Failed to check camera availability: {}", e);
            Err(format!("Failed to check camera availability: {}", e))
        }
    }
}

/// Get supported formats for a specific camera
#[command]
pub async fn get_camera_formats(device_id: String) -> Result<Vec<CameraFormat>, String> {
    match CameraSystem::list_cameras() {
        Ok(cameras) => {
            if let Some(camera) = cameras.iter().find(|c| c.id == device_id) {
                log::debug!(
                    "Camera {} supports {} formats",
                    device_id,
                    camera.supports_formats.len()
                );
                Ok(camera.supports_formats.clone())
            } else {
                let msg = format!("Camera with ID '{}' not found", device_id);
                log::warn!("{}", msg);
                Err(msg)
            }
        }
        Err(e) => {
            log::error!("Failed to get camera formats: {}", e);
            Err(format!("Failed to get camera formats: {}", e))
        }
    }
}

/// Get recommended format for high-quality photography
#[command]
pub async fn get_recommended_format() -> Result<CameraFormat, String> {
    let format = crate::platform::optimizations::get_photography_format();
    log::info!(
        "Recommended photography format: {}x{} @ {}fps ({})",
        format.width,
        format.height,
        format.fps,
        format.format_type
    );
    Ok(format)
}

/// Get optimal camera settings for high-quality capture
#[command]
pub async fn get_optimal_settings() -> Result<crate::types::CameraInitParams, String> {
    let params = crate::platform::optimizations::get_optimal_settings();
    log::info!(
        "Optimal settings: Device {} with {}x{} @ {}fps",
        params.device_id,
        params.format.width,
        params.format.height,
        params.format.fps
    );
    Ok(params)
}

/// Comprehensive system diagnostics for troubleshooting
///
/// Returns detailed information about the camera system state,
/// useful for debugging issues and verifying setup.
#[command]
pub async fn get_system_diagnostics() -> Result<SystemDiagnostics, String> {
    log::info!("Running system diagnostics...");

    let platform = Platform::current();
    let crate_version = crate::VERSION.to_string();

    // Get platform info
    let platform_info = CameraSystem::get_platform_info().ok();

    // Get available cameras
    let cameras = CameraSystem::list_cameras().unwrap_or_default();
    let camera_count = cameras.len();

    // Build camera summaries
    let camera_summaries: Vec<CameraSummary> = cameras
        .iter()
        .map(|c| CameraSummary {
            id: c.id.clone(),
            name: c.name.clone(),
            is_available: c.is_available,
            format_count: c.supports_formats.len(),
            max_resolution: c
                .supports_formats
                .iter()
                .map(|f| (f.width, f.height))
                .max_by_key(|(w, h)| w * h),
        })
        .collect();

    // Check permission status
    let permission_status = crate::commands::permissions::check_camera_permission_status()
        .await
        .map(|p| p.status.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let diagnostics = SystemDiagnostics {
        crate_version,
        platform: platform.as_str().to_string(),
        backend: platform_info
            .as_ref()
            .map(|p| p.backend.clone())
            .unwrap_or_else(|| "unknown".to_string()),
        camera_count,
        cameras: camera_summaries,
        permission_status,
        features_enabled: get_enabled_features(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    log::info!(
        "Diagnostics complete: {} cameras on {} ({})",
        diagnostics.camera_count,
        diagnostics.platform,
        diagnostics.backend
    );

    Ok(diagnostics)
}

/// System diagnostics response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemDiagnostics {
    pub crate_version: String,
    pub platform: String,
    pub backend: String,
    pub camera_count: usize,
    pub cameras: Vec<CameraSummary>,
    pub permission_status: String,
    pub features_enabled: Vec<String>,
    pub timestamp: String,
}

/// Summary of a camera device
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CameraSummary {
    pub id: String,
    pub name: String,
    pub is_available: bool,
    pub format_count: usize,
    pub max_resolution: Option<(u32, u32)>,
}

/// Get list of enabled features
fn get_enabled_features() -> Vec<String> {
    vec![
        "camera_capture".to_string(),
        "quality_validation".to_string(),
        "device_monitoring".to_string(),
        "focus_stacking".to_string(),
    ]
}
