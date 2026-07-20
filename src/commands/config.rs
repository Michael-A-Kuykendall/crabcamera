use crate::config::CrabCameraConfig;
use std::sync::{Arc, LazyLock, RwLock};
use tauri::command;

static GLOBAL_CONFIG: LazyLock<Arc<RwLock<CrabCameraConfig>>> =
    LazyLock::new(|| Arc::new(RwLock::new(CrabCameraConfig::load_or_default())));

/// Get the current configuration
///
/// # Errors
/// Returns an `Err` if the global configuration lock is poisoned.
#[command]
pub async fn get_config() -> Result<CrabCameraConfig, String> {
    let config = GLOBAL_CONFIG.read().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

/// Update configuration
///
/// # Errors
/// Returns an `Err` if the new configuration fails validation, if the global
/// configuration lock is poisoned, or if the configuration cannot be saved to
/// disk.
#[command]
pub async fn update_config(new_config: CrabCameraConfig) -> Result<(), String> {
    // Validate first
    new_config.validate().map_err(|e| e.clone())?;

    {
        let mut config = GLOBAL_CONFIG.write().map_err(|e| e.to_string())?;
        *config = new_config.clone();
    }

    // Save to file
    new_config
        .save_to_file(CrabCameraConfig::default_path())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Reset configuration to defaults
///
/// # Errors
/// Returns an `Err` if the global configuration lock is poisoned or if the
/// default configuration cannot be saved to disk.
#[command]
pub async fn reset_config() -> Result<CrabCameraConfig, String> {
    let default_config = CrabCameraConfig::default();

    {
        let mut config = GLOBAL_CONFIG
            .write()
            .map_err(|e| format!("Failed to write config: {e}"))?;
        *config = default_config.clone();
    }

    // Save defaults to file
    default_config
        .save_to_file(CrabCameraConfig::default_path())
        .map_err(|e| e.to_string())?;

    Ok(default_config)
}

/// Get camera configuration
///
/// # Errors
/// Returns an `Err` if the global configuration lock is poisoned.
#[command]
pub async fn get_camera_config() -> Result<crate::config::CameraConfig, String> {
    let config = GLOBAL_CONFIG.read().map_err(|e| e.to_string())?;
    Ok(config.camera.clone())
}

/// Get quality configuration (full config object)
///
/// # Errors
/// Returns an `Err` if the global configuration lock is poisoned.
#[command]
pub async fn get_full_quality_config() -> Result<crate::config::QualityConfig, String> {
    let config = GLOBAL_CONFIG.read().map_err(|e| e.to_string())?;
    Ok(config.quality.clone())
}

/// Get storage configuration
///
/// # Errors
/// Returns an `Err` if the global configuration lock is poisoned.
#[command]
pub async fn get_storage_config() -> Result<crate::config::StorageConfig, String> {
    let config = GLOBAL_CONFIG.read().map_err(|e| e.to_string())?;
    Ok(config.storage.clone())
}

/// Get advanced configuration
///
/// # Errors
/// Returns an `Err` if the global configuration lock is poisoned.
#[command]
pub async fn get_advanced_config() -> Result<crate::config::AdvancedConfig, String> {
    let config = GLOBAL_CONFIG.read().map_err(|e| e.to_string())?;
    Ok(config.advanced.clone())
}

/// Update camera configuration
///
/// # Errors
/// Returns an `Err` if the global configuration lock is poisoned, if the
/// resulting configuration fails validation, or if it cannot be saved to disk.
#[command]
pub async fn update_camera_config(
    camera_config: crate::config::CameraConfig,
) -> Result<(), String> {
    let mut config = GLOBAL_CONFIG.write().map_err(|e| e.to_string())?;
    config.camera = camera_config;

    config.validate().map_err(|e| e.clone())?;

    config
        .save_to_file(CrabCameraConfig::default_path())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Update quality configuration (full config object)
///
/// # Errors
/// Returns an `Err` if the global configuration lock is poisoned, if the
/// resulting configuration fails validation, or if it cannot be saved to disk.
#[command]
pub async fn update_full_quality_config(
    quality_config: crate::config::QualityConfig,
) -> Result<(), String> {
    let mut config = GLOBAL_CONFIG.write().map_err(|e| e.to_string())?;
    config.quality = quality_config;

    config.validate().map_err(|e| e.clone())?;

    config
        .save_to_file(CrabCameraConfig::default_path())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Update storage configuration
///
/// # Errors
/// Returns an `Err` if the global configuration lock is poisoned, if the
/// resulting configuration fails validation, or if it cannot be saved to disk.
#[command]
pub async fn update_storage_config(
    storage_config: crate::config::StorageConfig,
) -> Result<(), String> {
    let mut config = GLOBAL_CONFIG.write().map_err(|e| e.to_string())?;
    config.storage = storage_config;

    config.validate().map_err(|e| e.clone())?;

    config
        .save_to_file(CrabCameraConfig::default_path())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Update advanced configuration
///
/// # Errors
/// Returns an `Err` if the global configuration lock is poisoned, if the
/// resulting configuration fails validation, or if it cannot be saved to disk.
#[command]
pub async fn update_advanced_config(
    advanced_config: crate::config::AdvancedConfig,
) -> Result<(), String> {
    let mut config = GLOBAL_CONFIG.write().map_err(|e| e.to_string())?;
    config.advanced = advanced_config;

    config.validate().map_err(|e| e.clone())?;

    config
        .save_to_file(CrabCameraConfig::default_path())
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_config() {
        let result = get_config().await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.camera.default_fps, 30);
    }

    #[tokio::test]
    async fn test_reset_config() {
        let result = reset_config().await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.camera.default_resolution, [1920, 1080]);
    }

    #[tokio::test]
    async fn test_get_camera_config() {
        let result = get_camera_config().await;
        assert!(result.is_ok());

        let camera_config = result.unwrap();
        assert!(camera_config.auto_reconnect);
    }

    #[tokio::test]
    async fn test_other_getters_return_values() {
        assert!(get_full_quality_config().await.is_ok());
        assert!(get_storage_config().await.is_ok());
        assert!(get_advanced_config().await.is_ok());
    }

    #[tokio::test]
    async fn test_update_config_and_subconfigs() {
        let base = CrabCameraConfig::default();
        update_config(base.clone())
            .await
            .expect("update_config should succeed for default config");

        update_camera_config(base.camera.clone())
            .await
            .expect("update_camera_config should succeed");
        update_full_quality_config(base.quality.clone())
            .await
            .expect("update_full_quality_config should succeed");
        update_storage_config(base.storage.clone())
            .await
            .expect("update_storage_config should succeed");
        update_advanced_config(base.advanced.clone())
            .await
            .expect("update_advanced_config should succeed");
    }
}
