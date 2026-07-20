use crate::platform::{CameraSystem, PlatformInfo, SystemTestResult};
use crate::types::{CameraDeviceInfo, CameraFormat, Platform};
use tauri::command;

use crate::registry::{FeatureManifest, SystemRegistry};

/// Get the official system capabilities manifest
#[command]
pub async fn get_system_manifest() -> Vec<FeatureManifest> {
    SystemRegistry::get_manifest()
}

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

    // Get platform info — preserve error so callers can distinguish failure from absent backend
    let (platform_info, platform_info_error) = match CameraSystem::get_platform_info() {
        Ok(info) => (Some(info), None),
        Err(e) => {
            log::warn!("Platform info unavailable: {}", e);
            (None, Some(e.to_string()))
        }
    };

    // Get available cameras — preserve enumeration error
    let (cameras, camera_enumeration_error) = match CameraSystem::list_cameras() {
        Ok(cams) => (cams, None),
        Err(e) => {
            log::warn!("Camera enumeration failed: {}", e);
            (vec![], Some(e.to_string()))
        }
    };

    let (cameras, camera_enumeration_error) = match CameraSystem::list_cameras() {
        Ok(cams) => (cams, None),
        Err(e) => {
            let msg = e.to_string();
            log::warn!("Camera enumeration failed: {}", msg);
            (vec![], Some(msg))
        }
    };
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

    // Check permission status — preserve error
    let (permission_status, permission_error) =
        match crate::commands::permissions::check_camera_permission_status().await {
            Ok(p) => (p.status.to_string(), None),
            Err(e) => ("unknown".to_string(), Some(e)),
        };

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
        platform_info_error,
        camera_enumeration_error,
        permission_error,
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
    /// Version of the crabcamera crate.
    pub crate_version: String,
    /// Operating system platform (e.g., "windows", "macos").
    pub platform: String,
    /// Camera backend in use (e.g., "MediaFoundation", "AVFoundation").
    pub backend: String,
    /// Number of detected cameras.
    pub camera_count: usize,
    /// List of summarized camera devices.
    pub cameras: Vec<CameraSummary>,
    /// Status of camera permissions ("granted", "denied", "unknown").
    pub permission_status: String,
    /// List of enabled cargo features compiled into this build.
    pub features_enabled: Vec<String>,
    /// ISO 8601 timestamp of the diagnostics report.
    pub timestamp: String,
    /// Error from platform info query, if any.
    pub platform_info_error: Option<String>,
    /// Error from camera enumeration, if any.
    pub camera_enumeration_error: Option<String>,
    /// Error from permission check, if any.
    pub permission_error: Option<String>,
}

/// Summary of a camera device
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CameraSummary {
    /// Unique device ID.
    pub id: String,
    /// Human-readable device name.
    pub name: String,
    /// Whether the device is currently accessible.
    pub is_available: bool,
    /// Number of supported video formats.
    pub format_count: usize,
    /// Maximum supported resolution (width, height), if any.
    pub max_resolution: Option<(u32, u32)>,
}

/// Get list of Cargo features compiled into this build.
fn get_enabled_features() -> Vec<String> {
    [
        ("recording", cfg!(feature = "recording")),
        ("audio", cfg!(feature = "audio")),
        ("headless", cfg!(feature = "headless")),
        ("contextlite", cfg!(feature = "contextlite")),
    ]
    .into_iter()
    .filter(|(_, enabled)| *enabled)
    .map(|(name, _)| name.to_string())
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_current_platform_returns_known_value() {
        let platform = get_current_platform().await.expect("platform query should succeed");
        assert!(matches!(platform.as_str(), "windows" | "macos" | "linux" | "unknown"));
    }

    #[tokio::test]
    async fn test_get_recommended_format_has_valid_shape() {
        let format = get_recommended_format()
            .await
            .expect("recommended format should be available");
        assert!(format.width > 0);
        assert!(format.height > 0);
        assert!(format.fps > 0.0);
        assert!(!format.format_type.is_empty());
    }

    #[tokio::test]
    async fn test_get_optimal_settings_has_valid_shape() {
        let params = get_optimal_settings()
            .await
            .expect("optimal settings should be available");
        assert!(!params.device_id.is_empty());
        assert!(params.format.width > 0);
        assert!(params.format.height > 0);
        assert!(params.format.fps > 0.0);
    }

    #[test]
    fn test_get_enabled_features_contains_recording_when_enabled() {
        let features = get_enabled_features();

        #[cfg(feature = "recording")]
        assert!(features.iter().any(|f| f == "recording"));

        #[cfg(not(feature = "recording"))]
        assert!(!features.iter().any(|f| f == "recording"));
    }

    #[tokio::test]
    async fn test_system_diagnostics_shape() {
        let diagnostics = get_system_diagnostics()
            .await
            .expect("diagnostics should always return a report");

        assert!(!diagnostics.crate_version.is_empty());
        assert!(matches!(diagnostics.platform.as_str(), "windows" | "macos" | "linux" | "unknown"));
        assert!(!diagnostics.backend.is_empty());
        assert!(!diagnostics.permission_status.is_empty());
        assert!(!diagnostics.timestamp.is_empty());

        for cam in diagnostics.cameras {
            assert!(!cam.id.is_empty());
            assert!(!cam.name.is_empty());
        }
    }
}
